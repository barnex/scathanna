use super::internal::*;
use std::f32::consts::PI;

/// UniformDisk maps a point (u,v) from the unit square to the unit disk,
/// preserving uniformity.
///
/// This is used for sampling a disk-shaped light source.
pub fn uniform_disk((u, v): (f32, f32)) -> (f32, f32) {
	let theta = (2.0 * PI) * u;
	let r = f32::sqrt(v);
	(r * f32::cos(theta), r * f32::sin(theta))
}

/// CosineSphere transforms a point (u,v) from the unit square to a vector
/// on the hemisphere around the given normal, cosine weighted.
/// I.e. the resulting vectors are distributed proportionally to the cosine of the angle with the normal,
/// assuming that the original (u,v) are uniformly distributed.
///
/// This is used for cosine-weighted importance sampling. I.e. for Lambertian (matte) scattering.
pub fn cosine_sphere((u, v): (f32, f32), normal: vec3) -> vec3 {
	// Malley’s Method: project disk onto hemisphere.
	// http://www.pbr-book.org/3ed-2018/Monte_Carlo_Integration/2D_Sampling_with_Multidimensional_Transformations.html#fig:malley
	let (x, y) = uniform_disk((u, v));
	let z = f32::sqrt(1.0 - (x * x + y * y));
	make_basis(normal) * vec3(x, y, z)
}

/// Create orthonormal basis with given z-axis.
/// See Shirley, Fundamentals of Computer Graphics
pub fn make_basis(ez: vec3) -> Matrix3<f32> {
	//debug_assert!(ez.is_normalized());
	let mut t = ez;
	let mut i = 0;
	let mut min = f32::abs(t[i]);
	if f32::abs(t[1]) < min {
		i = 1;
		min = f32::abs(t[1]);
	}
	if f32::abs(t[2]) < min {
		i = 2;
	}
	t[i] = 1.0;

	let ex = t.cross(ez).normalized();
	let ey = ex.cross(ez);

	Matrix3::from([ex, ey, ez])
}
