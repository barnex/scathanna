use super::vector2::*;
use std::ops::Mul;

#[allow(non_camel_case_types)]
pub type dvec2 = Vector2<f64>;

pub const fn dvec2(x: f64, y: f64) -> dvec2 {
	dvec2::new(x, y)
}

impl Mul<dvec2> for f64 {
	type Output = dvec2;

	#[inline]
	fn mul(self, rhs: dvec2) -> Self::Output {
		rhs.mul(self)
	}
}
