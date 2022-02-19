use std::mem;
use std::ops::Mul;
use vector::*;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct mat4 {
	el: [[f32; 4]; 4],
}

impl mat4 {
	pub fn from(el: [[f32; 4]; 4]) -> Self {
		Self { el }
	}

	/// Convenience constructor, transposes its input.
	pub fn transpose(el: [[f32; 4]; 4]) -> Self {
		Self::from(el).transposed()
	}

	pub fn as_array(&self) -> &[f32; 16] {
		unsafe { mem::transmute(self) }
	}

	pub const UNIT: Self = Self {
		el: [
			[1.0, 0.0, 0.0, 0.0], //
			[0.0, 1.0, 0.0, 0.0], //
			[0.0, 0.0, 1.0, 0.0], //
			[0.0, 0.0, 0.0, 1.0], //
		],
	};

	pub const ZERO: Self = Self {
		el: [
			[0.0, 0.0, 0.0, 0.0], //
			[0.0, 0.0, 0.0, 0.0], //
			[0.0, 0.0, 0.0, 0.0], //
			[0.0, 0.0, 0.0, 0.0], //
		],
	};

	#[must_use]
	pub fn transposed(&self) -> Self {
		let mut t = Self::ZERO.clone();
		for i in 0..4 {
			for j in 0..4 {
				t.el[i][j] = self.el[j][i];
			}
		}
		t
	}

	// TODO: mul<vec4>
	pub fn transform_point_ignore_w(&self, rhs: vec3) -> vec3 {
		let m = self.el;
		let (x, y, z) = (rhs.x(), rhs.y(), rhs.z());
		vec3(
			m[0][0] * x + m[1][0] * y + m[2][0] * z,
			m[0][1] * x + m[1][1] * y + m[2][1] * z,
			m[0][2] * x + m[1][2] * y + m[2][2] * z,
		)
	}
}

impl Mul<&mat4> for &mat4 {
	type Output = mat4;

	/// Matrix-Matrix multiplication.
	fn mul(self, rhs: &mat4) -> mat4 {
		let mut c = mat4::ZERO;
		for i in 0..4 {
			for j in 0..4 {
				for k in 0..4 {
					c.el[i][j] = c.el[i][j] + rhs.el[i][k] * self.el[k][j]
				}
			}
		}
		c
	}
}

// allows chaining multiplications:  &a * &b * &c
impl Mul<&mat4> for mat4 {
	type Output = mat4;

	/// Matrix-Matrix multiplication.
	fn mul(self, rhs: &mat4) -> mat4 {
		(&self).mul(rhs)
	}
}

impl Mul<mat4> for mat4 {
	type Output = mat4;

	/// Matrix-Matrix multiplication.
	fn mul(self, rhs: mat4) -> mat4 {
		(&self).mul(&rhs)
	}
}
