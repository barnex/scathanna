use super::internal::*;
use rayon::prelude::*;
use std::hash::Hash;
use std::sync::Mutex;
use std::thread;
use std::{panic, sync::mpsc::*};

pub struct Bakery {
	to_worker: Sender<Request>,
	from_worker: Receiver<Response>,
}

enum Request {
	UpdateCell(ivec3, Arc<Node>),
	RequestVAO(ivec3),
	RequestLightmap(ivec3),
	RequestFullBake(ivec3, PathBuf),
}
use Request::*;

type Response = Vec<(ivec3, CellBuffer)>;

impl Bakery {
	pub fn new(voxels: &Voxels) -> Self {
		let (to_worker, from_ctrl) = channel();
		let (to_ctrl, from_worker) = channel();

		let mut worker = Worker {
			voxels: voxels.cow_clone(),
			from_ctrl,
			to_ctrl,
			vao_backlog: default(),
			lightmap_backlog: default(),
		};

		thread::spawn(move || worker.serve_loop());

		Self { to_worker, from_worker }
	}

	pub fn update_cell(&mut self, cell_pos: ivec3, cell: Arc<Node>) {
		self.send(UpdateCell(cell_pos, cell))
	}

	pub fn request_vao(&mut self, cell_pos: ivec3) {
		self.send(RequestVAO(cell_pos))
	}

	pub fn request_lightmap(&mut self, cell_pos: ivec3) {
		self.send(RequestLightmap(cell_pos))
	}

	pub fn request_full_bake(&mut self, center: ivec3, dir: &Path) {
		self.send(RequestFullBake(center, dir.to_owned()))
	}

	fn send(&mut self, request: Request) {
		self.to_worker.send(request).expect("Bakery: worker died")
	}

	pub fn try_recv(&mut self) -> Option<Vec<(ivec3, CellBuffer)>> {
		try_recv(&mut self.from_worker)
	}
}

struct Worker {
	voxels: Voxels,
	vao_backlog: HashSet<ivec3>,
	lightmap_backlog: HashSet<ivec3>,
	from_ctrl: Receiver<Request>,
	to_ctrl: Sender<Response>,
}

impl Worker {
	// TODO: return Result
	fn serve_loop(&mut self) -> ! {
		loop {
			println!("Bakery: idle");
			self.wait_and_drain();
			while !self.backlog_is_empty() {
				self.bake_one_fast();
				self.try_drain();
			}
		}
	}

	/// block until there is a message,
	/// then handle all further messages until none is available at the moment.
	fn wait_and_drain(&mut self) {
		let first = self.recv();
		self.handle_message(first);
		self.try_drain();
	}

	fn try_drain(&mut self) {
		while let Some(msg) = self.try_recv() {
			self.handle_message(msg)
		}
	}

	fn handle_message(&mut self, msg: Request) {
		match msg {
			UpdateCell(cell_pos, cell) => self.update_cell(cell_pos, cell),
			RequestVAO(cell_pos) => self.request_vao(cell_pos),
			RequestLightmap(cell_pos) => self.request_lightmap(cell_pos),
			RequestFullBake(center, dir) => self.request_full_bake(center, dir),
		}
	}

	fn update_cell(&mut self, cell_pos: ivec3, cell: Arc<Node>) {
		self.voxels.set_cell(cell_pos, cell)
	}

	fn request_vao(&mut self, cell_pos: ivec3) {
		self.vao_backlog.insert(cell_pos);
	}

	fn request_lightmap(&mut self, cell_pos: ivec3) {
		self.lightmap_backlog.insert(cell_pos);
	}

	// High-quality lightmap baking, save to file in addition to showing in editor.
	// TODO: should be cancelled upon edit, new bake request.
	fn request_full_bake(&mut self, center: ivec3, dir: PathBuf) {
		println!("Full bake to {:?}. Go get a coffee...", &dir);

		// Sort cells to bake by distance from camera `center`,
		// so that we will see nearby cells first.
		let mut cells_to_bake = self.voxels.iter_cell_positions().collect::<Vec<_>>();
		cells_to_bake.sort_unstable_by_key(|&pos| ((pos + (Voxels::ICELL_SIZE / 2) * ivec3::ONES) - center).len2());

		let to_ctrl = Mutex::new(self.to_ctrl.clone());
		cells_to_bake.into_par_iter().for_each(|cell_pos| {
			if let Some(model) = bake_cell_buffer(&self.voxels, cell_pos, true) {
				let fname = VoxelWorld::lightmap_file(&dir, cell_pos);
				match imageutil::save(model.lightmap.image(), &fname) {
					Ok(_) => println!("Bakery: saved {:?}", &fname),
					Err(e) => println!("Bakery: ERROR saving {:?}: {}", &fname, e), // Error happens async, can't do much more than print
				}
				to_ctrl.lock().unwrap().send(vec![(cell_pos, model)]).unwrap()
			}
		})
	}

	fn backlog_is_empty(&self) -> bool {
		self.lightmap_backlog.is_empty() && self.vao_backlog.is_empty()
	}

	// Low-quality lightmaps used for interactive map editing:
	// bake all cells from the VAO backlog (high priority, should be baked together to avoid gaps),
	// or one from the lightmap backlog (low priority).
	fn bake_one_fast(&mut self) {
		if !self.vao_backlog.is_empty() {
			println!("Bakery: vao backlog size: {}", self.vao_backlog.len());
			let models = self
				.vao_backlog
				.par_drain()
				.map(|cell_pos| (cell_pos, bake_cell_buffer(&self.voxels, cell_pos, false)))
				.filter(|(_, m)| m.is_some())
				.map(|(cell_pos, m)| (cell_pos, m.unwrap()))
				.collect();
			self.send(models)
		} else if let Some(cell_pos) = pop(&mut self.lightmap_backlog) {
			//println!("Bakery: lightmap backlog size: {}", self.lightmap_backlog.len());
			if let Some(result) = bake_cell_buffer(&self.voxels, cell_pos, false) {
				self.send(vec![(cell_pos, result)])
			}
		}
	}

	fn recv(&mut self) -> Request {
		self.from_ctrl.recv().unwrap()
	}

	fn try_recv(&mut self) -> Option<Request> {
		try_recv(&mut self.from_ctrl)
	}

	fn send(&mut self, msg: Response) {
		self.to_ctrl.send(msg).unwrap()
	}
}

/// Take an arbitrary element from a set.
fn pop<T>(set: &mut HashSet<T>) -> Option<T>
where
	T: Eq + Hash + Copy,
{
	match set.iter().next() {
		Some(&v) => set.take(&v),
		None => None,
	}
}

// Non-blocking receive, panics on disconnect.
fn try_recv<T>(chan: &mut Receiver<T>) -> Option<T> {
	match chan.try_recv() {
		Ok(v) => Some(v),
		Err(TryRecvError::Empty) => None,
		Err(TryRecvError::Disconnected) => panic!("Bakery: try_recv: Disconnected"),
	}
}
