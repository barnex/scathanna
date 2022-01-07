use super::internal::*;

/// Node in an octree that efficiently stores voxels.
/// When a level in the octree is filled entirely with the same type of voxel
/// (including empty), its children are not stored explicitly.
#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub enum Node {
	Uniform(VoxelType),        // Node is filled with 8x the same voxel
	Composite(Box<[Node; 8]>), // Node is filled with different voxels.
}

use Node::*;

impl Node {
	pub fn set_range(&mut self, self_size: u32, range: &Cuboid, value: VoxelType) {
		let self_range = Cuboid::cube(ivec3::ZERO, self_size);

		debug_assert!(self_size.is_power_of_two());
		debug_assert!(!range.is_empty());
		debug_assert!(!self_range.is_empty());
		debug_assert!(range.intersects(&self_range));

		if range == &self_range {
			*self = Self::Uniform(value);
		} else {
			debug_assert!(self_range.contains_range(&range));
			debug_assert!(self_size > 1);

			let children = self.ensure_composite();
			let child_size = self_size / 2;

			for (i, &dir) in Self::CHILD_POS.iter().enumerate() {
				let child_pos = self_range.min + dir * (child_size as i32);
				let child_range = Cuboid::cube(child_pos, child_size);
				let intersection = child_range.intersect(range);
				if !intersection.is_empty() {
					children[i].set_range(child_size, &intersection.translate(-child_pos), value)
				}
			}

			// simplify
			if let Uniform(v) = children[0] {
				if children.iter().map(|c| c == &Uniform(v)).fold(true, and) {
					*self = Uniform(v);
				}
			}
		}
	}

	pub fn query_cube(&self, self_size: u32, range: &Cube) -> CubeType {
		debug_assert!(self_size.is_power_of_two());
		debug_assert!(range.size() >= 1);
		debug_assert!(range.size() <= self_size);

		match self {
			Uniform(v) => CubeType::Uniform(*v),
			Composite(ch) => {
				debug_assert!(self_size > 1);
				if range.size() == self_size {
					debug_assert!(range == &Cube::new(ivec3(0, 0, 0), self_size));
					debug_assert!(!ch.iter().map(|c| c == &ch[0]).fold(true, and));
					CubeType::Mixed
				} else {
					// queried range is smaller than self: recurse
					debug_assert!(self_size > range.size());
					debug_assert!(Cuboid::cube(ivec3(0, 0, 0), self_size).contains_range(&Cuboid::new(range.min(), range.max())));

					// child position => index in children array
					let child_size = self_size / 2;
					let mask = child_size as i32;
					let child_dir = range.position().map(|v| ((v & mask) != 0) as usize);
					let child_idx = child_dir.z << 2 | child_dir.y << 1 | child_dir.x << 0;
					debug_assert!(child_idx <= 8);

					let child_range = range.translate(-child_dir.map(|v| v as i32) * (child_size as i32));

					ch[child_idx].query_cube(child_size, &child_range)
				}
			}
		}
	}

	pub fn visit_nonempty_cubes<F: FnMut(Cube, VoxelType)>(&self, self_pos: ivec3, self_size: u32, mut f: F) {
		self.visit_nonempty_cubes_rec(Cube::new(ivec3(0, 0, 0), self_size), &mut |cube, voxel| f(cube.translate(self_pos), voxel))
	}

	fn visit_nonempty_cubes_rec<F: FnMut(Cube, VoxelType)>(&self, self_range: Cube, f: &mut F) {
		match self {
			Uniform(v) => {
				if !v.is_empty() {
					f(self_range, *v)
				}
			}
			Composite(ch) => {
				for (i, child_range) in Self::child_ranges(&self_range).enumerate() {
					ch[i].visit_nonempty_cubes_rec(child_range, f)
				}
			}
		}
	}

	pub fn intersects(&self, self_range: Cube, ray: &DRay) -> bool {
		self.intersect(self_range, ray).is_some()
	}

	pub fn intersect(&self, self_range: Cube, ray: &DRay) -> Option<(VoxelType, f64)> {
		if self == &Node::Uniform(VoxelType(0)) {
			return None;
		}

		let size = self_range.size() as i32;
		let min = self_range.position().map(|v| v as f64);
		let max = (self_range.position() + ivec3(size, size, size)).map(|v| v as f64);
		if !BoundingBox64::new(min, max).intersects(ray) {
			return None;
		}

		match self {
			Node::Uniform(v) => BoundingBox64::new(min, max).intersect(ray).map(|t| (*v, t)),
			Node::Composite(ch) => {
				let mut result = None;
				for (i, child_range) in Self::child_ranges(&self_range).enumerate() {
					result = frontmost(result, ch[i].intersect(child_range, ray))
				}
				result
			}
		}
	}
}

impl Default for Node {
	fn default() -> Self {
		Self::Uniform(VoxelType::default())
	}
}

impl std::fmt::Debug for Node {
	/// A compact text representation:
	///  - Just the voxel number for Uniforms
	///  - List of children for composites.
	/// E.g.:
	///    [0,1,0,0,0,0,0,[2,3,4,5,2,1,2,3]]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Uniform(v) => write!(f, "{}", v.0),
			Self::Composite(ch) => {
				write!(f, "[")?;
				for (i, ch) in ch.iter().enumerate() {
					if i != 0 {
						write!(f, ",")?;
					}
					ch.fmt(f)?;
				}
				write!(f, "]")
			}
		}
	}
}

//__________________________________________________________________ private

impl Node {
	/// Turn a Node into a Composite variant if not yet the case,
	/// return its children.
	fn ensure_composite(&mut self) -> &mut [Node; 8] {
		match self {
			Composite(ch) => ch,
			Uniform(v) => {
				*self = Composite(Box::new([
					// Node is not (and should not be) Copy,
					// so making an array of it is not concise...
					Uniform(*v),
					Uniform(*v),
					Uniform(*v),
					Uniform(*v),
					Uniform(*v),
					Uniform(*v),
					Uniform(*v),
					Uniform(*v),
				]));
				if let Composite(ch) = self {
					ch
				} else {
					unreachable!()
				}
			}
		}
	}

	const CHILD_POS: [ivec3; 8] = [
		ivec3(0, 0, 0),
		ivec3(1, 0, 0),
		ivec3(0, 1, 0),
		ivec3(1, 1, 0),
		ivec3(0, 0, 1),
		ivec3(1, 0, 1),
		ivec3(0, 1, 1),
		ivec3(1, 1, 1),
	];

	fn child_ranges(self_range: &Cube) -> impl Iterator<Item = Cube> + '_ {
		let size = self_range.size() / 2;
		Self::CHILD_POS.iter().map(move |dir| Cube::new(self_range.position() + *dir * (size as i32), size))
	}
}

//__________________________________________________________________ test

#[cfg(test)]
mod test {
	use super::*;

	// Shorthand for constructing a Uniform Node.
	//fn uniform(voxel_type: u8) -> Node {
	//	Uniform(VoxelType(voxel_type))
	//}

	// Shorthand for constructing a Composite Node w/ uniform children.
	fn composite(ch: [u8; 8]) -> Node {
		Composite(Box::new(ch.map(|v| Node::Uniform(VoxelType(v)))))
	}

	fn crange(pos: (i32, i32, i32), size: u32) -> Cube {
		Cube::new(pos.into(), size)
	}

	fn irange(min: (i32, i32, i32), max: (i32, i32, i32)) -> Cuboid {
		Cuboid::new(min.into(), max.into())
	}

	fn cube(min: (i32, i32, i32), size: u32) -> Cuboid {
		Cuboid::cube(min.into(), size)
	}

	fn v(voxel_type: u8) -> VoxelType {
		VoxelType(voxel_type)
	}

	#[test]
	fn intersect() {
		const S: u32 = 16;
		// uniform node at (0,0,0)
		{
			let mut node = Node::default();
			node.set_range(S, &cube((0, 0, 0), S), v(2));
			assert_eq!(node.intersect(Cube::new(ivec3(0, 0, 0), S), &DRay::new(dvec3(-1.0, 0.5, 0.5), dvec3(1.0, 0.0, 0.0))), Some((v(2), 1.0)));
			assert_eq!(node.intersect(Cube::new(ivec3(0, 0, 0), S), &DRay::new(dvec3(-1.0, 0.5, 0.5), dvec3(-1.0, 0.0, 0.0))), None);
		}

		// uniform node at (16,0,0)
		{
			let mut node = Node::default();
			node.set_range(S, &cube((0, 0, 0), S), v(2));
			assert_eq!(
				node.intersect(Cube::new(ivec3(16, 0, 0), S), &DRay::new(dvec3(-1.0, 0.5, 0.5), dvec3(1.0, 0.0, 0.0))),
				Some((v(2), 17.0))
			);
			assert_eq!(node.intersect(Cube::new(ivec3(16, 0, 0), S), &DRay::new(dvec3(-1.0, 0.5, 0.5), dvec3(-1.0, 0.0, 0.0))), None);
		}

		// empty node should not intersect
		{
			let mut node = Node::default();
			node.set_range(S, &cube((0, 0, 0), S), v(0));
			assert_eq!(node.intersect(Cube::new(ivec3(0, 0, 0), S), &DRay::new(dvec3(-1.0, 0.5, 0.5), dvec3(1.0, 0.0, 0.0))), None);
			assert_eq!(node.intersect(Cube::new(ivec3(0, 0, 0), S), &DRay::new(dvec3(-1.0, 0.5, 0.5), dvec3(-1.0, 0.0, 0.0))), None);
		}
	}

	#[test]
	fn visit_nonempty_cubes() {
		// construct 8x8x8 cube
		const S: u32 = 8;
		let mut root = Node::default();

		root.set_range(S, &irange((0, 0, 1), (1, 1, 2)), v(1));
		root.set_range(S, &irange((2, 0, 4), (4, 2, 6)), v(2));

		let mut collect = Vec::new();
		root.visit_nonempty_cubes(ivec3(0, 0, 0), S, |range, value| collect.push((range, value)));
		assert_eq!(
			collect,
			vec![
				(crange((0, 0, 1), 1), v(1)), //
				(crange((2, 0, 4), 2), v(2)), //
			]
		);
	}

	#[test]
	fn simplify_1() {
		// construct 4x4x4 cube
		const S: u32 = 4;
		let mut root = Node::default();

		// fill it uniformly, bit by bit
		root.set_range(S, &irange((0, 0, 0), (4, 4, 2)), v(1));
		root.set_range(S, &irange((0, 0, 2), (4, 4, 4)), v(1));

		for pos in irange((0, 0, 0), (4, 4, 4)).iter() {
			assert_eq!(root.query_cube(S, &Cube::new(pos, 1)), CubeType::Uniform(v(1)));
		}

		assert_eq!(root.query_cube(S, &crange((0, 0, 0), 4)), CubeType::Uniform(v(1)));
	}

	#[test]
	fn simplify_2() {
		// construct 8x8x8 cube
		const S: u32 = 8;
		let mut root = Node::default();

		// fill it uniformly, voxel by voxel
		for pos in irange((0, 0, 0), (8, 8, 8)).iter() {
			root.set_range(S, &Cuboid::cube(pos, 1), VoxelType(1));
		}

		assert_eq!(root.query_cube(S, &crange((0, 0, 0), 8)), CubeType::Uniform(v(1)));
	}

	#[test]
	fn query_cube() {
		{
			// construct 2x2x2 cube filled with [1,2,3,4,5,6,7,8].
			const S: u32 = 2;
			let mut root = Node::default();
			root.set_range(S, &Cuboid::cube(ivec3(0, 0, 0), 1), VoxelType(1));
			root.set_range(S, &Cuboid::cube(ivec3(1, 0, 0), 1), VoxelType(2));
			root.set_range(S, &Cuboid::cube(ivec3(0, 1, 0), 1), VoxelType(3));
			root.set_range(S, &Cuboid::cube(ivec3(1, 1, 0), 1), VoxelType(4));
			root.set_range(S, &Cuboid::cube(ivec3(0, 0, 1), 1), VoxelType(5));
			root.set_range(S, &Cuboid::cube(ivec3(1, 0, 1), 1), VoxelType(6));
			root.set_range(S, &Cuboid::cube(ivec3(0, 1, 1), 1), VoxelType(7));
			root.set_range(S, &Cuboid::cube(ivec3(1, 1, 1), 1), VoxelType(8));
			assert_eq!(root, composite([1, 2, 3, 4, 5, 6, 7, 8]));

			// query the root
			assert_eq!(root.query_cube(S, &Cube::new(ivec3(0, 0, 0), 2)), CubeType::Mixed);

			// query children (exercises recursion).
			assert_eq!(root.query_cube(S, &crange((0, 0, 0), 1)), CubeType::Uniform(VoxelType(1)));
			assert_eq!(root.query_cube(S, &crange((1, 0, 0), 1)), CubeType::Uniform(VoxelType(2)));
			assert_eq!(root.query_cube(S, &crange((0, 1, 0), 1)), CubeType::Uniform(VoxelType(3)));
			assert_eq!(root.query_cube(S, &crange((1, 1, 0), 1)), CubeType::Uniform(VoxelType(4)));
			assert_eq!(root.query_cube(S, &crange((0, 0, 1), 1)), CubeType::Uniform(VoxelType(5)));
			assert_eq!(root.query_cube(S, &crange((1, 0, 1), 1)), CubeType::Uniform(VoxelType(6)));
			assert_eq!(root.query_cube(S, &crange((0, 1, 1), 1)), CubeType::Uniform(VoxelType(7)));
			assert_eq!(root.query_cube(S, &crange((1, 1, 1), 1)), CubeType::Uniform(VoxelType(8)));
		}

		{
			// construct 4x4x4 cube
			const S: u32 = 4;
			let mut root = Node::default();

			// Test some random sub-cubes
			assert_eq!(root.query_cube(S, &crange((0, 0, 0), 4)), CubeType::Uniform(VoxelType(0)));
			assert_eq!(root.query_cube(S, &crange((2, 0, 2), 2)), CubeType::Uniform(VoxelType(0)));
			assert_eq!(root.query_cube(S, &crange((0, 2, 2), 2)), CubeType::Uniform(VoxelType(0)));
			assert_eq!(root.query_cube(S, &crange((0, 0, 0), 1)), CubeType::Uniform(VoxelType(0)));
			assert_eq!(root.query_cube(S, &crange((0, 1, 2), 1)), CubeType::Uniform(VoxelType(0)));
			assert_eq!(root.query_cube(S, &crange((3, 3, 3), 1)), CubeType::Uniform(VoxelType(0)));
			assert_eq!(root.query_cube(S, &crange((3, 2, 1), 1)), CubeType::Uniform(VoxelType(0)));

			// Set a 2x2x2 sub-cube
			root.set_range(S, &Cuboid::cube(ivec3(2, 0, 2), 2), VoxelType(1));
			// root is now mixed
			assert_eq!(root.query_cube(S, &crange((0, 0, 0), 4)), CubeType::Mixed);
			// the cube we just set
			assert_eq!(root.query_cube(S, &crange((2, 0, 2), 2)), CubeType::Uniform(VoxelType(1)));
			// children of the cube we just set
			assert_eq!(root.query_cube(S, &crange((2, 0, 2), 1)), CubeType::Uniform(VoxelType(1)));
			assert_eq!(root.query_cube(S, &crange((3, 0, 2), 1)), CubeType::Uniform(VoxelType(1)));
			// others unaffected
			assert_eq!(root.query_cube(S, &crange((2, 0, 0), 2)), CubeType::Uniform(VoxelType(0)));
			assert_eq!(root.query_cube(S, &crange((0, 0, 0), 2)), CubeType::Uniform(VoxelType(0)));
			assert_eq!(root.query_cube(S, &crange((2, 2, 2), 2)), CubeType::Uniform(VoxelType(0)));
			assert_eq!(root.query_cube(S, &crange((2, 2, 2), 1)), CubeType::Uniform(VoxelType(0)));
			assert_eq!(root.query_cube(S, &crange((3, 3, 3), 1)), CubeType::Uniform(VoxelType(0)));
		}
	}

	#[test]
	fn regression_query_cube_64() {
		const S: u32 = 64;
		let mut root = Node::default();

		root.set_range(S, &cube((63, 1, 36), 1), v(42));
		assert_eq!(root.query_cube(S, &crange((63, 1, 36), 1)), CubeType::Uniform(VoxelType(42)));
	}

	#[test]
	// Test `set_range`, check internal representation.
	fn set_range() {
		{
			// set leaf
			let mut root = Node::default();
			root.set_range(1, &Cuboid::cube(ivec3(0, 0, 0), 1), VoxelType(42));
			assert_eq!(root, Uniform(VoxelType(42)));
		}

		{
			// set inner to uniform
			let mut root = Node::default();
			root.set_range(2, &Cuboid::cube(ivec3(0, 0, 0), 2), VoxelType(42));
			assert_eq!(root, Uniform(VoxelType(42)));
		}
		{
			// set inner to composite
			let mut root = Node::default();

			root.set_range(2, &Cuboid::cube(ivec3(0, 0, 0), 1), VoxelType(42));
			assert_eq!(root, composite([42, 0, 0, 0, 0, 0, 0, 0]));

			root.set_range(2, &Cuboid::cube(ivec3(1, 1, 1), 1), VoxelType(43));
			assert_eq!(root, composite([42, 0, 0, 0, 0, 0, 0, 43]));
		}
		{
			// set larger range to composite
			let mut root = Node::default();
			root.set_range(2, &Cuboid::new(ivec3(0, 0, 0), ivec3(2, 1, 1)), VoxelType(1));
			assert_eq!(root, composite([1, 1, 0, 0, 0, 0, 0, 0]));
			// TODO: test w/ query cube.
		}
		{
			// set larger range to composite
			let mut root = Node::default();
			root.set_range(2, &Cuboid::new(ivec3(0, 0, 0), ivec3(1, 2, 1)), VoxelType(1));
			assert_eq!(root, composite([1, 0, 1, 0, 0, 0, 0, 0]));
			// TODO: test w/ query cube.
		}
		{
			// set larger range to composite
			let mut root = Node::default();
			root.set_range(2, &Cuboid::new(ivec3(0, 0, 0), ivec3(1, 1, 2)), VoxelType(1));
			assert_eq!(root, composite([1, 0, 0, 0, 1, 0, 0, 0]));
			// TODO: test w/ query cube.
		}
	}
}
