use super::*;
use std::ops::Mul;

/// 2-component f64 vector, like GLSL's dvec2.
#[allow(non_camel_case_types)]
pub type dvec2 = Vector2<f64>;

pub const fn dvec2(x: f64, y: f64) -> dvec2 {
	dvec2::new(x, y)
}

impl Mul<dvec2> for f64 {
	type Output = dvec2;

	/// Multiply by a constant.
	/// ```
	/// # use vector::*;
	/// assert_eq!(2.0 * dvec2(1.0, 2.0), dvec2(2.0, 4.0));
	/// ```
	#[inline]
	fn mul(self, rhs: dvec2) -> Self::Output {
		rhs.mul(self)
	}
}

#[allow(non_camel_case_types)]
pub type dvec3 = Vector3<f64>;

pub const fn dvec3(x: f64, y: f64, z: f64) -> dvec3 {
	dvec3::new(x, y, z)
}

impl Mul<dvec3> for f64 {
	type Output = dvec3;

	#[inline]
	fn mul(self, rhs: dvec3) -> Self::Output {
		rhs.mul(self)
	}
}

impl From<vec3> for dvec3 {
	fn from(v: vec3) -> Self {
		v.map(|v| v as f64)
	}
}

impl dvec3 {
	pub fn to_f32(self) -> vec3 {
		self.into()
	}
}

#[allow(non_camel_case_types)]
pub type dvec4 = Vector4<f64>;

pub const fn dvec4(x: f64, y: f64, z: f64, w: f64) -> dvec4 {
	dvec4::new(x, y, z, w)
}

impl Mul<dvec4> for f64 {
	type Output = dvec4;

	#[inline]
	fn mul(self, rhs: dvec4) -> Self::Output {
		rhs.mul(self)
	}
}
