use super::internal::*;

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Rectangle {
	pub position: ivec3,
	pub size: uvec2,
	pub direction: Direction,
}

impl Rectangle {
	/// Vertices in counterclockwise order (as seen looking on to the surface normal).
	pub fn vertices(&self) -> [ivec3; 4] {
		let [du, dv] = self.direction.tangents().map(Direction::ivec);
		let (du, dv) = (du * self.size.x() as i32, dv * self.size.y() as i32);
		let o = self.position;

		[
			o, //
			o + du,
			o + du + dv,
			o + dv,
		]
	}
}
