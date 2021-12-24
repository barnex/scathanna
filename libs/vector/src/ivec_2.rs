use super::vec_2::vec2;
use super::vector2::*;
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
