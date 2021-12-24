use super::internal::*;

/// Size of a 3D bounding box around the Entity's Position.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Bounds {
	pub size: vec3,
}

impl Bounds {
	/// Convert to a bounding box at position `pos`, in world coordinates.
	pub fn at(&self, pos: vec3) -> BoundingBox {
		BoundingBox::new(pos - 0.5 * self.size, pos + 0.5 * self.size)
	}
}
