use super::vec_3::vec3;
use super::vector3::*;
use std::ops::Mul;

#[allow(non_camel_case_types)]
pub type ivec3 = Vector3<i32>;

pub const fn ivec3(x: i32, y: i32, z: i32) -> ivec3 {
	ivec3::new(x, y, z)
}

impl ivec3 {
	pub const ZERO: Self = ivec3(0, 0, 0);
	pub const ONES: Self = ivec3(1, 1, 1);
	pub const EX: Self = ivec3(1, 0, 0);
	pub const EY: Self = ivec3(0, 1, 0);
	pub const EZ: Self = ivec3(0, 0, 1);

	pub fn to_f32(self) -> vec3 {
		self.map(|v| v as f32)
	}
}

impl Mul<ivec3> for i32 {
	type Output = ivec3;

	#[inline]
	fn mul(self, rhs: ivec3) -> Self::Output {
		rhs.mul(self)
	}
}
