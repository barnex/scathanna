use super::ivec_2::ivec2;
use super::vector2::*;
use std::ops::Mul;

#[allow(non_camel_case_types)]
pub type vec2 = Vector2<f32>;

pub const fn vec2(x: f32, y: f32) -> vec2 {
	vec2::new(x, y)
}

impl vec2 {
	pub fn to_ivec(self) -> ivec2 {
		self.map(|v| v as i32)
	}
}

impl Mul<vec2> for f32 {
	type Output = vec2;

	#[inline]
	fn mul(self, rhs: vec2) -> Self::Output {
		rhs.mul(self)
	}
}
