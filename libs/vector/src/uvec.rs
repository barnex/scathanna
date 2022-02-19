use super::*;
use std::ops::Mul;

#[allow(non_camel_case_types)]
pub type uvec2 = Vector2<u32>;

pub const fn uvec2(x: u32, y: u32) -> uvec2 {
	uvec2::new(x, y)
}

impl Mul<uvec2> for u32 {
	type Output = uvec2;

	#[inline]
	fn mul(self, rhs: uvec2) -> Self::Output {
		rhs.mul(self)
	}
}

impl uvec2 {
	#[inline]
	pub fn to_i32(self) -> ivec2 {
		self.map(|v| v as i32)
	}

	pub fn to_f32(self) -> Vector2<f32> {
		self.map(|v| v as f32)
	}
}

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
		ivec3(self.x() as i32, self.y() as i32, self.z() as i32)
	}
}

#[allow(non_camel_case_types)]
pub type uvec4 = Vector4<u32>;

pub const fn uvec4(x: u32, y: u32, z: u32, w: u32) -> uvec4 {
	uvec4::new(x, y, z, w)
}

impl Mul<uvec4> for u32 {
	type Output = uvec4;

	#[inline]
	fn mul(self, rhs: uvec4) -> Self::Output {
		rhs.mul(self)
	}
}
