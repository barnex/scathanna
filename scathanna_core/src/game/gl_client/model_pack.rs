use super::internal::*;

pub struct ModelPack {
	engine: Rc<Engine>,

	player_models: PlayerModels,
	laserbeam_model: Model,
	particle_explosion_vao: VertexArray,
	particle_beam_vao: VertexArray,
	entity_models: HashMap<String, Model>,
}

impl ModelPack {
	pub fn new(engine: Rc<Engine>) -> Result<Self> {
		Ok(Self {
			laserbeam_model: Model::new(engine.wavefront_obj("laserbeam")?, Material::FlatTexture(engine.texture("laserbeam", RED))).double_sided(),
			particle_explosion_vao: particle_explosion_vao(&engine, 800),
			particle_beam_vao: particle_beam_vao(&engine),
			player_models: PlayerModels::new(&engine)?,
			entity_models: Self::load_entity_models(&engine)?,
			engine,
		})
	}

	fn load_entity_models(engine: &Engine) -> Result<HashMap<String, Model>> {
		let mut models = HashMap::default();

		for kind in EKind::ALL_EKINDS {
			let obj_file = kind.as_str();
			let model = Model::new(engine.wavefront_obj_scaled(obj_file, 3.0)?, Material::MatteTexture(engine.texture(obj_file, GREY))).double_sided();
			models.insert(obj_file.to_owned(), model);
		}
		Ok(models)
	}

	// _______________________________________________________________________________________ entities

	pub fn entity_model(&self, kind: EKind) -> &Model {
		self.entity_models.get(kind.as_str()).unwrap_or_else(|| self.entity_models.get("box").unwrap())
	}

	// _______________________________________________________________________________________ players

	pub fn player_model(&self, avatar_id: u8) -> &PlayerModel {
		self.player_models.get(avatar_id)
	}

	// _______________________________________________________________________________________ effects

	pub fn draw_particles_effect(&self, pos: vec3, color: vec3, ttl: f32) {
		draw_particle_explosion(&self.engine, &self.particle_explosion_vao, pos, color, ttl)
	}

	pub fn draw_particle_beam_effect(&self, start: vec3, orientation: Orientation, len: f32, color_filter: vec3, ttl: f32) {
		draw_particle_beam_effect(&self.engine, &self.particle_beam_vao, start, orientation, len, color_filter, ttl)
	}

	pub fn draw_respawn_effect(&self, pos: vec3, ttl: f32) {
		let time = RESPAWN_TTL - ttl;
		let gravity = -30.0;

		let location_mat = translation_matrix(pos);

		self.engine.set_cull_face(false);
		let color = YELLOW;
		let alpha = 0.5;
		self.engine.shaders().use_particles(color, alpha, gravity, time, &location_mat);
		self.engine.draw_triangles(&self.particle_explosion_vao);
	}

	pub fn draw_laserbeam_effect(&self, start: vec3, orientation: Orientation, len: f32, _ttl: f32) {
		let pitch_mat = pitch_matrix(-90.0 * DEG - orientation.pitch);
		let yaw_mat = yaw_matrix(-orientation.yaw);
		let width = 2.0;
		let scale_mat = mat4::from([
			[width, 0.0, 0.0, 0.0], //
			[0.0, len, 0.0, 0.0],
			[0.0, 0.0, width, 0.0],
			[0.0, 0.0, 0.0, 1.0],
		]);
		let location_mat = translation_matrix(start);

		let mat = location_mat * yaw_mat * pitch_mat * scale_mat;
		self.engine.draw_model_with(&self.laserbeam_model, &mat);
	}
}
