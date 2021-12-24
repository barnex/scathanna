use super::internal::*;

/// Aligned, cubic range of integer positions.
/// Size is a power of two, position is aligned to size.
/// E.g.: a `4x4x4` cube with position a multiple of 4.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cube {
	position: ivec3,
	size: u32,
}

impl Cube {
	/// Size must be a power of two, position is aligned to size.
	/// E.g.: a `4x4x4` cube with position a multiple of 4.
	pub fn new(position: ivec3, size: u32) -> Self {
		debug_assert!(size.is_power_of_two());
		debug_assert!(is_aligned_to(position, size));
		Self { position, size }
	}

	pub fn position(&self) -> ivec3 {
		self.position
	}

	pub fn size(&self) -> u32 {
		self.size
	}

	#[must_use]
	pub fn translate(&self, delta: ivec3) -> Self {
		Self {
			position: self.position + delta,
			size: self.size,
		}
	}

	pub fn min(&self) -> ivec3 {
		self.position
	}

	pub fn max(&self) -> ivec3 {
		let size = self.size as i32;
		self.position + ivec3(size, size, size)
	}
}

impl fmt::Debug for Cube {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}-{}", self.position(), self.size())
	}
}

// test commented out because it spews noise to stderr.
/*
#[cfg(test)]
mod test {
	use super::*;

	#[test]
	#[should_panic]
	fn ensure_aligned() {
		let _ = CRange::new(ivec3(0, 0, 12), 8);
	}
}
*/
