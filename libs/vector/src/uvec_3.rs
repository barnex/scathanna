use super::ivec_3::*;
use super::vector3::*;
use std::ops::Mul;

#[allow(non_camel_case_types)]
pub type uvec3 = Vector3<u32>;

pub const fn uvec3(x: u32, y: u32, z: u32) -> uvec3 {
	uvec3::new(x, y, z)
}

impl Mul<uvec3> for u32 {
	type Output = uvec3;

	#[inline]
	fn mul(self, rhs: uvec3) -> Self::Output {
		rhs.mul(self)
	}
}

impl uvec3 {
	#[inline]
	pub fn as_ivec(self) -> ivec3 {
		ivec3(self.x as i32, self.y as i32, self.z as i32)
	}
}
