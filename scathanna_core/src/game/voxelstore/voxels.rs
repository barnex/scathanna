use super::internal::*;

/// Efficiently stores voxels (handles sparsity, large uniform regions).
/// See `VoxelWorld` for a wrapper that can also draw the voxels.
#[derive(Default)]
pub struct Voxels {
	// Arc allows for copy-on-write clones used during async lightmap baking.
	cells: HashMap<ivec3, Arc<Node>>,
}

impl Voxels {
	pub const LOG_CELL_SIZE: u32 = 6;
	pub const CELL_SIZE: u32 = 1 << Self::LOG_CELL_SIZE;
	pub const ICELL_SIZE: i32 = Self::CELL_SIZE as i32;
	pub const CELL_MASK: i32 = !((1 << (Self::LOG_CELL_SIZE)) - 1);

	pub fn new() -> Self {
		Self::default()
	}

	/// Efficiently set the type of voxels inside `range`.
	/// Large, aligned, cubic sub-regions are handled efficiently.
	pub fn set_range(&mut self, range: &Cuboid, value: VoxelType) {
		let hull = Self::aligned_hull(range);
		for cell_pos in hull.iter_by(Self::CELL_SIZE) {
			let cell_range = Cuboid::cube(cell_pos, Self::CELL_SIZE);
			let overlap = range.intersect(&cell_range);
			debug_assert!(!overlap.is_empty());
			let internal_range = overlap.translate(-cell_pos);
			// modify Node in-place unless currently in use for ray-tracing.
			// In that case, make a clone.
			// This might do a tiny bit of redundant cloning if the Arc<Node>
			// has already been sent out for ray tracing but tracing has not yet started.
			let cow = Arc::make_mut(self.cells.entry(cell_pos).or_default());
			cow.set_range(Self::CELL_SIZE, &internal_range, value)
		}
	}

	pub fn set_voxel(&mut self, pos: ivec3, value: VoxelType) {
		self.set_range(&Cuboid::cube(pos, 1), value)
	}

	/// A copy-on-write clone.
	pub fn cow_clone(&self) -> Self {
		Self {
			cells: self.cells.iter().map(|(k, v)| (*k, Arc::clone(v))).collect(),
		}
	}

	pub fn at(&self, pos: ivec3) -> VoxelType {
		match self.query_cube(Cube::new(pos, 1)) {
			CubeType::Uniform(v) => v,
			_ => unreachable!(),
		}
	}

	pub fn iter_cell_positions(&self) -> impl Iterator<Item = ivec3> + '_ {
		self.cells.keys().copied()
	}

	/// Summarized contents of a cube:
	///   - Uniform (filled with one single voxel type) or
	///   - Mixed (filled with different voxel types).
	fn query_cube(&self, cube: Cube) -> CubeType {
		if cube.size() > Self::CELL_SIZE {
			panic!("VoxelStore::query_cube: size too large: {}, max={}", cube.size(), Self::CELL_SIZE);
		}

		let cell_pos = cube.position().map(Self::align_down);
		match self.cells.get(&cell_pos) {
			None => CubeType::Uniform(VoxelType::EMPTY),
			Some(node) => node.query_cube(Self::CELL_SIZE, &cube.translate(-cell_pos)),
		}
	}

	pub fn collect_nonempty_cubes(&self, cell_pos: ivec3) -> Vec<(Cube, VoxelType)> {
		let mut collect = Vec::new();
		self.visit_nonempty_cubes(cell_pos, |cube, voxel| collect.push((cube, voxel)));
		collect
	}

	pub fn iter_nonempty_cubes(&self, cell_pos: ivec3) -> impl Iterator<Item = (Cube, VoxelType)> {
		self.collect_nonempty_cubes(cell_pos).into_iter()
	}

	pub fn visit_nonempty_cubes<F: FnMut(Cube, VoxelType)>(&self, cell_pos: ivec3, f: F) {
		if let Some(node) = self.cell_at(cell_pos) {
			node.visit_nonempty_cubes(cell_pos, Self::CELL_SIZE, f)
		}
	}

	pub fn iter_faces(&self, cell_pos: ivec3) -> impl Iterator<Item = (Rectangle, VoxelType)> {
		self.collect_faces(cell_pos).into_iter()
	}

	pub fn collect_faces(&self, cell_pos: ivec3) -> Vec<(Rectangle, VoxelType)> {
		let mut collect = Vec::new();
		self.visit_faces(cell_pos, |rect, voxel| collect.push((rect, voxel)));
		collect
	}

	pub fn collect_faces_by_voxel(&self, cell_pos: ivec3) -> HashMap<VoxelType, Vec<Rectangle>> {
		let mut map = HashMap::<VoxelType, Vec<Rectangle>>::default();
		self.visit_faces(cell_pos, |rect, voxel| map.entry(voxel).or_default().push(rect));
		map
	}

	/// Visit all visible cube faces.
	/// Fully occluded faces are not visited. E.g.:
	///
	///   +------+------
	///   |✓    ✗|✗
	///   |✓    ✗|✗
	///   +------+------
	///
	/// Partially occluded faces are split into smaller ones,
	/// and only the visible ones are added.
	///
	///   +---------+
	///   |        ✓|
	///   |         +----
	///   |        ✗|✗
	///   +---------+----
	///
	pub fn visit_faces<F>(&self, cell_pos: ivec3, mut f: F)
	where
		F: FnMut(Rectangle, VoxelType),
	{
		self.visit_nonempty_cubes(cell_pos, |cube, voxel| {
			use Direction::*;
			for dir in [X, Y, Z, MinusX, MinusY, MinusZ] {
				self.visit_faces_rec(cell_pos, dir, &cube, voxel, &mut f)
			}
		});
	}

	fn visit_faces_rec<F: FnMut(Rectangle, VoxelType)>(&self, cell_pos: ivec3, dir: Direction, cube: &Cube, voxel: VoxelType, f: &mut F) {
		/// Out of the 8 child cubes, return the 4 of them on one particular side
		/// (E.g. the left side == axis X, side L)
		/// Used to subdivide a face that is partially occluded by a CubeType::Mixed neighbor.
		fn children_on_side(dir: Direction) -> [ivec3; 4] {
			[(0, 0), (0, 1), (1, 0), (1, 1)].map(|v| ivec2::from(v).insert(dir.unoriented_axis(), dir.side_index() as i32))
		}

		let neigbour_cube = cube.translate(dir.ivec() * cube.size() as i32);
		match self.query_cube(neigbour_cube) {
			CubeType::Uniform(VoxelType::EMPTY) => {
				let face = Rectangle {
					position: cube.position() + dir.unoriented_vec() * dir.side_index() as i32 * (cube.size() as i32),
					size: uvec2(cube.size(), cube.size()),
					direction: dir,
				};
				f(face, voxel);
			}
			CubeType::Mixed => {
				for delta in children_on_side(dir) {
					let size = cube.size() / 2;
					let cube = Cube::new(cube.position() + delta * size as i32, size);
					self.visit_faces_rec(cell_pos, dir, &cube, voxel, f)
				}
			}
			CubeType::Uniform(_) => (/*face fully occluded by neighbor, so don't visit*/),
		}
	}

	/// Aligned cuboid fully enclosing `range`.
	pub fn aligned_hull(range: &Cuboid) -> Cuboid {
		let min = range.min.map(Self::align_down);
		let max = range.max.map(Self::align_up);

		if range.is_empty() {
			Cuboid::new(min, min) // empty
		} else {
			Cuboid::new(min, max)
		}
	}

	/// Align to CELL_SIZE (downwards).
	pub fn align_down(i: i32) -> i32 {
		i & Self::CELL_MASK
	}

	/// Align to CELL_SIZE, upwards.
	pub fn align_up(i: i32) -> i32 {
		if Self::is_alignedi(i) {
			i
		} else {
			Self::align_down(i + Self::ICELL_SIZE)
		}
	}

	/// Is position aligned to CELL_SIZE?.
	pub fn is_aligned(pos: ivec3) -> bool {
		pos.map(Self::align_down) == pos
	}

	fn is_alignedi(i: i32) -> bool {
		Self::align_down(i) == i
	}

	pub fn cell_at(&self, pos: ivec3) -> Option<Arc<Node>> {
		debug_assert!(Self::is_aligned(pos));
		self.cells.get(&pos).map(Arc::clone)
	}

	pub fn set_cell(&mut self, pos: ivec3, cell: Arc<Node>) {
		debug_assert!(Self::is_aligned(pos));
		self.cells.insert(pos, cell);
	}

	/// Intersect ray with voxels, return voxel type and intersection distance along ray.
	pub fn intersect(&self, ray: &DRay) -> Option<(VoxelType, f64)> {
		let cell_pos = ray
			.start //
			.map(|v| v as i32)
			.map(Self::align_down);

		let mut result = None;
		for dx in -2..=2 {
			for dy in -2..=2 {
				for dz in -2..=2 {
					let cell_pos = cell_pos + Self::ICELL_SIZE * ivec3(dx, dy, dz);
					if let Some(node) = self.cell_at(cell_pos) {
						result = frontmost(result, node.intersect(Cube::new(cell_pos, Self::CELL_SIZE), ray))
					}
				}
			}
		}
		result
	}

	pub fn intersects(&self, ray: &DRay) -> bool {
		let cell_pos = ray
			.start //
			.map(|v| v as i32)
			.map(Self::align_down);

		for dx in [0, -1, 1] {
			for dy in [0, -1, 1] {
				for dz in [0, -1, 1] {
					let cell_pos = cell_pos + Self::ICELL_SIZE * ivec3(dx, dy, dz);
					if let Some(node) = self.cell_at(cell_pos) {
						//result = frontmost(result, node.intersect(Cube::new(cell_pos, Self::CELL_SIZE), ray))
						if node.intersects(Cube::new(cell_pos, Self::CELL_SIZE), ray) {
							return true;
						}
					}
				}
			}
		}
		return false;
	}

	pub fn intersection_point(&self, ray: &DRay) -> Option<vec3> {
		self.intersect(&ray).map(|(_, t)| ray.at(t - 1e-3)).map(|v| v.map(|v| v as f32))
	}

	/// Voxel position where `ray` intersects the world.
	/// Pass a small forward or backward `offset` to select the voxel right before or right behind the intersection.
	pub fn intersect_voxel(&self, ray: &DRay, offset: f64) -> Option<ivec3> {
		self.intersect(&ray).map(|(_, t)| Self::round_to_voxel(ray.at(t + offset)))
	}

	fn round_to_voxel(pos: dvec3) -> ivec3 {
		pos.map(|v| v.floor() as i32)
	}

	// _______________________ I/O ___________________________

	/// Serialize in gzipped bincode
	pub fn serialize<W: Write>(&self, w: W) -> Result<()> {
		let gz = GzEncoder::new(w, flate2::Compression::best());
		Ok(bincode::serialize_into(gz, self)?)
	}

	/// Deserialize from gzipped bincode
	pub fn deserialize<R: Read>(r: R) -> Result<Self> {
		let gz = GzDecoder::new(r);
		Ok(bincode::deserialize_from(gz)?)
	}

	/// Serialize to file, `.bincode.gz`
	pub fn save<P: AsRef<Path>>(&self, fname: P) -> Result<()> {
		self.serialize(create(fname.as_ref())?)
	}

	/// Deserialize from file, `.bincode.gz`
	pub fn load<P: AsRef<Path>>(fname: P) -> Result<Self> {
		Self::deserialize(open(fname.as_ref())?)
	}

	/// Serialize to bytes, gzipped bincode.
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut buf = Vec::new();
		self.serialize(&mut buf).unwrap();
		buf
	}

	/// Deserialize from bytes, gzipped bincode.
	pub fn from_bytes(map_data: &[u8]) -> Result<Self> {
		Self::deserialize(std::io::Cursor::new(map_data))
	}

	pub fn bumps(&self, bounds: &BoundingBox<f32>) -> bool {
		let imin = bounds.min.map(f32::floor).to_ivec();
		let imax = bounds.max.map(f32::ceil).to_ivec();

		for iz in imin.z..=imax.z {
			for iy in imin.y..=imax.y {
				for ix in imin.x..=imax.x {
					let pos = ivec3(ix, iy, iz);
					if !self.at(pos).is_empty() {
						return true;
					}
				}
			}
		}
		false
	}
}

impl Serialize for Voxels {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
		// Arc<Node> does not easily serialize.
		// Just copy out the data.
		let mut data = HashMap::default();
		for (cell_pos, cell) in &self.cells {
			data.insert(cell_pos, cell.as_ref().clone());
		}
		data.serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for Voxels {
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let data = HashMap::deserialize(deserializer)?;
		let mut voxels = Self::new();
		for (cell_pos, cell) in data {
			voxels.cells.insert(cell_pos, Arc::new(cell));
		}
		Ok(voxels)
	}
}

//______________________________________________________________________________________________ tests

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn intersect() {
		let mut s = Voxels::new();

		s.set_range(&Cuboid::cube(ivec3(0, 0, 0), 1), VoxelType(2));
		assert_eq!(s.intersect(&DRay::new(dvec3(-1.0, 0.5, 0.5), dvec3(1.0, 0.0, 0.0))), Some((VoxelType(2), 1.0)));

		s.set_range(&Cuboid::cube(ivec3(0, -10, 0), 1), VoxelType(2));
		assert_eq!(s.intersect(&DRay::new(dvec3(0.5, -7.0, 0.5), dvec3(0.0, -1.0, 0.0))), Some((VoxelType(2), 2.0)));
	}

	fn sort_by<T, F>(mut v: Vec<T>, compare: F) -> Vec<T>
	where
		F: Fn(&T, &T) -> std::cmp::Ordering,
	{
		v.sort_by(compare);
		v
	}

	/*
	#[test]
	fn faces() {
		use Direction::*;

		let cmp_faces = |a: &(Rectangle, VoxelType), b: &(Rectangle, VoxelType)| {
			(a.0.position.x, a.0.position.y, a.0.position.z, a.0.size.x, a.0.size.y, a.0.direction as usize, a.1 .0).cmp(&(
				b.0.position.x,
				b.0.position.y,
				b.0.position.z,
				b.0.size.x,
				b.0.size.y,
				b.0.direction as usize,
				b.1 .0,
			))
		};

		let mut store = VoxelStore::new();
		store.set_range(&Cuboid::cube(ivec3(0, 0, 0), 2), VoxelType(1));

		let cell_pos = ivec3(0, 0, 0);
		let got = sort_by(store.collect_faces(cell_pos), cmp_faces);

		let want = sort_by(
			vec![
				(Rectangle::new((0, 0, 0), (2, 2), MinusX), VoxelType(1)), //
				(Rectangle::new((2, 0, 0), (2, 2), X), VoxelType(1)),      //
				(Rectangle::new((0, 0, 0), (2, 2), MinusY), VoxelType(1)), //
				(Rectangle::new((0, 2, 0), (2, 2), X), VoxelType(1)),      //
				(Rectangle::new((0, 0, 0), (2, 2), MinusZ), VoxelType(1)), //
				(Rectangle::new((0, 0, 2), (2, 2), Z), VoxelType(1)),      //
			],
			cmp_faces,
		);

		assert_eq!(got, want);
	}
	*/

	#[test]
	fn nonempty_cubes() {
		let mut store = Voxels::new();
		store.set_range(&Cuboid::cube(ivec3(8, 4, -4), 2), VoxelType(1));
		store.set_range(&Cuboid::cube(ivec3(2, 4, -4), 1), VoxelType(2));

		let cmp_cubes = |a: &(Cube, VoxelType), b: &(Cube, VoxelType)| {
			(a.0.position().x, a.0.position().y, a.0.position().z, a.0.size(), a.1 .0).cmp(&(b.0.position().x, b.0.position().y, b.0.position().z, b.0.size(), b.1 .0))
		};

		let cell_pos = ivec3(0, 0, -Voxels::ICELL_SIZE);
		let got = sort_by(store.collect_nonempty_cubes(cell_pos), cmp_cubes);
		let want = sort_by(
			vec![
				(cube((8, 4, -4), 2), VoxelType(1)), //
				(cube((2, 4, -4), 1), VoxelType(2)), //
			],
			cmp_cubes,
		);

		assert_eq!(got, want);
	}

	fn at(store: &Voxels, pos: ivec3) -> VoxelType {
		match store.query_cube(Cube::new(pos, 1)) {
			CubeType::Uniform(v) => v,
			_ => unreachable!(),
		}
	}

	fn set(store: &mut Voxels, pos: ivec3, v: VoxelType) {
		store.set_range(&Cuboid::cube(pos, 1), v)
	}

	fn range(min: (i32, i32, i32), max: (i32, i32, i32)) -> Cuboid {
		Cuboid::new(min.into(), max.into())
	}

	fn cube(pos: (i32, i32, i32), size: u32) -> Cube {
		Cube::new(pos.into(), size)
	}

	#[test]
	fn set_and_query() {
		let mut store = Voxels::new();
		store.set_range(&range((0, 0, 0), (1, 1, 1)), VoxelType(1));

		assert_eq!(store.query_cube(cube((0, 0, 0), 1)), CubeType::Uniform(VoxelType(1)));
	}

	#[test]
	fn set_and_query_1() {
		let mut store = Voxels::new();

		assert_eq!(at(&store, ivec3(0, 0, 0)), VoxelType::EMPTY);
		assert_eq!(at(&store, ivec3(-1, -1, -1)), VoxelType::EMPTY);
		assert_eq!(at(&store, ivec3(100, 100, 100)), VoxelType::EMPTY);
		assert_eq!(at(&store, ivec3(-100, -100, -100)), VoxelType::EMPTY);
		assert_eq!(at(&store, ivec3(-1, 1, 100)), VoxelType::EMPTY);

		set(&mut store, ivec3(0, 0, 0), VoxelType(1));
		assert_eq!(at(&store, ivec3(0, 0, 0)), VoxelType(1));
		assert_eq!(at(&store, ivec3(-1, -1, -1)), VoxelType::EMPTY);
		assert_eq!(at(&store, ivec3(100, 100, 100)), VoxelType::EMPTY);
		assert_eq!(at(&store, ivec3(-100, -100, -100)), VoxelType::EMPTY);
		assert_eq!(at(&store, ivec3(-1, 1, 100)), VoxelType::EMPTY);

		set(&mut store, ivec3(-1, 1, 100), VoxelType(2));
		assert_eq!(at(&store, ivec3(-1, 1, 100)), VoxelType(2));
		assert_eq!(at(&store, ivec3(0, 0, 0)), VoxelType(1));
		assert_eq!(at(&store, ivec3(-1, -1, -1)), VoxelType::EMPTY);
		assert_eq!(at(&store, ivec3(100, 100, 100)), VoxelType::EMPTY);
		assert_eq!(at(&store, ivec3(-100, -100, -100)), VoxelType::EMPTY);

		set(&mut store, ivec3(-1000, 1000, 100), VoxelType(3));
		assert_eq!(at(&store, ivec3(-1000, 1000, 100)), VoxelType(3));
		assert_eq!(at(&store, ivec3(-1, 1, 100)), VoxelType(2));
		assert_eq!(at(&store, ivec3(0, 0, 0)), VoxelType(1));
		assert_eq!(at(&store, ivec3(-1, -1, -1)), VoxelType::EMPTY);
		assert_eq!(at(&store, ivec3(100, 100, 100)), VoxelType::EMPTY);
		assert_eq!(at(&store, ivec3(-100, -100, -100)), VoxelType::EMPTY);
	}

	#[test]
	fn set_range() {
		let mut store = Voxels::new();
		store.set_range(&Cuboid::new(ivec3(1, 2, 3), ivec3(2, 4, 6)), VoxelType(1));

		let at = |x, y, z| at(&store, ivec3(x, y, z)).0;
		assert_eq!(at(-1, -1, -1), 0);
		assert_eq!(at(0, 0, 0), 0);
		assert_eq!(at(1, 2, 3), 1);
		assert_eq!(at(1, 3, 3), 1);
		assert_eq!(at(1, 2, 4), 1);
		assert_eq!(at(1, 3, 4), 1);
		assert_eq!(at(1, 2, 5), 1);
		assert_eq!(at(1, 3, 5), 1);
		assert_eq!(at(2, 2, 3), 0);
		assert_eq!(at(1, 3, 3), 1);
		assert_eq!(at(1, 2, 6), 0);
	}

	#[test]
	fn test_aligned_hull() {
		let numbers = [-1025, -1024, -1023, -65, -64, -63, 31, 32, 33, 63, 64, 65, 511, 512, 513];

		for (iz, &z1) in numbers.iter().enumerate() {
			for (iy, &y1) in numbers.iter().enumerate() {
				for (ix, &x1) in numbers.iter().enumerate() {
					for &z2 in &numbers[iz..] {
						for &y2 in &numbers[iy..] {
							for &x2 in &numbers[ix..] {
								let range = Cuboid::new(ivec3(x1, y1, z1), ivec3(x2, y2, z2));
								let hull = Voxels::aligned_hull(&range);
								if range.is_empty() {
									assert!(hull.is_empty())
								} else {
									assert!(hull.contains_range(&range));
								}
							}
						}
					}
				}
			}
		}
	}
}
