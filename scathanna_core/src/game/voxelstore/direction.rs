use super::internal::*;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub enum Direction {
	MinusX = 0,
	X = 1,
	MinusY = 2,
	Y = 3,
	MinusZ = 4,
	Z = 5,
}

use Direction::*;

impl Direction {
	/// All directions, handy for iterating over.
	pub const ALL: [Direction; 6] = [MinusX, X, MinusY, Y, MinusZ, Z];

	pub fn ivec(self) -> ivec3 {
		use Direction::*;
		match self {
			X => ivec3(1, 0, 0),
			Y => ivec3(0, 1, 0),
			Z => ivec3(0, 0, 1),
			MinusX => ivec3(-1, 0, 0),
			MinusY => ivec3(0, -1, 0),
			MinusZ => ivec3(0, 0, -1),
		}
	}

	pub fn vec(self) -> vec3 {
		self.ivec().to_f32()
	}

	pub fn unoriented_axis(self) -> usize {
		use Direction::*;
		match self {
			MinusX | X => 0,
			MinusY | Y => 1,
			MinusZ | Z => 2,
		}
	}

	pub fn unoriented_vec(self) -> ivec3 {
		use Direction::*;
		match self {
			MinusX | X => ivec3::EX,
			MinusY | Y => ivec3::EY,
			MinusZ | Z => ivec3::EZ,
		}
	}

	pub fn side_index(self) -> usize {
		// TODO: usize?
		use Direction::*;
		match self {
			MinusX | MinusY | MinusZ => 0,
			X | Y | Z => 1,
		}
	}

	pub fn tangents(self) -> [Self; 2] {
		use Direction::*;
		match self {
			X | MinusX => [Y, Z],
			Y | MinusY => [X, Z],
			Z | MinusZ => [X, Y],
		}
	}
}
