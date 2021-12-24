use super::internal::*;

/// Axis Aligned Box, used to accelerate intersection tests with groups of objects.
/// See https://en.wikipedia.org/wiki/Minimum_bounding_box#Axis-aligned_minimum_bounding_box.
#[derive(Clone, Debug, PartialEq)]
pub struct BoundingBox<T>
where
	T: Copy,
{
	pub min: Vector3<T>,
	pub max: Vector3<T>,
}

impl<T> BoundingBox<T>
where
	T: Copy,
{
	pub fn new(min: Vector3<T>, max: Vector3<T>) -> Self {
		Self { min, max }
	}
}

impl BoundingBox<f32> {
	/// Convert to double precision.
	pub fn to_f64(&self) -> BoundingBox<f64> {
		BoundingBox::new(self.min.into(), self.max.into())
	}
}

pub type BoundingBox64 = BoundingBox<f64>;

impl BoundingBox64 {
	#[must_use]
	pub fn join(&self, rhs: &Self) -> Self {
		Self {
			min: dmin3(self.min, rhs.min),
			max: dmax3(self.max, rhs.max),
		}
	}

	#[must_use]
	pub fn add(&self, rhs: dvec3) -> Self {
		Self {
			min: dmin3(self.min, rhs),
			max: dmax3(self.max, rhs),
		}
	}

	#[inline]
	pub fn intersects(&self, r: &DRay) -> bool {
		self.intersect(r).is_some()
	}

	#[inline]
	pub fn intersect(&self, r: &DRay) -> Option<f64> {
		let start = r.start;
		let invdir = r.dir.map(|v| 1.0 / v);
		let min: dvec3 = self.min.into();
		let max: dvec3 = self.max.into();

		let tmin = mul3(min - start, invdir);
		let tmax = mul3(max - start, invdir);

		let ten = dmin3(tmin, tmax);
		let tex = dmax3(tmin, tmax);

		let ten = f64::max(f64::max(ten.x, ten.y), ten.z);
		let tex = f64::min(f64::min(tex.x, tex.y), tex.z);

		// `>=` aims to cover the degenerate case where
		// the box has size 0 along a dimension
		// (e.g. when wrapping an axis-aligned rectangle).
		if tex >= f64::max(0.0, ten) {
			Some(ten)
		} else {
			None
		}
	}

	pub fn center(&self) -> dvec3 {
		(self.min + self.max) * 0.5
	}
}

pub fn min3(a: vec3, b: vec3) -> vec3 {
	a.zip(b, f32::min)
}

pub fn max3(a: vec3, b: vec3) -> vec3 {
	a.zip(b, f32::max)
}

pub fn dmin3(a: dvec3, b: dvec3) -> dvec3 {
	a.zip(b, f64::min)
}

pub fn dmax3(a: dvec3, b: dvec3) -> dvec3 {
	a.zip(b, f64::max)
}

pub fn mul3(a: dvec3, b: dvec3) -> dvec3 {
	use std::ops::Mul;
	a.zip(b, f64::mul)
}

impl<'a, I> From<I> for BoundingBox64
where
	I: Iterator<Item = &'a dvec3> + 'a,
{
	fn from(mut iter: I) -> Self {
		let first = iter.next().expect("BoundingBox: from iterator: iterator cannot be empty");
		let mut bb = Self { min: *first, max: *first };
		for pos in iter {
			bb = bb.add(*pos);
		}
		bb
	}
}

#[cfg(test)]
mod test {
	use super::*;

	const EX: dvec3 = dvec3::EX;
	const EY: dvec3 = dvec3::EY;
	const EZ: dvec3 = dvec3::EZ;

	fn ray(start: (f64, f64, f64), dir: dvec3) -> DRay {
		DRay::new(dvec3::from(start), dir)
	}

	#[test]
	fn intersect() {
		let min = dvec3(1.0, 2.0, 3.0);
		let max = dvec3(2.0, 5.0, 6.0);
		let bb = BoundingBox64::new(min, max);

		/*
			Cases with the ray along X:

			<-(14)  (13)->     <-(16) (15)->   <-(18) (17)->

							  +-----------+(2,5,6)
							  |           |
							  |           |
			<-(2)  (1)->      |<-(4) (3)->|  <-(6) (5)->
							  |           |
							  |           |
					   (1,2,3)+-----------+

			<-(8)  (7)->       <-(9) (10)->   <-(12) (11)->
		*/
		assert!(bb.intersects(&ray((0.0, 3.0, 4.0), EX))); //   (1)
		assert!(!bb.intersects(&ray((0.0, 3.0, 4.0), -EX))); // (2)
		assert!(bb.intersects(&ray((1.5, 3.0, 4.0), EX))); //   (3)
		assert!(bb.intersects(&ray((1.5, 3.0, 4.0), -EX))); //  (4)
		assert!(!bb.intersects(&ray((2.5, 3.0, 4.0), EX))); //  (5)
		assert!(bb.intersects(&ray((2.5, 3.0, 4.0), -EX))); //  (6)

		// as above, but shifted down (Y) to miss the box.
		assert!(!bb.intersects(&ray((0.0, -1.0, 4.0), EX))); // (7)
		assert!(!bb.intersects(&ray((0.0, -1.0, 4.0), -EX))); //(8)
		assert!(!bb.intersects(&ray((1.5, -1.0, 4.0), EX))); // (9)
		assert!(!bb.intersects(&ray((1.5, -1.0, 4.0), -EX))); //(10)
		assert!(!bb.intersects(&ray((2.5, -1.0, 4.0), EX))); // (11)
		assert!(!bb.intersects(&ray((2.5, -1.0, 4.0), -EX))); //(12)

		// as above, but shifted up (Y) to miss the box.
		assert!(!bb.intersects(&ray((0.0, 6.0, 4.0), EX))); // (13)
		assert!(!bb.intersects(&ray((0.0, 6.0, 4.0), -EX))); //(14)
		assert!(!bb.intersects(&ray((1.5, 6.0, 4.0), EX))); // (15)
		assert!(!bb.intersects(&ray((1.5, 6.0, 4.0), -EX))); //(16)
		assert!(!bb.intersects(&ray((2.5, 6.0, 4.0), EX))); // (17)
		assert!(!bb.intersects(&ray((2.5, 6.0, 4.0), -EX))); //(18)

		/*
			Cases with the ray along Y:

								   ^
								   |
								  (6)
								  (5)
								   |
								   v
							  +-----------+(2,5,6)
							  |    ^      |
							  |    |      |
							  |   (3)     |
							  |   (4)     |
							  |    |      |
							  |    v      |
					   (1,2,3)+-----------+
									^
									|
								   (1)
								   (2)
									|
									v

		*/
		assert!(bb.intersects(&ray((1.5, 1.0, 4.0), EY))); //   (1)
		assert!(!bb.intersects(&ray((1.5, 1.0, 4.0), -EY))); // (2)
		assert!(bb.intersects(&ray((1.5, 3.0, 4.0), EY))); //   (3)
		assert!(bb.intersects(&ray((1.5, 3.0, 4.0), -EY))); //  (4)
		assert!(bb.intersects(&ray((1.5, 6.0, 4.0), -EY))); //  (5)
		assert!(!bb.intersects(&ray((1.5, 6.0, 4.0), EY))); //  (6)

		// as above, but shifted left to miss the box
		assert!(!bb.intersects(&ray((0.5, 1.0, 4.0), EY)));
		assert!(!bb.intersects(&ray((0.5, 1.0, 4.0), -EY)));
		assert!(!bb.intersects(&ray((0.5, 3.0, 4.0), EY)));
		assert!(!bb.intersects(&ray((0.5, 3.0, 4.0), -EY)));
		assert!(!bb.intersects(&ray((0.5, 6.0, 4.0), -EY)));
		assert!(!bb.intersects(&ray((0.5, 6.0, 4.0), EY)));

		// as above, but shifted right to miss the box
		assert!(!bb.intersects(&ray((3.0, 1.0, 4.0), EY)));
		assert!(!bb.intersects(&ray((3.0, 1.0, 4.0), -EY)));
		assert!(!bb.intersects(&ray((3.0, 3.0, 4.0), EY)));
		assert!(!bb.intersects(&ray((3.0, 3.0, 4.0), -EY)));
		assert!(!bb.intersects(&ray((3.0, 6.0, 4.0), -EY)));
		assert!(!bb.intersects(&ray((3.0, 6.0, 4.0), EY)));

		// as above, but shifted right to miss the box
		assert!(!bb.intersects(&ray((3.0, 1.0, 4.0), EY)));
		assert!(!bb.intersects(&ray((3.0, 1.0, 4.0), -EY)));
		assert!(!bb.intersects(&ray((3.0, 3.0, 4.0), EY)));
		assert!(!bb.intersects(&ray((3.0, 3.0, 4.0), -EY)));
		assert!(!bb.intersects(&ray((3.0, 6.0, 4.0), -EY)));
		assert!(!bb.intersects(&ray((3.0, 6.0, 4.0), EY)));

		// Similar cases with the ray along Z:
		assert!(bb.intersects(&ray((1.5, 3.0, 2.0), EZ)));
		assert!(!bb.intersects(&ray((1.5, 3.0, 2.0), -EZ)));
		assert!(bb.intersects(&ray((1.5, 3.0, 4.0), EZ)));
		assert!(bb.intersects(&ray((1.5, 3.0, 4.0), -EZ)));
		assert!(bb.intersects(&ray((1.5, 3.0, 7.0), -EZ)));
		assert!(!bb.intersects(&ray((1.5, 3.0, 7.0), EZ)));

		// as above, but shifted to miss the box
		assert!(!bb.intersects(&ray((-1.0, 3.0, 2.0), EZ)));
		assert!(!bb.intersects(&ray((-1.0, 3.0, 2.0), -EZ)));
		assert!(!bb.intersects(&ray((-1.0, 3.0, 4.0), EZ)));
		assert!(!bb.intersects(&ray((-1.0, 3.0, 4.0), -EZ)));
		assert!(!bb.intersects(&ray((-1.0, 3.0, 7.0), -EZ)));
		assert!(!bb.intersects(&ray((-1.0, 3.0, 7.0), EZ)));
	}

	#[test]
	fn degenerate() {
		// Corner case: bounding box with size zero in one dimension.
		// It should still work (e.g.: this may happen when bounding a 2D shape).
		let bb = BoundingBox64::new(dvec3(-1., -1., 0.), dvec3(1., 1., 0.));
		assert!(bb.intersects(&ray((0., 0., 1.), -EZ)));
	}
}
