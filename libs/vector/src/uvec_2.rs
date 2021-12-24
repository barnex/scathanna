use super::ivec_2::*;
use super::vector2::*;
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
