use super::internal::*;

#[derive(Debug, Clone)]
pub struct Ray<T>
where
	T: Copy,
{
	pub start: Vector3<T>,
	pub dir: Vector3<T>,
}

// A Ray is a half-line with a start point (exclusive),
// extending in direction dir (unit vector).
pub type DRay = Ray<f64>;

pub type Rayf = Ray<f32>;

impl Rayf {
	/// Constructs a ray with given starting point and direction.
	/// Both must be finite, and dir must be a unit vector.
	#[inline]
	pub fn new(start: vec3, dir: vec3) -> Self {
		//debug_assert!(start.is_finite());
		//debug_assert!(is_normalized(dir));
		Self { start, dir }
	}
}

impl DRay {
	/// Constructs a ray with given starting point and direction.
	/// Both must be finite, and dir must be a unit vector.
	#[inline]
	pub fn new(start: dvec3, dir: dvec3) -> Self {
		debug_assert!(start.is_finite());
		debug_assert!(is_normalized(dir));
		Self { start, dir }
	}

	/// The ray with its starting point offset by `delta_t` along the ray direction.
	#[must_use]
	#[inline]
	pub fn offset(&self, delta_t: f64) -> Self {
		Self::new(self.start + delta_t * self.dir, self.dir)
	}

	/// Point at distance `t` (positive) from the start.
	#[inline]
	pub fn at(&self, t: f64) -> dvec3 {
		self.start + t * self.dir
	}

	/// Checks that the ray is free of NaNs and has an approximately normalized direction.
	/// Intended for use with `debug_assert`.
	pub fn is_valid(&self) -> bool {
		self.start.is_finite() && is_normalized(self.dir)
	}
}

/// Test if the vector has approximately unit length.
/// Intended for use with debug_assert! where a unit vector is expected.
fn is_normalized(v: dvec3) -> bool {
	(v.len() - 1.0).abs() < 1e-6
}
