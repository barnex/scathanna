use super::*;
use std::ops::Mul;

#[allow(non_camel_case_types)]
pub type vec2 = Vector2<f32>;

#[inline]
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

#[allow(non_camel_case_types)]
pub type vec3 = Vector3<f32>;

#[inline]
pub const fn vec3(x: f32, y: f32, z: f32) -> vec3 {
	vec3::new(x, y, z)
}

impl vec3 {
	//pub fn to_ivec(self) -> ivec3 {
	//	self.map(|v| v as i32)
	//}

	pub fn floor(self) -> ivec3 {
		self.map(|v| v.floor() as i32)
	}

	pub fn to_f64(self) -> dvec3 {
		self.map(|v| v as f64)
	}
}

impl From<dvec3> for vec3 {
	fn from(v: dvec3) -> Self {
		v.map(|v| v as f32)
	}
}

impl Mul<vec3> for f32 {
	type Output = vec3;

	#[inline]
	fn mul(self, rhs: vec3) -> Self::Output {
		rhs.mul(self)
	}
}

#[allow(non_camel_case_types)]
pub type vec4 = Vector4<f32>;

#[inline]
pub const fn vec4(x: f32, y: f32, z: f32, w: f32) -> vec4 {
	vec4::new(x, y, z, w)
}

impl Mul<vec4> for f32 {
	type Output = vec4;

	#[inline]
	fn mul(self, rhs: vec4) -> Self::Output {
		rhs.mul(self)
	}
}
