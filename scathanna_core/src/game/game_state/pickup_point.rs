use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PickupPoint {
	pub pos: ivec3,

	#[serde(default)]
	pub taken: bool,
}

impl PickupPoint {
	pub fn position(&self) -> vec3 {
		self.pos.to_f32()
	}
}
