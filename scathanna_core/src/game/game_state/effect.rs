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
	ParticleBeam { start: vec3, orientation: Orientation, len: f32 },
	Respawn { pos: vec3 },
}

pub const LASERBEAM_TTL: f32 = 0.25; // seconds
pub const PARTICLES_TTL: f32 = 2.0; // seconds
pub const RESPAWN_TTL: f32 = 1.5; // seconds

impl Effect {
	pub fn particle_explosion(pos: vec3, color: vec3) -> Self {
		Self {
			ttl: PARTICLES_TTL,
			typ: EffectType::ParticleExplosion { pos, color },
		}
	}

	pub fn respawn(pos: vec3) -> Self {
		Self {
			ttl: RESPAWN_TTL,
			typ: EffectType::Respawn { pos },
		}
	}
}

pub fn simple_line_effect(start: vec3, end: vec3) -> ClientMsg {
	ClientMsg::AddEffect {
		effect: Effect {
			ttl: LASERBEAM_TTL,
			typ: EffectType::SimpleLine { start, end },
		},
	}
}

pub fn laserbeam_effect(start: vec3, orientation: Orientation, len: f32) -> ClientMsg {
	ClientMsg::AddEffect {
		effect: Effect {
			ttl: LASERBEAM_TTL,
			typ: EffectType::LaserBeam { start, orientation, len },
		},
	}
}

// TODO: Effect::particle_beam
pub fn particle_beam_effect(start: vec3, orientation: Orientation, len: f32) -> ClientMsg {
	ClientMsg::AddEffect {
		effect: Effect {
			ttl: PARTICLES_TTL,
			typ: EffectType::ParticleBeam { start, orientation, len },
		},
	}
}
