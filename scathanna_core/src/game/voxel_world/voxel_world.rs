use super::internal::*;

/// Stores and draws voxels.
/// Drawing uses async baked lightmaps.
pub struct VoxelWorld {
	voxels: Voxels,
	models: HashMap<ivec3, Vec<Model>>, //map position (aligned to CELL_SIZE) to corresponding VAO
	status: HashMap<ivec3, Status>,
	bakery: Bakery,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum Status {
	DirtyVAO,
	DirtyLightmap,
	VAOInflight,
	LightmapInflight,
	Clean,
}
use Status::*;

impl Default for Status {
	fn default() -> Self {
		DirtyVAO
	}
}

impl VoxelWorld {
	pub fn new(voxels: Voxels) -> Self {
		Self {
			bakery: Bakery::new(&voxels),
			voxels,
			models: default(),
			status: default(),
		}
	}

	pub fn load(engine: &Engine, dir: &Path) -> Result<Self> {
		let voxel_file = dir.join(MapData::VOXEL_FILE);
		let voxels = Voxels::load(voxel_file)?;

		let mut models = HashMap::default();
		let mut status = HashMap::default();
		for cell_pos in voxels.iter_cell_positions() {
			if let Some(cell_buffer) = load_cell_buffer(&voxels, cell_pos, dir) {
				let model = Self::upload_cell_models(engine, cell_buffer);
				models.insert(cell_pos, model);
				status.insert(cell_pos, Clean);
			}
		}

		Ok(Self {
			bakery: Bakery::new(&voxels),
			voxels,
			models,
			status,
		})
	}

	pub fn save(&self, dir: &Path) -> Result<()> {
		let voxel_file = dir.join(MapData::VOXEL_FILE);
		self.voxels().serialize(create(&voxel_file)?)
	}

	pub fn voxels(&self) -> &Voxels {
		&self.voxels
	}

	pub fn set_range(&mut self, range: &Cuboid, value: VoxelType) {
		// TODO: only invalidate if set to different voxel :)
		self.voxels.set_range(range, value);

		// grow range by 1 voxel so that immediately adjacent cells are invalidated too.
		self.mark_dirty_vao(&Cuboid::new(range.min - ivec3(1, 1, 1), range.max + ivec3(1, 1, 1)));
		self.mark_dirty_lightmap(&Cuboid::new(range.min - ivec3(1, 1, 1) * Voxels::ICELL_SIZE, range.max + ivec3(1, 1, 1) * Voxels::ICELL_SIZE));
	}

	pub fn lightmap_file(dir: &Path, cell_pos: ivec3) -> PathBuf {
		let hex = |i| i as u16;
		dir.join(format!("lm_{:04x}_{:04x}_{:04x}.png", hex(cell_pos.x), hex(cell_pos.y), hex(cell_pos.z)))
	}

	fn mark_dirty_vao(&mut self, range: &Cuboid) {
		println!("VoxelStore: mark_dirty_vao {}", range);
		for pos in Voxels::aligned_hull(&range).iter_by(Voxels::CELL_SIZE) {
			if self.voxels.cell_at(pos).is_some() {
				//println!("VoxelStore: mark_dirty_lightmap_vao: mark {} {:?} -> DirtyVAO", pos, self.status(pos));
				self.status.insert(pos, DirtyVAO);
			}
		}
	}

	// Mark range as DirtyLightmap if not DirtyVAO (which takes precedence).
	fn mark_dirty_lightmap(&mut self, range: &Cuboid) {
		println!("VoxelStore: mark_dirty_lightmap {}", range);
		for pos in Voxels::aligned_hull(&range).iter_by(Voxels::CELL_SIZE) {
			if self.voxels.cell_at(pos).is_some() && self.status(pos) != DirtyVAO {
				println!("VoxelStore: mark_dirty_lightmap: mark {} {:?} -> DirtyLightmap", pos, self.status(pos));
				self.status.insert(pos, DirtyLightmap);
			}
		}
	}

	/// Draw voxel world as seen from camera.
	pub fn draw(&mut self, engine: &Engine, camera: &Camera) {
		let visible_range = Self::visible_range(camera); // TODO

		self.receive_models(engine);

		self.request_models_if_dirty(&visible_range);
		self.draw_current_models(engine, &visible_range);
	}

	/// TODO
	pub fn request_full_bake(&mut self, center: ivec3, dir: &Path) {
		self.bakery.request_full_bake(center, dir)
	}

	/// Receive freshly baked models, if any.
	fn receive_models(&mut self, engine: &Engine) {
		while let Some(vec) = self.bakery.try_recv() {
			//println!("VoxelWorld: recv {} from bakery", pos);
			for (pos, buffer) in vec {
				self.models.insert(pos, Self::upload_cell_models(engine, buffer));
				self.status.insert(pos, Clean);
			}
		}
	}

	pub fn upload_cell_models(engine: &Engine, cell_buffer: CellBuffer) -> Vec<Model> {
		let lm_tex = engine.lightmap_from_mem(cell_buffer.lightmap.image());

		// convert faces to models on the GPU.
		cell_buffer
			.models
			.into_iter()
			.map(|(v, meshbuffer)| {
				Model::new(
					engine.build_vao(&meshbuffer),
					Material::Lightmap {
						texture: voxel_texture(engine, v),
						lightmap: lm_tex.clone(),
					},
				)
			})
			.collect()
	}

	/// Send out dirty cells for async re-baking.
	fn request_models_if_dirty(&mut self, visible_range: &Cuboid) {
		for pos in visible_range.iter_by(Voxels::CELL_SIZE) {
			// TODO: empty cells should not be marked dirty.
			if let Some(cell) = self.voxels.cell_at(pos) {
				match self.status(pos) {
					DirtyVAO => {
						//println!("VoxelStore: request_models: Some(cell) {} is Dirty", pos);
						//println!("VoxelStore: request_models: updating Arc Node {} in bakery", pos);
						self.bakery.update_cell(pos, cell);
						println!("VoxelStore: request_models: request_vao {} from bakery", pos);
						self.bakery.request_vao(pos);
						//println!("VoxelStore: request_models: mark {} {:?} -> Inflight", pos, self.status(pos));
						self.status.insert(pos, VAOInflight);
					}
					DirtyLightmap => {
						println!("VoxelStore: request_models: request_lightmap {} from bakery", pos);
						self.bakery.request_lightmap(pos);
						self.status.insert(pos, LightmapInflight);
					}
					_ => (),
				}
			}
		}
	}

	/// Draw using the models that we currently have, possibly outdated.
	fn draw_current_models(&self, engine: &Engine, visible_range: &Cuboid) {
		engine.set_line_width(1.5);
		for pos in visible_range.iter_by(Voxels::CELL_SIZE) {
			if let Some(models) = self.models.get(&pos) {
				for model in models {
					engine.draw_model(model);
				}
			}
		}
	}

	fn status(&self, cell_pos: ivec3) -> Status {
		*self.status.get(&cell_pos).unwrap_or(&DirtyVAO)
	}

	pub fn visible_range(camera: &Camera) -> Cuboid {
		let cam_ipos = camera.position.map(|v| v as i32); // TODO: * pitch
		let r = 512; // TODO
		let min = (cam_ipos - ivec3(r, r, r)).map(Voxels::align_down);
		let max = (cam_ipos + ivec3(r, r, r)).map(Voxels::align_up);
		Cuboid::new(min, max)
	}
}

/// Texture for a voxel type (sand, lava, ...)
fn voxel_texture(engine: &Engine, voxel: VoxelType) -> Rc<Texture> {
	match voxel.0 {
		0 => panic!("requesting texture for empty voxel"),
		1 => engine.texture("snow", WHITE),             //
		2 => engine.texture("lava", RED),               //
		3 => engine.texture("plasma", GREEN),           //
		4 => engine.texture("whitestone2", LIGHT_GREY), //
		5 => engine.texture("greystone", DARK_GREY),    //
		6 => engine.texture("sand", YELLOW),            //
		7 => engine.texture("whiterstone", GREY),       //
		8 => engine.texture("stars", MAGENTA),          //
		9 => engine.texture("sponge", GREY),            //
		10 => engine.texture("light", YELLOW),          //
		_ => engine.texture("xy", GREY),
	}
}
