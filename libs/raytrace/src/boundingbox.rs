use super::internal::*;
use std::cmp::PartialOrd;

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
	T: Copy + PartialOrd,
{
	/// Bounding box containing all points with coordinates between `min` and `max`.
	/// `min`'s components must not be larger than `max`'s.
	#[inline]
	pub fn new(min: Vector3<T>, max: Vector3<T>) -> Self {
		debug_assert!(min.zip(max, |min, max| (min, max)).iter().all(|(min, max)| min <= max));
		Self { min, max }
	}
}

impl<T> BoundingBox<T>
where
	T: Copy,
{
	/// Convert the bounding box's vertex positions to a different type.
	/// ```
	/// # use raytrace::*;
	/// # use vector::*;
	/// let bb = BoundingBox::new(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0));
	/// assert_eq!(bb.convert::<i32>(), BoundingBox::new(ivec3(1, 2, 3), ivec3(4, 5, 6)));
	///
	/// ```
	pub fn convert<U>(&self) -> BoundingBox<U>
	where
		T: Convert<U>,
		U: Copy + PartialOrd,
	{
		BoundingBox::new(self.min.convert(), self.max.convert())
	}
}

impl<T> BoundingBox<T>
where
	T: Float,
{
	/// Center position.
	/// ```
	/// # use raytrace::*;
	/// # use vector::*;
	/// let bb = BoundingBox::new(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0));
	/// assert_eq!(bb.center(), vec3(2.5, 3.5, 4.5));
	/// ```
	pub fn center(&self) -> Vector3<T> {
		(self.min + self.max) / (T::ONE + T::ONE)
	}
}

impl<T> BoundingBox<T>
where
	T: Number,
{
	/// Size in each direction.
	/// ```
	/// # use raytrace::*;
	/// # use vector::*;
	/// let bb = BoundingBox::new(ivec3(1, 2, 3), ivec3(2, 4, 8));
	/// assert_eq!(bb.size(), ivec3(1, 2, 5));
	/// ```
	pub fn size(&self) -> Vector3<T> {
		self.max - self.min
	}

	/// Construct a bounding box enclosing `self` and an added point.
	/// ```
	/// # use raytrace::*;
	/// # use vector::*;
	/// let a = BoundingBox::new(ivec3(1, 2, 3), ivec3(4, 5, 6));
	/// let b = a.add(ivec3(-1, 20, 4));
	/// assert_eq!(b, BoundingBox::new(ivec3(-1, 2, 3), ivec3(4, 20, 6)));
	/// ```
	#[must_use]
	pub fn add(&self, rhs: Vector3<T>) -> Self {
		Self::new(self.min.zip(rhs, T::partial_min), self.max.zip(rhs, T::partial_max))
	}

	/// Construct a bounding box enclosing `self` and `rhs`.
	/// ```
	/// # use raytrace::*;
	/// # use vector::*;
	/// let a = BoundingBox::new(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0));
	/// let b = BoundingBox::new(vec3(-1.0, 5.0, 4.0), vec3(40.0, 20.0, 6.0));
	/// assert_eq!(a.join(&b), BoundingBox::new(vec3(-1.0, 2.0, 3.0), vec3(40.0, 20.0, 6.0)));
	/// ```
	#[must_use]
	pub fn join(&self, rhs: &Self) -> Self {
		Self::new(self.min.zip(rhs.min, T::partial_min), self.max.zip(rhs.max, T::partial_max))
	}

	/// Construct a bounding box enclosing all points from an iterator.
	/// ```
	/// # use raytrace::*;
	/// # use vector::*;
	/// let points = [vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0), vec3(-1.0, 20.0, 4.0)];
	/// let bb = BoundingBox::from(points.iter());
	/// assert_eq!(bb, Some(BoundingBox::new(vec3(-1.0, 2.0, 3.0), vec3(4.0, 20.0, 6.0))));
	///
	/// let points = Vec::<vec3>::new();
	/// let bb = BoundingBox::from(points.iter());
	/// assert_eq!(bb, None);
	/// ```
	pub fn from<'i>(mut vertices: impl Iterator<Item = &'i Vector3<T>> + 'i) -> Option<Self> {
		let first = match vertices.next() {
			None => return None,
			Some(v) => v,
		};
		let mut bb = Self::new(*first, *first);
		for v in vertices {
			bb = bb.add(*v);
		}
		Some(bb)
	}

	/// Test if a point lies inside the bounding box
	/// (including its boundaries).
	/// ```
	/// # use raytrace::*;
	/// # use vector::*;
	/// let bb = BoundingBox::new(ivec3(1,2,3), ivec3(4,5,6));
	/// assert_eq!(bb.contains(ivec3(1,2,3)), true);
	/// assert_eq!(bb.contains(ivec3(4,5,6)), true);
	/// assert_eq!(bb.contains(ivec3(2,4,4)), true);
	/// assert_eq!(bb.contains(ivec3(2,4,9)), false);
	/// assert_eq!(bb.contains(ivec3(2,9,4)), false);
	/// assert_eq!(bb.contains(ivec3(9,4,4)), false);
	/// assert_eq!(bb.contains(ivec3(-1,4,4)), false);
	/// assert_eq!(bb.contains(ivec3(2,-1,4)), false);
	/// assert_eq!(bb.contains(ivec3(2,4,-1)), false);
	/// ```
	pub fn contains(&self, point: Vector3<T>) -> bool {
		   point.x() >= self.min.x() //.
		&& point.x() <= self.max.x() 
		&& point.y() >= self.min.y()
		&& point.y() <= self.max.y()
		&& point.z() >= self.min.z() 
		&& point.z() <= self.max.z()
	}
}

pub type BoundingBox64 = BoundingBox<f64>;
pub type BoundingBox32 = BoundingBox<f32>;

impl<T> BoundingBox<T> where T: Float{
	#[inline]
	pub fn intersects(&self, r: &Ray<T>) -> bool {
		self.intersect(r).is_some()
	}

	#[inline]
	pub fn intersect(&self, r: &Ray<T>) -> Option<T> {

		let tmin = (self.min - r.start).div3(r.dir);
		let tmax = (self.max - r.start).div3(r.dir);

		let ten3 = tmin.zip(tmax, T::partial_min);
		let tex3 = tmin.zip(tmax, T::partial_max);

		let ten = ten3.reduce(T::partial_max);
		let tex = tex3.reduce(T::partial_min);

		// `>=` aims to cover the degenerate case where
		// the box has size 0 along a dimension
		// (e.g. when wrapping an axis-aligned rectangle).
		if tex >= T::partial_max(T::ZERO, ten) {
			Some(ten)
		} else {
			None
		}
	}
}


#[cfg(test)]
mod test {
	use super::*;

	const EX: dvec3 = dvec3::EX;
	const EY: dvec3 = dvec3::EY;
	const EZ: dvec3 = dvec3::EZ;

	fn ray(start: (f64, f64, f64), dir: dvec3) -> Ray64 {
		Ray64::new(dvec3::from(start), dir)
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
