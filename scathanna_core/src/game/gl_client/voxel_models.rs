use super::internal::*;

pub struct VoxelModels {
	cell_models: HashMap<ivec3, Vec<Model>>,
}

impl VoxelModels {
	pub fn load(engine: &Engine, voxels: &Voxels, dir: &Path, enable_borders: bool) -> Result<Self> {
		let mut cell_models = HashMap::default();

		for cell_pos in voxels.iter_cell_positions() {
			if let Some(cell_buffer) = load_cell_buffer(voxels, cell_pos, dir) {
				let models = VoxelWorld::upload_cell_models(engine, cell_buffer);
				cell_models.insert(cell_pos, models);
			}
		}

		if enable_borders {
			let edges = highlighted_edges(engine, voxels);
			for (cell_pos, edge_model) in edges {
				cell_models.entry(cell_pos).or_default().push(edge_model)
			}
		}

		Ok(Self { cell_models })
	}

	pub fn draw(&self, engine: &Engine, camera: &Camera) {
		engine.set_line_width(2.0);
		let visible_range = VoxelWorld::visible_range(camera);
		for pos in visible_range.iter_by(Voxels::CELL_SIZE) {
			if let Some(models) = self.cell_models.get(&pos) {
				if can_see_cell(camera, pos) {
					for model in models {
						engine.draw_model(model);
					}
				}
			}
		}
	}
}

fn can_see_cell(camera: &Camera, cell_pos: ivec3) -> bool {
	const CS: i32 = Voxels::CELL_SIZE as i32;
	for dx in [0, CS] {
		for dy in [0, CS] {
			for dz in [0, CS] {
				let pos = cell_pos + ivec3(dx, dy, dz);
				if camera.can_see(pos.to_f32()) {
					return true;
				}
			}
		}
	}
	false
}
