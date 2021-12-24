use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SpawnPoint {
	pub pos: ivec3,
}

impl SpawnPoint {
	pub fn position(&self) -> vec3 {
		self.pos.to_f32()
	}

	pub fn orientation(&self) -> Orientation {
		Orientation::default() // TODO
	}
}
