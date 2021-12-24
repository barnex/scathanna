use super::vec3;
use super::vector3::*;
use std::ops::Mul;

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
	pub fn is_finite(self) -> bool {
		self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
	}

	pub fn to_f32(self) -> vec3 {
		self.into()
	}
}
