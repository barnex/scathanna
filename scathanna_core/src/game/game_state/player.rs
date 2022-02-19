use super::internal::*;
use ClientMsg::*;

/// Player data.
/// Part of the GameState.
/// Can be sent over the wire to communicate updates (e.g. position moved).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
	// controlled by server:
	pub id: ID,        // uniquely identifies player across server and all clients
	pub name: String,  // nickname
	pub avatar_id: u8, // determines which avatar model is drawn (gl_client.rs).
	pub team: Team,
	pub health: i32,
	pub spawned: bool, // playing or waiting for respawn?
	pub next_spawn_point: vec3,
	pub powerup: Option<EKind>,
	pub invulnerability_ttl: Option<f32>, // seconds of invulnerability left

	// controlled locally, synced to server:
	pub skeleton: Skeleton, // fully determines player position

	// controlled locally, not synced:
	pub local: LocalState,
}

/// Controlled by the local client, never overwritten by the server.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LocalState {
	pub gun_cooldown: f32, // Time until next shot allowed.
	pub feet_phase: f32,   // used for avatar animation (gl_client.rs).
	pub feet_pitch: f32,
}

/// Key for identifying players
pub type ID = u32;

const FEET_ANIM_SPEED: f32 = 12.0;
const FEET_ANIM_DAMP: f32 = 6.0;
const DEFAULT_GUN_COOLDOWN: f32 = 0.7; // seconds

impl Player {
	pub const HSIZE: f32 = 3.8;
	pub const VSIZE: f32 = 5.8;
	pub const CAM_HEIGHT: f32 = 5.4;
	pub const WALK_SPEED: f32 = 24.0;
	const JUMP_SPEED: f32 = 24.0;

	pub fn new(id: ID, position: vec3, orientation: Orientation, name: String, avatar_id: u8, team: Team) -> Self {
		Self {
			id,
			name,
			avatar_id,
			spawned: false,
			health: 100,
			next_spawn_point: position,
			powerup: None,
			team,
			invulnerability_ttl: None,

			skeleton: Skeleton::new(position, orientation, Self::HSIZE, Self::VSIZE),
			local: default(),
		}
	}

	// __________________________________________________________________________________ control

	/// Record a diff for controlling this player with keyboard/mouse input.
	/// Called on a clone of the World's player (!so need to be careful for self-interaction!).
	pub fn control(&mut self, upd: &mut ClientMsgs, input_state: &InputState, world: &World, dt: f32) {
		if self.spawned {
			self.tick_movement(upd, input_state, world, dt);
			self.control_shooting(upd, input_state, world, dt);
		} else {
			self.set_orientation(input_state);
			if input_state.is_pressed(Key::Mouse1) {
				self.skeleton.position = self.next_spawn_point;
				upd.push(ReadyToSpawn);
			}
		}
		upd.push(MovePlayer(self.skeleton.frame()));
	}

	// __________________ shooting

	fn control_shooting(&mut self, upd: &mut ClientMsgs, input_state: &InputState, world: &World, dt: f32) {
		// not allowed to shoot if gun is still cooling down.
		self.local.gun_cooldown -= dt;
		if self.local.gun_cooldown > 0.0 {
			return;
		}

		if input_state.is_pressed(Key::Mouse1) {
			self.shoot(upd, world, dt)
		} else if input_state.is_down(Key::Mouse1) && self.can_shoot_berserk(world) {
			self.shoot(upd, world, dt)
		}
	}

	fn can_shoot_berserk(&self, world: &World) -> bool {
		use EKind::*;
		match self.powerup {
			Some(BerserkerHelmet) => true,
			Some(PartyHat) => self.is_on_lava(world),
			_ => false,
		}
	}

	pub fn is_on_lava(&self, world: &World) -> bool {
		let probe = self.position() - 0.2 * vec3::EY;
		world.map.voxels.at(probe.floor()) == VoxelType::LAVA
	}

	fn gun_cooldown(&self, world: &World) -> f32 {
		use EKind::*;
		const FAST: f32 = 0.05;
		match self.powerup {
			Some(CowboyHat) => FAST,
			Some(BerserkerHelmet) => FAST,
			Some(PartyHat) => {
				if self.is_on_lava(world) {
					FAST
				} else {
					DEFAULT_GUN_COOLDOWN
				}
			}
			_ => DEFAULT_GUN_COOLDOWN,
		}
	}

	fn shoot(&mut self, upd: &mut ClientMsgs, world: &World, _dt: f32) {
		// shooting, so gun will need to cool down before next shot is allowed.
		self.local.gun_cooldown = self.gun_cooldown(world);

		let line_of_fire = self.line_of_fire(world);
		let start = line_of_fire.start.to_f32();
		let end = self.shoot_at(world);
		let len = (end - start).len();
		upd.push(ClientMsg::AddEffect(Effect::particle_beam(start, self.orientation(), len, self.team.color_filter())));
		upd.push(ClientMsg::PlaySound(SoundEffect::spatial(pick_random(&["bang1", "bang2", "bang3", "bang4"]), self.position(), 30.0)));
		upd.push(ClientMsg::PlaySound(SoundEffect::spatial(pick_random(&["ricochet1", "ricochet2", "ricochet3", "ricochet4"]), end, 1.0)));

		if let Some((_, Some(victim_id))) = world.intersect_except(self.id, &line_of_fire) {
			upd.push(HitPlayer(victim_id));
		}

		// effect when shooting lava
		if world.map.voxels.at_pos(end) == VoxelType::LAVA {
			upd.push(AddEffect(Effect::particle_explosion(end, RED)));
			upd.push(PlaySound(SoundEffect::spatial("lava", end, 1.0)))
		}
	}

	// __________________ movement

	fn tick_movement(&mut self, upd: &mut ClientMsgs, input_state: &InputState, world: &World, dt: f32) {
		self.set_orientation(input_state);
		self.tick_walk(upd, input_state, world, dt);
		self.tick_jump(upd, input_state, world, dt);
		self.skeleton.tick(upd, world, dt);
	}

	fn tick_walk(&mut self, _upd: &mut ClientMsgs, input_state: &InputState, world: &World, dt: f32) {
		let walk_speed = Self::WALK_SPEED * walk_dir(self.orientation().yaw, input_state);
		self.skeleton.try_walk(dt, world, walk_speed);
	}

	fn tick_jump(&mut self, upd: &mut ClientMsgs, input_state: &InputState, world: &World, _dt: f32) {
		if input_state.is_down(Key::Jump) {
			if self.skeleton.try_jump(world, Self::JUMP_SPEED) {
				upd.push(ClientMsg::PlaySound(SoundEffect::spatial("jump", self.position(), 0.3)))
			}
		}

		if self.powerup == Some(EKind::XMasHat) && input_state.is_pressed(Key::Jump) {
			self.skeleton.unconditional_jump(Self::JUMP_SPEED);
			upd.push(ClientMsg::PlaySound(SoundEffect::spatial("fly", self.position(), 0.3)))
		}
	}

	pub fn animate_feet(&mut self, dt: f32) {
		let walk_speed = self.skeleton.velocity;
		let vspeed = self.skeleton.velocity.y();
		if walk_speed != vec3::ZERO {
			// move feet in semicircles while moving
			self.local.feet_phase += dt * FEET_ANIM_SPEED;

			if self.local.feet_phase > PI {
				self.local.feet_phase = -PI;
			}
		} else {
			// gradually relax feet to resting position while still
			self.local.feet_phase *= 1.0 - (FEET_ANIM_DAMP * dt);
			self.local.feet_phase = clamp(self.local.feet_phase, -PI, PI);
		}

		let target_pitch = if vspeed > 0.0 {
			-30.0 * DEG
		} else if vspeed < 0.0 {
			30.0 * DEG
		} else {
			0.0
		};
		let damp = FEET_ANIM_DAMP * dt;
		self.local.feet_pitch = (1.0 - damp) * self.local.feet_pitch + damp * target_pitch;
	}

	// TODO: make private once diff/apply is used throughout.
	pub fn set_position(&mut self, position: vec3) {
		self.skeleton.position = position
	}

	fn set_orientation(&mut self, input_state: &InputState) {
		self.skeleton.orientation = Orientation {
			yaw: input_state.mouse_yaw(),
			pitch: input_state.mouse_pitch(),
		}
	}

	// __________________________________________________________________________________ accessors

	/// Center-bottom position of the bounding box.
	pub fn position(&self) -> vec3 {
		self.skeleton.position
	}

	pub fn center(&self) -> vec3 {
		self.skeleton.bounds().center()
	}

	pub fn orientation(&self) -> Orientation {
		self.skeleton.orientation
	}

	pub fn camera(&self) -> Camera {
		Camera {
			position: self.position() + vec3(0.0, Self::CAM_HEIGHT, 0.0),
			orientation: self.orientation(),
		}
	}

	/// Ray looking through the player's crosshair.
	pub fn line_of_sight(&self) -> Ray64 {
		Ray64::new(self.camera().position.into(), self.orientation().look_dir().into())
	}

	/// Ray from the player's gun nozzle to where the player is looking.
	/// I.e., the trajectory a bullet would follow.
	pub fn line_of_fire(&self, world: &World) -> Ray64 {
		let start = self.gun_center();
		let look_at = self.look_at(world);
		let shoot_from_gun = Ray64::new(start.into(), (look_at - start).normalized().into());

		// Because of parallax between the nozzle and camera position,
		// an object can sometimes be in front of the gun but not in front of the camera.
		// This can lead to seemingly inexplicably missed shots.
		//
		// Many games have this behavior when shooting from the hip.
		// However, here it is particularly severe as the gun is quite far from the camera.
		// Therefore, when an object blocks the line of fire but not the line of sight,
		// shoot from the "eye" rather than from the hip so that the shot is not missed.
		let shoot_at = world.intersect_except(self.id, &shoot_from_gun).map(|(t, _)| shoot_from_gun.at(t).to_f32());
		if let Some(shoot_at) = shoot_at {
			if (shoot_at - look_at).len() > 2.0 {
				return self.line_of_sight();
			}
		}

		shoot_from_gun
	}

	/// (Absolute) position of the player's gun.
	/// Note: this is not necessarily the position where a bullet starts,
	/// use `line_of_fire().start` for that.
	fn gun_center(&self) -> vec3 {
		let gun_internal = self.gun_pos_internal();
		self.position() + self.skeleton.orientation.look_right() * gun_internal.x() + gun_internal.y() * vec3::EY
	}

	/// Position the user is looking at.
	/// If looking at the (infinitely far) sky,
	/// this returns a far-away point in the looking direction.
	pub fn look_at(&self, world: &World) -> vec3 {
		let line_of_sight = self.line_of_sight();
		world
			.intersect_except(self.id, &line_of_sight)
			.map(|(t, _)| line_of_sight.at(t))
			.unwrap_or(self.camera().position.to_f64() + 10000.0 * self.orientation().look_dir().to_f64())
			.into()
	}

	/// (Absolute) position where the player's gun would hit if it fired.
	pub fn shoot_at(&self, world: &World) -> vec3 {
		let line_of_fire = self.line_of_fire(&world);
		line_of_fire.at(world.intersect_except(self.id, &line_of_fire).map(|(t, _)| t + 0.01).unwrap_or(10000.0)).into()
	}

	pub fn gun_pos_internal(&self) -> vec3 {
		vec3(0.5 * self.skeleton.hsize + 0.4, 0.66 * self.skeleton.vsize, 0.0)
	}

	/// Intersect ray with player hitbox.
	pub fn intersect(&self, ray: &Ray64) -> Option<f64> {
		// Cannot get hit if not spawned.
		match self.spawned {
			true => self.skeleton.bounds().convert::<f64>().intersect(ray),
			false => None,
		}
	}
}
