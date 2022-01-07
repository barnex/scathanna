use super::internal::*;

pub struct PlayerModels {
	models: [PlayerModel; 7],
}

const HEAD_PITCH_FACTOR: f32 = 0.25;

impl PlayerModels {
	// TODO: lazy load?
	pub fn new(engine: &Engine) -> Result<Self> {
		Ok(Self {
			models: [
				PlayerModel::frog(engine)?, //
				PlayerModel::panda(engine)?,
				PlayerModel::turkey(engine)?,
				PlayerModel::pig(engine)?,
				PlayerModel::hamster(engine)?,
				PlayerModel::chicken(engine)?,
				PlayerModel::bunny(engine)?,
			],
		})
	}

	pub fn get(&self, avatar_id: u8) -> &PlayerModel {
		// avatar_id gets checked higher up so should be valid.
		// But just in case, return a default if invalid nevertheless.
		self.models.get(avatar_id as usize).unwrap_or(&self.models[0])
	}
}

pub fn parse_avatar_id(s: &str) -> Result<u8> {
	let opts = ["frog", "panda", "turkey", "pig", "hamster", "chicken", "bunny"];
	match s.parse() {
		Ok(v) => Ok(v),
		Err(_) => opts //
			.iter()
			.position(|&v| v == s)
			.map(|v| v as u8)
			.ok_or(error(format!("avatar options: {}", opts.join(",")))),
	}
}

/// Models (on the GPU) needed to draw player avatars.
pub struct PlayerModel {
	head: Model,
	foot: Model,
	gun: Model,
	head_height: f32,
	head_scale: f32,
	foot_scale: f32,
	foot_sep: f32,
}

fn gun(engine: &Engine) -> Result<Model> {
	Ok(Model::new(engine.wavefront_obj("bubblegun")?, Material::MatteTexture(engine.texture("party_hat", WHITE))))
}

impl PlayerModel {
	pub fn frog(engine: &Engine) -> Result<Self> {
		let texture = Material::MatteTexture(engine.texture("frog", GREEN));
		Ok(Self {
			head: Model::new(engine.wavefront_obj("froghead")?, texture.clone()),
			foot: Model::new(engine.wavefront_obj("frogfoot")?, texture.clone()),
			gun: gun(engine)?,
			head_height: 2.0,
			head_scale: 4.0,
			foot_scale: 2.5,
			foot_sep: 0.15,
		})
	}

	pub fn panda(engine: &Engine) -> Result<Self> {
		let texture = Material::MatteTexture(engine.texture("panda", WHITE));
		Ok(Self {
			head: Model::new(engine.wavefront_obj("pandahead")?, texture.clone()),
			foot: Model::new(engine.wavefront_obj("frogfoot")?, texture.clone()),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

	pub fn pig(engine: &Engine) -> Result<Self> {
		let texture = Material::MatteTexture(engine.texture("pig", WHITE));
		Ok(Self {
			head: Model::new(engine.wavefront_obj("pighead")?, texture.clone()),
			foot: Model::new(engine.wavefront_obj("simple_foot")?, texture.clone()),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

	pub fn turkey(engine: &Engine) -> Result<Self> {
		let texture = Material::MatteTexture(engine.texture("turkey", WHITE));
		Ok(Self {
			head: Model::new(engine.wavefront_obj("turkeyhead")?, texture.clone()),
			foot: Model::new(engine.wavefront_obj("chickenleg")?, texture.clone()),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 3.0,
			foot_sep: 0.05,
		})
	}

	pub fn hamster(engine: &Engine) -> Result<Self> {
		let texture = Material::MatteTexture(engine.texture("hamster", WHITE));
		Ok(Self {
			head: Model::new(engine.wavefront_obj("hamsterhead")?, texture.clone()),
			foot: Model::new(engine.wavefront_obj("simple_foot")?, texture.clone()),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

	pub fn chicken(engine: &Engine) -> Result<Self> {
		let texture = Material::MatteTexture(engine.texture("chicken", WHITE));
		Ok(Self {
			head: Model::new(engine.wavefront_obj("chickenhead")?, texture.clone()),
			foot: Model::new(engine.wavefront_obj("chickenleg")?, texture.clone()),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 2.8,
			foot_sep: 0.05,
		})
	}

	pub fn bunny(engine: &Engine) -> Result<Self> {
		let texture = Material::MatteTexture(engine.texture("bunny", WHITE));

		Ok(Self {
			head: Model::new(engine.wavefront_obj("bunnyhead")?, texture.clone()),
			foot: Model::new(engine.wavefront_obj("simple_foot")?, texture.clone()),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

	/// Draw player model as seen by others.
	pub fn draw_3rd_person(&self, engine: &Engine, player: &Player) {
		self.draw_head(engine, player);
		self.draw_feet(engine, player);
		self.draw_gun(engine, player);

		if DBG_GEOMETRY {
			engine.draw_boundingbox(player.skeleton.bounds());
		}
	}

	/// Draw player model as seen by self.
	pub fn draw_1st_person(&self, engine: &Engine, player: &Player) {
		self.draw_feet(engine, player);
		self.draw_gun(engine, player);
	}

	fn draw_gun(&self, engine: &Engine, player: &Player) {
		let scale_mat = scale_matrix(4.5);
		let Orientation { yaw, pitch } = player.orientation();
		let pitch_mat = pitch_matrix(-pitch);
		let hand_mat = translation_matrix(player.gun_pos_internal());
		let yaw_mat = yaw_matrix(-yaw);
		let pos_mat = translation_matrix(player.position());

		let transf = &pos_mat * &yaw_mat * &hand_mat * &pitch_mat * &scale_mat;

		engine.draw_model_with(&self.gun, &transf);
	}

	fn draw_head(&self, engine: &Engine, player: &Player) {
		let Orientation { yaw, pitch } = player.orientation();
		let head_pos = self.head_height * vec3::EY;
		let transf = translation_matrix(player.position() + head_pos) * yaw_matrix(-yaw) * pitch_matrix(-pitch * HEAD_PITCH_FACTOR) * scale_matrix(self.head_scale);
		engine.draw_model_with(&self.head, &transf);
	}

	pub fn draw_hat(&self, engine: &Engine, player: &Player, hat: &Model) {
		let Orientation { yaw, pitch } = player.orientation();
		let pitch_mat = pitch_matrix(-pitch * HEAD_PITCH_FACTOR);
		let top_mat = translation_matrix((self.head_height + 0.75 * self.head_scale) * vec3::EY);

		let yaw_mat = yaw_matrix(-yaw);
		let pos_mat = translation_matrix(player.position());
		let transf = &pos_mat * &yaw_mat * &pitch_mat * &top_mat;
		engine.draw_model_with(hat, &transf);
	}

	fn draw_feet(&self, engine: &Engine, player: &Player) {
		let scale_mat = scale_matrix(self.foot_scale);
		let pitch_mat = pitch_matrix(player.local.feet_pitch);
		let [left_mat, right_mat] = self.feet_pos_internal(player).map(translation_matrix);
		let yaw_mat = yaw_matrix(-player.orientation().yaw);
		let pos_mat = translation_matrix(player.position());

		let transf_l = &pos_mat * &yaw_mat * &left_mat * &pitch_mat * &scale_mat;
		let transf_r = &pos_mat * &yaw_mat * &right_mat * &pitch_mat * &scale_mat;

		engine.draw_model_with(&self.foot, &transf_l);
		engine.draw_model_with(&self.foot, &transf_r);
	}

	fn feet_pos_internal(&self, player: &Player) -> [vec3; 2] {
		let anim_r = 1.0;
		let c = anim_r * player.local.feet_phase.cos();
		let s = anim_r * player.local.feet_phase.sin();
		[
			vec3(-0.35 * player.skeleton.hsize, f32::max(0.0, s), c) - self.foot_sep * vec3::EX,
			vec3(0.35 * player.skeleton.hsize, f32::max(0.0, -s), -c) + self.foot_sep * vec3::EX,
		]
	}
}
