use super::internal::*;

/// A GLClient controls and renders a GameState.
/// (In MVP parlance it would be a "view" and "controller",
/// where the GameState is the "model").
///
/// A GlClient can be run either independently or inside a NetClient.
/// (see binary scathanna_gl, subcommands `play`, `client`)
pub struct GLClient {
	// High-level access to OpenGL functionality.
	engine: Rc<Engine>,

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
	model_pack: ModelPack,
}

pub const DBG_GEOMETRY: bool = false;
const ENABLE_BORDERS: bool = false;

impl GLClient {
	/// Construct a GLClient that renders and controls a GameState through Player `ID`.
	/// Loads the needed textures and models from local disk.
	pub fn new(engine: Rc<Engine>, world: World, player_id: ID) -> Result<Self> {
		let dir = map_directory(&world.map.name);
		let voxel_models = VoxelModels::load(&engine, &world.map.voxels, &dir, ENABLE_BORDERS)?;

		engine.set_sun_direction(world.map.metadata.sun_direction);

		Ok(Self {
			last_tick: Instant::now(),
			input_state: InputState::new(),
			mouse_sens: 0.001, // TODO

			state: ClientState::new(engine.clone(), player_id, world),
			voxel_models,
			model_pack: ModelPack::new(engine.clone())?,
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
	pub fn draw(&self, width: u32, height: u32) {
		let camera = &self.state.local_player().camera();
		self.engine.set_camera((width, height), camera);
		self.engine.clear(0.8, 0.8, 1.0);
		self.voxel_models.draw(&self.engine, camera);
		self.draw_players(camera);
		self.draw_entities(camera);
		self.draw_effects(camera);
		self.engine.draw_crosshair();
		self.state.hud().draw(&self.engine, &self.state.local_player());
	}

	fn draw_entities(&self, camera: &Camera) {
		for (_, entity) in &self.state.world().entities {
			if camera.can_see(entity.position) {
				let model = self.model_pack.entity_model(entity.kind);
				self.engine.draw_model_at(&model, entity.position);
			}
		}
	}

	fn draw_effects(&self, camera: &Camera) {
		for e in &self.state.world().effects {
			self.draw_effect(e, camera)
		}
	}

	fn draw_effect(&self, e: &Effect, camera: &Camera) {
		use EffectType::*;
		match e.typ {
			SimpleLine { start, end } => self.draw_simple_line_effect(start, end),
			LaserBeam { start, orientation, len } => self.model_pack.draw_laserbeam_effect(start, orientation, len, e.ttl),
			ParticleBeam {
				start,
				orientation,
				len,
				color_filter,
			} => self.model_pack.draw_particle_beam_effect(start, orientation, len, color_filter, e.ttl),
			ParticleExplosion { pos, color } => {
				if camera.can_see(pos) {
					self.model_pack.draw_particles_effect(pos, color, e.ttl)
				}
			}
			Respawn { pos } => {
				if camera.can_see(pos) {
					self.model_pack.draw_respawn_effect(pos, e.ttl)
				}
			}
		}
	}

	fn draw_simple_line_effect(&self, start: vec3, end: vec3) {
		self.engine.draw_line(RED, 4.0, start, end)
	}

	fn draw_players(&self, camera: &Camera) {
		for (_, player) in self.state.world().players.iter() {
			if !player.spawned {
				// don't draw players before they spawn
				continue;
			}

			if player.id == self.state.player_id() {
				let sun_intens = self.sun_intensity_at(player.center());
				self.draw_player_1st_person(player, sun_intens);
			} else {
				if camera.can_see(player.position()) {
					self.draw_player_3d_person(player)
				}
			}
		}
	}

	fn sun_intensity_at(&self, pos: vec3) -> f32 {
		let ray = DRay::new(pos.into(), self.state.world().map.metadata.sun_direction.to_f64());

		match self.state.world().map.voxels.intersects(&ray) {
			false => 1.0,
			true => 0.5,
		}
	}

	fn draw_player_1st_person(&self, player: &Player, sun_intens: f32) {
		let model = self.model_pack.player_model(player.avatar_id);
		model.draw_1st_person(&self.engine, player, sun_intens);
		if DBG_GEOMETRY {
			self.draw_line_of_fire(player);
		}
	}

	fn draw_player_3d_person(&self, player: &Player) {
		let model = self.model_pack.player_model(player.avatar_id);
		model.draw_3rd_person(&self.engine, player);
		if let Some(hat) = player.powerup {
			//self.draw_hat(player, hat)
			let hat = self.model_pack.entity_model(hat);
			model.draw_hat(&self.engine, player, hat)
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
