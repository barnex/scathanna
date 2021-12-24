use super::internal::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub struct VoxelType(pub u8);

impl VoxelType {
	pub const fn is_empty(&self) -> bool {
		self.0 == Self::EMPTY.0
	}

	pub const EMPTY: Self = Self(0);
	pub const LAVA: Self = Self(2);
}

impl fmt::Display for VoxelType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}
