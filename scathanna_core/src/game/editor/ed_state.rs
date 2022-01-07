use super::internal::*;
use std::cell::RefCell;
use std::fs;

pub struct EdState {
	// Input
	last_tick: Instant,
	input_state: InputState,
	mouse_sens: f64,

	// World rendering
	engine: Rc<Engine>,
	camera: Camera,
	models: ModelPack,

	voxel_world: VoxelWorld,
	metadata: Metadata,
	spawn_point_model: Model,

	log_msg: RefCell<String>,

	// Editing
	cursor_size: uvec3,
	current_paint: VoxelType,
	stdin: StdinPipe,
	dir: PathBuf,
}

impl EdState {
	/// EdState with empty world.
	fn new(engine: Rc<Engine>, dir: &Path, voxel_world: VoxelWorld, metadata: Metadata) -> Self {
		Self {
			dir: dir.to_owned(),
			last_tick: Instant::now(),
			input_state: InputState::new(),
			mouse_sens: 0.001, // TODO
			camera: Camera::new(vec3(0.0, 0.0, 0.0)),

			metadata,

			voxel_world,
			models: ModelPack::new(engine.clone()).expect("load model pack"),
			spawn_point_model: Model::new(engine.wavefront_obj("box").expect("load model"), Material::UniformColor(YELLOW)),
			engine,
			cursor_size: uvec3(1, 1, 1),
			current_paint: VoxelType(1),
			stdin: pipe_stdin(),
			log_msg: RefCell::new(dir.to_string_lossy().into()),
		}
	}

	/// Mostly empty EdState to start building a new map from.
	pub fn create_new(dir: &Path) -> Result<Self> {
		fs::create_dir_all(dir)?;
		let mut voxels = Voxels::new();
		voxels.set_range(&Cuboid::cube(ivec3(0, 0, 0), Voxels::CELL_SIZE), VoxelType(1));
		let voxel_world = VoxelWorld::new(voxels);
		let engine = Rc::new(Engine::new());
		let metadata = Metadata::new();
		Ok(Self::new(engine, dir, voxel_world, metadata))
	}

	/// Load EdState from a `my_map.sc/` directory.
	pub fn load(dir: &Path) -> Result<Self> {
		let engine = Rc::new(Engine::new());
		let voxel_world = VoxelWorld::load(&engine, dir)?;
		let metadata = Metadata::load(&dir.join(MapData::METADATA_FILE)).unwrap_or_default();
		Ok(Self::new(engine, dir, voxel_world, metadata))
	}

	/// Save EdState to its directory.
	pub fn save(&self) -> Result<()> {
		self.log(format!("saving {:?}", &self.dir));
		self.voxel_world.save(&self.dir)?;
		self.metadata.save(&self.dir.join(MapData::METADATA_FILE))?;
		Ok(())
	}

	/// Get Voxels.
	fn voxels(&self) -> &Voxels {
		self.voxel_world.voxels()
	}

	/// Set Voxels.
	fn set_range(&mut self, range: &Cuboid, voxel: VoxelType) {
		self.voxel_world.set_range(range, voxel);
	}

	// __________________________________________________________________________________ event handling

	/// Handle keyboard input.
	pub fn on_key(&mut self, k: Key, pressed: bool) {
		self.input_state.record_key(k, pressed)
	}

	/// Handle mouse input.
	pub fn on_mouse_move(&mut self, x: f64, y: f64) {
		self.input_state.record_mouse((x * self.mouse_sens, y * self.mouse_sens))
	}

	/// Handle draw request.
	pub fn draw(&mut self, width: u32, height: u32) {
		self.engine.set_camera((width, height), &self.camera);
		self.engine.clear(0.8, 0.8, 1.0);
		self.voxel_world.draw(&self.engine, &self.camera);
		self.engine.print_bottom_left(WHITE, &self.log_msg.borrow());
		self.engine.draw_crosshair();
		self.draw_spawn_points();
		self.draw_pickup_points();
		self.draw_cursor();
	}

	// __________________________________________________________________________________ internal logic

	/// Advance time since last tick.
	fn tick(&mut self) {
		let dt = self.update_dt();

		self.handle_stdin();
		self.handle_save();
		self.rotate_camera();
		self.move_camera(dt.as_secs_f32());
		self.handle_editing();

		self.input_state.clear(); // must be last
	}

	fn handle_stdin(&mut self) {
		if let Some(cmd) = self.stdin.try_read() {
			if cmd.len() != 0 {
				self.handle_command(&cmd);
			}
		}
	}

	fn handle_command(&mut self, cmd: &str) {
		match cmd {
			"pos" => println!("cursor position: {}", self.cursor_range().map(|c| c.min).unwrap_or_default()),
			"spawn" => {
				if let Some(pos) = self.cursor_position() {
					self.add_spawn_point(pos)
				}
			}
			"pickup" => {
				if let Some(pos) = self.cursor_position() {
					self.add_pickup_point(pos)
				}
			}
			unknown => println!("unknown command: {}", unknown),
		}
	}

	fn add_spawn_point(&mut self, pos: ivec3) {
		self.metadata.spawn_points.push(SpawnPoint { pos });
	}

	fn add_pickup_point(&mut self, pos: ivec3) {
		self.metadata.pickup_points.push(PickupPoint { pos, taken: true });
	}

	fn handle_save(&self) {
		if self.input_state.is_pressed(Key::Save) {
			if let Err(e) = self.save() {
				self.log(format!("ERROR saving: {}", e))
			}
		}
	}

	fn rotate_camera(&mut self) {
		self.camera.orientation.yaw = self.input_state.mouse_yaw();
		self.camera.orientation.pitch = self.input_state.mouse_pitch();
	}

	fn move_camera(&mut self, dt: f32) {
		let fly_dir = fly_dir(self.camera.orientation.yaw, &self.input_state);
		let speed = 50.0;
		self.camera.position += speed * dt * fly_dir;
	}

	fn handle_editing(&mut self) {
		// Select paint.
		for (i, key) in Key::NUMERIC_KEYS.iter().copied().enumerate() {
			if self.input_state.is_pressed(key) {
				self.set_current_paint(VoxelType(i as u8 + 1))
			}
		}

		// Change cursor size.
		if self.input_state.is_pressed(Key::ScrollNext) {
			self.try_set_cursor_size(self.cursor_size * 2)
		}

		if self.input_state.is_pressed(Key::ScrollPrev) {
			self.try_set_cursor_size(self.cursor_size / 2)
		}

		if self.input_state.is_pressed(Key::CtrlX) {
			self.try_set_cursor_size(self.cursor_size.mul3(uvec3(2, 1, 1)))
		}

		if self.input_state.is_pressed(Key::AltX) {
			self.try_set_cursor_size(self.cursor_size.div3(uvec3(2, 1, 1)))
		}

		if self.input_state.is_pressed(Key::CtrlY) {
			self.try_set_cursor_size(self.cursor_size.mul3(uvec3(1, 2, 1)))
		}

		if self.input_state.is_pressed(Key::AltY) {
			self.try_set_cursor_size(self.cursor_size.div3(uvec3(1, 2, 1)))
		}

		if self.input_state.is_pressed(Key::CtrlZ) {
			self.try_set_cursor_size(self.cursor_size.mul3(uvec3(1, 1, 2)))
		}

		if self.input_state.is_pressed(Key::AltZ) {
			self.try_set_cursor_size(self.cursor_size.div3(uvec3(1, 1, 2)))
		}

		if self.input_state.is_pressed(Key::Grab) {
			if let Some(hitpoint) = self.crosshair_hitpoint(0.1) {
				self.set_current_paint(self.voxels().at(hitpoint))
			}
		}

		// Add/Destroy blocks
		if self.input_state.is_pressed(Key::Mouse1) {
			self.shoot_block(self.current_paint)
		}
		if self.input_state.is_pressed(Key::Mouse3) {
			self.destroy_block()
		}

		// Request lightmap baking.
		if self.input_state.is_pressed(Key::StartBake) {
			self.log("Full bake...".into());
			self.voxel_world.request_full_bake(self.camera.position.to_ivec(), &self.dir)
		}
	}

	// Now shooting blocks of this type.
	fn set_current_paint(&mut self, value: VoxelType) {
		self.current_paint = value;
		self.log(format!("painting with {}", self.current_paint.0));
	}

	// Now shooting blocks of size `cursor_size`,
	// but clamped to reasonable value.
	fn try_set_cursor_size(&mut self, cursor_size: uvec3) {
		self.cursor_size = cursor_size.map(|v| clamp(v, 1, Voxels::CELL_SIZE));
		self.log_cursor_size()
	}

	// Log the cursor size to screen and stdout.
	fn log_cursor_size(&self) {
		let size = self.aligned_cursor_range(ivec3(0, 0, 0)).size();
		self.log(format!("{}x{}x{} cursor", size.x, size.y, size.z));
	}

	// Draw the outline of a 3D cuboid that we would fill with blocks on click.
	fn draw_cursor(&self) {
		self.engine.enable_line_offset();
		if let Some(range) = self.cursor_range() {
			self.engine.draw_boundingbox(BoundingBox::new(range.min.to_f32(), range.max.to_f32()))
		}
	}

	fn draw_spawn_points(&self) {
		let model = &self.spawn_point_model;
		for sp in &self.metadata.spawn_points {
			self.engine.draw_model_at(&model, sp.position());
		}
	}

	fn draw_pickup_points(&self) {
		let model = self.models.entity_model(EKind::GiftBox { pickup_point_id: None });
		for pp in &self.metadata.pickup_points {
			self.engine.draw_model_at(&model, pp.position());
		}
	}

	fn cursor_range(&self) -> Option<Cuboid> {
		self.crosshair_hitpoint(-0.1).map(|hitpoint| self.aligned_cursor_range(hitpoint))
	}

	fn cursor_position(&self) -> Option<ivec3> {
		self.cursor_range().map(|c| c.min)
	}

	fn shoot_block(&mut self, voxel: VoxelType) {
		if let Some(hitpoint) = self.crosshair_hitpoint(-0.1) {
			self.set_range(&self.aligned_cursor_range(hitpoint), voxel)
		}
	}

	fn destroy_block(&mut self) {
		if let Some(hitpoint) = self.crosshair_hitpoint(0.1) {
			self.set_range(&self.aligned_cursor_range(hitpoint), VoxelType::EMPTY)
		}
	}

	// Affected range when or adding/removing voxels.
	// In each direction, the cursor range gets aligned down the same power of two as present in the size.
	// E.g.:
	// 	size 1 => align 1
	// 	size 2 => align 2
	// 	size 3 => align 1
	// 	size 4 => align 4
	// 	size 5 => align 5
	// 	size 6 => align 2
	// 	size 8 => align 8
	//  ...
	fn aligned_cursor_range(&self, pos: ivec3) -> Cuboid {
		let aligned_pos = pos.zip(self.cursor_size.as_ivec(), |pos, size| pos & !(2i32.pow(size.trailing_zeros()) - 1));
		Cuboid::with_size(aligned_pos, self.cursor_size)
	}

	// When looking through the crosshair, where do we intersect the scene?
	// Pass a small forward or backward `offset` to select the voxel right before or right behind the intersection.
	fn crosshair_hitpoint(&self, offset: f64) -> Option<ivec3> {
		self.voxels().intersect_voxel(&self.camera.crosshair_ray(), offset)
	}

	const MAX_TICK: Duration = Duration::from_millis(100);

	// Get the seconds elapsed since the last `update_dt` call,
	// but clamp to a maximum of `MAX_TICK_SECS`.
	//
	// Say we are running smoothly at 60 FPS, then `dt` will be consistently 16 ms.
	// Now if the main GL thread temporarily drops to, say, 20 FPS, then `dt` will compensate at 50 ms,
	// so that we might see some stutter, but on average time won't progress any slower.
	//
	// However, if the GL thread blocks for a really long time, perhaps e.g. when loading resources,
	// then we don't want `dt` to grow unbounded else our moving avatar might make a huge jump in between frames.
	// Therefore `dt` is limited to no more than 100 ms. I.e. if the GL thread temporarily drops below 10 FPS
	// (unplayable anyway), then time will slow down and we won't make large unexpected movements.
	fn update_dt(&mut self) -> Duration {
		let now = Instant::now();
		let mut dt = now - self.last_tick;
		if dt > Self::MAX_TICK {
			dt = Self::MAX_TICK
		}
		self.last_tick = now;
		dt
	}

	// log a message to screen and stdout.
	fn log(&self, msg: String) {
		println!("{}", &msg);
		*self.log_msg.borrow_mut() = msg;
	}
}

impl EventHandler for EdState {
	/// Handle keyboard input.
	fn on_key(&mut self, k: Key, pressed: bool) {
		self.input_state.record_key(k, pressed)
	}

	/// Handle mouse input.
	fn on_mouse_move(&mut self, x: f64, y: f64) {
		self.input_state.record_mouse((x * self.mouse_sens, y * self.mouse_sens))
	}

	fn tick(&mut self) {
		self.tick()
	}

	fn draw(&mut self, width: u32, height: u32) {
		self.draw(width, height)
	}
}
