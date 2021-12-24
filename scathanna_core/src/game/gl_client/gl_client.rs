use super::internal::*;

/// A GLClient controls and renders a GameState.
/// (In MVP parlance it would be a "view" and "controller",
/// where the GameState is the "model").
///
/// A GlClient can be run either independently or inside a NetClient.
/// (see binary scathanna_gl, subcommands `play`, `client`)
pub struct GLClient {
	// High-level access to OpenGL functionality.
	engine: Engine,

	// Timekeeping. TODO: does this belong in GameState?
	last_tick: Instant,

	// Input
	input_state: InputState,
	mouse_sens: f64,

	// The world rendered by this client,
	// as seen and controlled by player `ID`.
	state: ClientState,

	// GPU state needed to render the world.
	//   - changes on every map reload:
	voxel_models: VoxelModels,
	//   - immutable forever:
	player_models: PlayerModels,
	laserbeam_model: Model,
	particle_explosion_vao: VertexArray,
	particle_beam_vao: VertexArray,
}

pub const DBG_GEOMETRY: bool = false;
const ENABLE_BORDERS: bool = false;

impl GLClient {
	/// Construct a GLClient that renders and controls a GameState through Player `ID`.
	/// Loads the needed textures and models from local disk.
	pub fn new(world: World, player_id: ID) -> Result<Self> {
		let mut engine = Engine::new();

		let dir = map_directory(&world.map.name);
		let voxel_models = VoxelModels::load(&mut engine, &world.map.voxels, &dir, ENABLE_BORDERS)?;
		let laserbeam_model = Model::new(engine.wavefront_obj("laserbeam")?, Material::FlatTexture(engine.texture("laserbeam", RED))).double_sided();
		let particle_explosion_vao = particle_explosion_vao(&mut engine, 800);
		let particle_beam_vao = particle_beam_vao(&mut engine);
		let player_models = PlayerModels::new(&mut engine)?;

		Ok(Self {
			last_tick: Instant::now(),
			input_state: InputState::new(),
			mouse_sens: 0.001, // TODO

			state: ClientState::new(player_id, world),
			player_models,
			laserbeam_model,
			particle_explosion_vao,
			particle_beam_vao,
			voxel_models,

			engine,
		})
	}

	// Access to the underlying GameState,
	// needed by NetClient to read/modify the GameState.
	pub fn state(&self) -> &ClientState {
		&self.state
	}

	// Access to the underlying GameState,
	// needed by NetClient to read/modify the GameState.
	pub fn state_mut(&mut self) -> &mut ClientState {
		&mut self.state
	}

	// __________________________________________________________________________________ rendering

	/// Handle draw request.
	pub fn draw(&mut self, width: u32, height: u32) {
		let camera = &self.state.local_player().camera();
		self.engine.set_camera((width, height), camera);
		self.engine.clear(0.8, 0.8, 1.0);
		self.voxel_models.draw(&mut self.engine, camera);
		self.draw_players();
		self.draw_effects();
		self.engine.draw_crosshair();
		draw_hud(&self.engine, &self.state.local_player(), &self.state.hud());
	}

	fn draw_effects(&self) {
		for e in &self.state.world().effects {
			self.draw_effect(e)
		}
	}

	fn draw_effect(&self, e: &Effect) {
		use EffectType::*;
		match e.typ {
			SimpleLine { start, end } => self.draw_simple_line_effect(start, end),
			LaserBeam { start, orientation, len } => self.draw_laserbeam_effect(start, orientation, len, e.ttl),
			ParticleExplosion { pos, color } => self.draw_particles_effect(pos, color, e.ttl),
			ParticleBeam { start, orientation, len } => self.draw_particle_beam_effect(start, orientation, len, e.ttl),
			Respawn { pos } => self.draw_respawn_effect(pos, e.ttl),
		}
	}

	fn draw_simple_line_effect(&self, start: vec3, end: vec3) {
		self.engine.draw_line(RED, 4.0, start, end)
	}

	fn draw_respawn_effect(&self, pos: vec3, ttl: f32) {
		let time = RESPAWN_TTL - ttl;
		let gravity = -30.0;

		let location_mat = translation_matrix(pos);

		self.engine.set_cull_face(false);
		let color = YELLOW;
		let alpha = 0.5;
		self.engine.shaders().use_particles(color, alpha, gravity, time, &location_mat);
		self.engine.draw_triangles(&self.particle_explosion_vao);
	}

	fn draw_laserbeam_effect(&self, start: vec3, orientation: Orientation, len: f32, _ttl: f32) {
		//let t = (LASERBEAM_TTL - ttl) / LASERBEAM_TTL;
		//let roll_mat = yaw_matrix(8.0 * t);
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

	fn draw_particles_effect(&self, pos: vec3, color: vec3, ttl: f32) {
		draw_particle_explosion(&self.engine, &self.particle_explosion_vao, pos, color, ttl)
	}

	fn draw_particle_beam_effect(&self, start: vec3, orientation: Orientation, len: f32, ttl: f32) {
		draw_particle_beam_effect(&self.engine, &self.particle_beam_vao, start, orientation, len, ttl)
	}

	fn draw_players(&self) {
		let camera = self.state.local_player().camera();

		for (_, player) in &self.state.world().players {
			if player.spawned {
				// don't draw players before they spawn

				let model = self.player_models.get(player.avatar_id);
				if player.id == self.state.player_id() {
					model.draw_1st_person(&self.engine, player);
					if DBG_GEOMETRY {
						self.draw_line_of_fire(player);
					}
				} else {
					if camera.can_see(player.position()) {
						model.draw_3rd_person(&self.engine, player)
					}
				}
			}
		}
	}

	fn draw_line_of_fire(&self, player: &Player) {
		let line_of_fire = player.line_of_fire(self.state.world());
		let start = line_of_fire.start.to_f32();
		let end = player.shoot_at(self.state.world());
		self.engine.draw_line(WHITE, 1.0, start, end);
	}

	// __________________________________________________________________________________ controlling
	// TODO: should this be moved to server?

	/// Update state and return the diff with the previous state.
	#[must_use]
	pub fn tick_and_diff(&mut self) -> ClientMsgs {
		let dt = self.update_dt();
		let upd = self.state.tick(&self.input_state, dt.as_secs_f32());
		self.input_state.clear(); // must be last
		upd
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

	/// Handle keyboard input.
	pub fn on_key(&mut self, k: Key, pressed: bool) {
		self.input_state.record_key(k, pressed)
	}

	/// Handle mouse input.
	pub fn on_mouse_move(&mut self, x: f64, y: f64) {
		self.input_state.record_mouse((x * self.mouse_sens, y * self.mouse_sens))
	}
}
