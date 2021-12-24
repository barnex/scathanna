use super::internal::*;

/// 3D range (i.e. a cuboid region) of integer indices.
/// Used to index VoxelStore Ranges.
/// TODO: private members.

#[derive(PartialEq, Eq, Debug)]
pub struct Cuboid {
	pub min: ivec3,
	pub max: ivec3,
}

//impl<T> Bounds3<T>
//where
//	T: Copy + Ord + std::fmt::Debug,
//{
//	pub fn new(min: Vector3<T>, max: Vector3<T>) -> Self {
//		#[cfg(debug)]
//		if min.x > max.x || min.y > max.y || min.z > max.z {
//			panic!("Bounds3::new: invalid arguments: min = {:?}, max = {:?}", min, max);
//		}
//
//		Self { min, max }
//	}
//}

impl Cuboid {
	pub fn new(min: ivec3, max: ivec3) -> Self {
		Self { min, max }
	}

	pub fn with_size(min: ivec3, size: uvec3) -> Self {
		Self::new(min, min + size.as_ivec())
	}

	/// Cubic range.
	pub fn cube(min: ivec3, size: u32) -> Self {
		let size = size as i32;
		Self {
			min,
			max: min + ivec3(size, size, size),
		}
	}

	pub fn size(&self) -> ivec3 {
		self.max - self.min
	}

	#[must_use]
	pub fn translate(&self, delta: ivec3) -> Self {
		Self::new(self.min + delta, self.max + delta)
	}

	#[must_use]
	pub fn intersect(&self, rhs: &Self) -> Self {
		Self::new(self.min.zip(rhs.min, i32::max), self.max.zip(rhs.max, i32::min))
	}

	pub fn intersects(&self, rhs: &Self) -> bool {
		!self.intersect(rhs).is_empty()
	}

	pub fn is_empty(&self) -> bool {
		self.min.x >= self.max.x || //.
		self.min.y >= self.max.y ||
		self.min.z >= self.max.z
	}

	pub fn contains(&self, idx: ivec3) -> bool {
		idx.x >= self.min.x && //.
		idx.y >= self.min.y &&
		idx.z >= self.min.z && 
		idx.x < self.max.x && 
		idx.y < self.max.y &&
		idx.z < self.max.z
	}

	pub fn contains_range(&self, other: &Cuboid) -> bool {
		self.min.x <= other.min.x && self.min.y <= other.min.y && self.min.z <= other.min.z && self.max.x >= other.max.x && self.max.y >= other.max.y && self.max.z >= other.max.z
	}

	pub fn iter(&self) -> impl Iterator<Item = ivec3> {
		self.iter_by(1)
	}

	/// Iterate over the positions inside this cube,
	/// with stride `step`
	/// `step` must evenly divide the cube's size. E.g.:
	///  cube size `8` => step `1`, `2`, `4` or `8`.
	pub fn iter_by(&self, step: u32) -> impl Iterator<Item = ivec3> {
		// TODO: real iterator, don't collect first.
		let step = step as i32;
		debug_assert!(self.size().x % step == 0);
		debug_assert!(self.size().y % step == 0);
		debug_assert!(self.size().z % step == 0);

		let (min, max) = (self.min, self.max);
		let mut items = Vec::new();

		let mut z = min.z;
		while z < max.z {
			let mut y = min.y;
			while y < max.y {
				let mut x = min.x;
				while x < max.x {
					items.push(ivec3(x, y, z));
					x += step;
				}
				y += step;
			}
			z += step;
		}

		items.into_iter()
	}
}

impl fmt::Display for Cuboid {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Range{}-{}", self.min, self.max)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn contains_range() {
		let a = Cuboid::new(ivec3(1, 2, 3), ivec3(2, 4, 6));
		let b = Cuboid::new(ivec3(1, 1, 3), ivec3(2, 5, 7));
		let c = Cuboid::new(ivec3(1, 3, 3), ivec3(2, 4, 6));
		assert!(a.contains_range(&a));
		assert!(b.contains_range(&a));
		assert!(!c.contains_range(&a));
	}

	#[test]
	fn intersect() {
		let a = Cuboid::new(ivec3(1, 2, 3), ivec3(2, 4, 6));
		let b = Cuboid::new(ivec3(1, 1, 4), ivec3(3, 4, 5));
		assert_eq!(a.intersect(&b), Cuboid::new(ivec3(1, 2, 4), ivec3(2, 4, 5)));
		assert_eq!(&a.intersect(&a), &a);
	}

	#[test]
	fn is_empty() {
		assert!(!Cuboid::new(ivec3(1, 2, 3), ivec3(2, 4, 6)).is_empty());
		assert!(Cuboid::new(ivec3(0, 0, 0), ivec3(0, 0, 0)).is_empty());
		assert!(Cuboid::new(ivec3(0, 0, 0), ivec3(1, 0, 0)).is_empty());
		assert!(Cuboid::new(ivec3(0, 0, 0), ivec3(0, 0, 1)).is_empty());
		assert!(Cuboid::new(ivec3(0, 0, 0), ivec3(0, 1, 1)).is_empty());
		assert!(Cuboid::new(ivec3(1, 0, 0), ivec3(0, 0, 0)).is_empty());
	}

	#[test]
	fn contains() {
		let b = Cuboid::new(ivec3(1, 2, 3), ivec3(2, 4, 6));
		assert!(!b.contains(ivec3(0, 2, 3)));
		assert!(!b.contains(ivec3(1, 1, 3)));
		assert!(!b.contains(ivec3(1, 2, 2)));
		assert!(b.contains(ivec3(1, 2, 3)));
		assert!(b.contains(ivec3(1, 3, 5)));
		assert!(!b.contains(ivec3(2, 4, 6)));
		assert!(!b.contains(ivec3(1, 4, 6)));
		assert!(!b.contains(ivec3(2, 3, 6)));
		assert!(!b.contains(ivec3(2, 4, 5)));
	}
}
