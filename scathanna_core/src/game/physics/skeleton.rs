use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Skeleton {
	pub hsize: f32,
	pub vsize: f32,
	pub position: vec3, // center bottom
	pub velocity: vec3,
	pub orientation: Orientation,
}

const G: f32 = 48.0; // TODO
const STAIRCLIMB_SPEED: f32 = 15.0; // TODO

impl Skeleton {
	pub fn new(pos: vec3, orientation: Orientation, hsize: f32, vsize: f32) -> Self {
		Self {
			position: pos,
			hsize,
			vsize,
			orientation,
			velocity: default(),
		}
	}

	pub fn tick(&mut self, upd: &mut ClientMsgs, world: &World, dt: f32) {
		let v1 = self.velocity.y();
		self.tick_gravity(G, dt);
		self.tick_move(world, dt);
		let v2 = self.velocity.y();
		if v1 < -1.0 && v2 == 0.0 {
			upd.push(ClientMsg::PlaySound(SoundEffect::spatial("land", self.position, 0.3)));
		}
		self.tick_rescue(world, dt);
	}

	fn tick_move(&mut self, world: &World, dt: f32) {
		let mut xbump = false;
		let mut zbump = false;
		let delta = self.velocity * dt;
		let sub_delta = delta / 16.0;

		for _i in 0..16 {
			let dx = vec3(sub_delta.x(), 0.0, 0.0);
			let dy = vec3(0.0, sub_delta.y(), 0.0);
			let dz = vec3(0.0, 0.0, sub_delta.z());

			if self.pos_ok(world, self.position + dx) {
				self.position += dx;
			} else {
				xbump = true;
			}

			if self.pos_ok(world, self.position + dy) {
				self.position += dy;
			} else {
				self.velocity[Y] = 0.0;
			}

			if self.pos_ok(world, self.position + dz) {
				self.position += dz;
			} else {
				zbump = true;
			}
		}

		// stair climbing
		if (xbump || zbump) && self.velocity.y() >= 0.0 {
			let probe_pos = self.position + vec3(delta.x(), 2.1, delta.z()); // what if we kept moving horizontally and took one step up?
			if self.pos_ok(world, probe_pos) {
				self.position += vec3(delta.x(), 0.0, delta.z());
			} else {
				if xbump {
					self.velocity[X] = 0.0;
				}
				if zbump {
					self.velocity[Z] = 0.0;
				}
			}
		}
	}

	// rescue player if somehow stuck inside a block: move them up.
	fn tick_rescue(&mut self, world: &World, dt: f32) {
		if !self.pos_ok(world, self.position) {
			self.position[Y] += STAIRCLIMB_SPEED * dt;
		}
	}

	pub fn on_ground(&self, world: &World) -> bool {
		!self.pos_ok(world, self.position - vec3(0.0, 0.05, 0.0))
	}

	// _________________________________________________________ mutators

	pub fn try_jump(&mut self, world: &World, jump_speed: f32) -> bool {
		if self.on_ground(world) {
			self.unconditional_jump(jump_speed);
			true
		} else {
			false
		}
	}

	pub fn unconditional_jump(&mut self, jump_speed: f32) {
		self.velocity[Y] = jump_speed
	}

	pub fn try_walk(&mut self, dt: f32, world: &World, walk_speed: vec3) {
		const MAX_AIRCTL_SPEED: f32 = Player::WALK_SPEED;
		const AIRCTL_ACCEL: f32 = 2.0;

		if self.on_ground(world) {
			self.velocity[X] = walk_speed[X];
			self.velocity[Z] = walk_speed[Z];
		} else {
			// flying through the air

			// always slightly damp movement
			let damp = 0.1;
			self.velocity *= 1.0 - damp * dt;

			// allow to control movement in the air a bit.
			if self.velocity.remove(1).len() > MAX_AIRCTL_SPEED {
				// flying too fast, damp aggressively
				self.velocity *= 1.0 - 4.0 * damp * dt;
			} else {
				// flying not too fast, allow some slow control
				self.velocity += (AIRCTL_ACCEL * dt) * walk_speed;
			}
		}
	}
	pub fn set_frame(&mut self, frame: Frame) {
		self.position = frame.position;
		self.velocity = frame.velocity;
		self.orientation = frame.orientation;
	}

	//______________________________________________________________________ accessors

	pub fn frame(&self) -> Frame {
		Frame {
			position: self.position,
			velocity: self.velocity,
			orientation: self.orientation,
		}
	}

	// is this player position allowed in the map?
	// I.e. not bumping into blocks.
	pub fn pos_ok(&self, world: &World, pos: vec3) -> bool {
		!world.map.voxels.bumps(&self.bounds_for(pos))
	}

	// bounding box for a player at position `pos`.
	fn bounds_for(&self, pos: vec3) -> BoundingBox<f32> {
		let min = pos - vec3(self.hsize / 2.0, 0.0, self.hsize / 2.0);
		let max = pos + vec3(self.hsize / 2.0, self.vsize, self.hsize / 2.0);
		BoundingBox::new(min, max)
	}

	pub fn bounds(&self) -> BoundingBox<f32> {
		self.bounds_for(self.position)
	}

	fn tick_gravity(&mut self, g: f32, dt: f32) {
		self.velocity[Y] -= g * dt;
		let damp = 0.05;
		self.velocity *= 1.0 - damp * dt;
	}
}
