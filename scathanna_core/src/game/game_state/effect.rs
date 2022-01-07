use super::internal::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Effect {
	pub ttl: f32,
	pub typ: EffectType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EffectType {
	SimpleLine { start: vec3, end: vec3 },
	LaserBeam { start: vec3, orientation: Orientation, len: f32 },
	ParticleExplosion { pos: vec3, color: vec3 },
	ParticleBeam { start: vec3, orientation: Orientation, len: f32, color_filter: vec3 },
	Respawn { pos: vec3 },
}

pub const LASERBEAM_TTL: f32 = 0.25; // seconds
pub const PARTICLES_TTL: f32 = 1.0; // seconds
pub const RESPAWN_TTL: f32 = 1.5; // seconds

impl Effect {
	pub fn simple_line(start: vec3, end: vec3) -> Self {
		Self {
			ttl: LASERBEAM_TTL,
			typ: EffectType::SimpleLine { start, end },
		}
	}

	pub fn laserbeam(start: vec3, orientation: Orientation, len: f32) -> Self {
		Self {
			ttl: LASERBEAM_TTL,
			typ: EffectType::LaserBeam { start, orientation, len },
		}
	}

	pub fn particle_explosion(pos: vec3, color: vec3) -> Self {
		Self {
			ttl: PARTICLES_TTL,
			typ: EffectType::ParticleExplosion { pos, color },
		}
	}

	pub fn particle_beam(start: vec3, orientation: Orientation, len: f32, color_filter: vec3) -> Self {
		Self {
			ttl: PARTICLES_TTL,
			typ: EffectType::ParticleBeam {
				start,
				orientation,
				len,
				color_filter,
			},
		}
	}

	pub fn ricochet(start: vec3, color_filter: vec3) -> Self {
		println!("ricochet");
		let mut rng = rand::thread_rng();
		let len = 50.0;
		let orientation = Orientation {
			pitch: rng.gen_range(20.0 * DEG..90.0 * DEG),
			yaw: rng.gen_range(-180.0 * DEG..180.0 * DEG),
		};
		Self {
			ttl: PARTICLES_TTL,
			typ: EffectType::ParticleBeam {
				start,
				orientation,
				len,
				color_filter,
			},
		}
	}

	pub fn respawn(pos: vec3) -> Self {
		Self {
			ttl: RESPAWN_TTL,
			typ: EffectType::Respawn { pos },
		}
	}
}
