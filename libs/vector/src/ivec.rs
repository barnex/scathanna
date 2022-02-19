use super::*;
use std::ops::Mul;

#[allow(non_camel_case_types)]
pub type ivec2 = Vector2<i32>;

pub const fn ivec2(x: i32, y: i32) -> ivec2 {
	ivec2::new(x, y)
}

impl ivec2 {
	pub fn to_vec(self) -> vec2 {
		self.map(|v| v as f32)
	}
	pub fn to_f32(self) -> vec2 {
		self.map(|v| v as f32)
	}
}

impl Mul<ivec2> for i32 {
	type Output = ivec2;

	#[inline]
	fn mul(self, rhs: ivec2) -> Self::Output {
		rhs.mul(self)
	}
}

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

#[allow(non_camel_case_types)]
pub type ivec4 = Vector4<i32>;

pub const fn ivec4(x: i32, y: i32, z: i32, w: i32) -> ivec4 {
	ivec4::new(x, y, z, w)
}

impl Mul<ivec4> for i32 {
	type Output = ivec4;

	#[inline]
	fn mul(self, rhs: ivec4) -> Self::Output {
		rhs.mul(self)
	}
}
