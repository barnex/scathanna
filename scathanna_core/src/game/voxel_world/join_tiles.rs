use super::internal::*;

// Fuse adjacent square tiles into larger rectangles (so that we will draw fewer faces).
// Uses a  greedy, but close to optimal, algorithm that first fuses horizontally adjacent tiles,
// then same-size vertically adjacent tiles.
//   +----+----+----+              +--------------+
//   |1x1 |    |    |              | 3x2          |
//   +----+----+----+              |              |
//   |    |    |    |        ->    |              |
//   +----+----+----+----+         +--------------+----+
//   |    |    |    |    |         | 4x1               |
//   +----+----+----+----+         +-------------------+
//
// Tiles must lie within one VoxelStore Cell at position `cell_pos`.
pub fn join_squares(squares: Vec<Rectangle>, cell_pos: ivec3) -> Vec<Rectangle> {
	let mut buf = TileBuffer::new();
	for s in squares.into_iter() {
		let size = {
			debug_assert!(s.size.x() == s.size.y());
			s.size.x()
		};
		buf.push_square((s.position - cell_pos, s.direction, size));
	}
	buf.join(cell_pos)
}

type Square = (ivec3, Direction, u32);

/// Temporary storage for (position- & axis-) aligned squares in a 3D space.
/// Upon `build()`, neighboring squares are merged into larger, equivalent faces
/// (which eventually reduces the number of triangles drawn).
#[derive(Default)]
struct TileBuffer {
	tiles: [[HashMap<i32, HashSet<Tile>>; 2]; 3], // squares indexed by axis, side, position along axis
}

/// A 2D square.
#[derive(PartialEq, Eq, Hash)]
struct Tile {
	corner: ivec2,
	size: u32,
}

const CS: usize = Voxels::CELL_SIZE as usize;

impl TileBuffer {
	fn new() -> Self {
		Self::default()
	}

	/// Add a square tile.
	/// Corner position must be aligned to `size`
	/// (E.g.: `corner_pos = (-4, 4, 8)`, `size = 4).
	fn push_square(&mut self, (pos, direction, size): Square) {
		debug_assert!(is_aligned_to(pos, size));

		let axis = direction.unoriented_axis();
		let normal = pos[axis];
		let corner = pos.remove(axis);

		self.tiles[axis][direction.side_index()].entry(normal).or_default().insert(Tile { corner, size });
	}

	/// Build an optimized mesh from the stored squares.
	fn join(self, pos_offset: ivec3) -> Vec<Rectangle> {
		let mut rectangles = Vec::new();
		for dir in Direction::ALL {
			for (normal_pos, squares) in &self.tiles[dir.unoriented_axis()][dir.side_index()] {
				Self::build_plane(&mut rectangles, squares, *normal_pos, dir, pos_offset);
			}
		}
		rectangles
	}

	/// Build an optimized mesh for the squares in one plane.
	/// Add result to `buf`.
	fn build_plane(buf: &mut Vec<Rectangle>, plane: &HashSet<Tile>, normal_pos: i32, dir: Direction, pos_offset: ivec3) {
		// Step 1: "rasterize" Tiles into 1x1 squares.
		// Store size of square(i,j) at index (i,j).
		// At this point all sizes are 1x1.
		let mut size_of_sq_at = Self::rasterize_to_1x1(plane);

		Self::merge_horizontal(&mut size_of_sq_at);
		Self::merge_vertical(&mut size_of_sq_at);

		// Step 3: convert back to 3D rectangles
		let axis = dir.unoriented_axis();
		for iy in 0..size_of_sq_at.len() {
			for ix in 0..size_of_sq_at[iy].len() {
				let size2d = size_of_sq_at[iy][ix];
				if size2d != uvec2(0, 0) {
					debug_assert!(size2d.x() > 0);
					debug_assert!(size2d.y() > 0);
					let pos2d = ivec2(ix as i32, iy as i32);
					let pos3d = pos2d.insert(axis, normal_pos) + pos_offset;
					buf.push(Rectangle {
						position: pos3d,
						direction: dir,
						size: size2d,
					});
				}
			}
		}
	}

	// "rasterize" Tiles into 1x1 squares
	// so that they can be easily fused into larger rectangles.
	//   +--------+----+              +----+----+----+
	//   |        |    | 0x0          |1x1 |1x1 |    | 0x0
	//   |  2x2   +----+----+   ->    +----+----+----+----+
	//   |        |    |    |         |1x1 |1x1 |    |    |
	//   +--------+----+----+         +----+----+----+----+
	fn rasterize_to_1x1(plane: &HashSet<Tile>) -> [[uvec2; CS]; CS] {
		let mut size_of_sq_at = [[uvec2(0, 0); CS]; CS];
		for Tile { corner, size, .. } in plane {
			let size = *size;
			for iy in 0..size {
				for ix in 0..size {
					let ix = ix + corner.x() as u32;
					let iy = iy + corner.y() as u32;
					debug_assert!(size_of_sq_at[iy as usize][ix as usize] == uvec2(0, 0)); // no overwrites please
					size_of_sq_at[iy as usize][ix as usize] = uvec2(1, 1);
				}
			}
		}
		size_of_sq_at
	}

	// Combine horizontally adjacent 1x1 tiles into longer strips.
	//
	// E.g.:
	// input (only 1x1 squares present):
	//   +---+---+---+---+---+---+---+---+---+
	//   |1x1|1x1|1x1|   |1x1|1x1|   |1x1|   |
	//   +---+---+---+---+---+---+---+---+---+
	//
	// output (longer strips present):
	//   +---+---+---+---+---+---+---+---+---+
	//   |3x1| . | . |   |2x1| . |   |1x1|   |
	//   +---+---+---+---+---+---+---+---+---+
	fn merge_horizontal(tiles: &mut [[uvec2; CS]; CS]) {
		for iy in 0..tiles.len() {
			let mut run_start: Option<usize> = None;
			for ix in 0..tiles.len() {
				match tiles[iy][ix].into() {
					(0, 0) => run_start = None,
					(1, 1) => match run_start {
						None => run_start = Some(ix),
						Some(start) => {
							tiles[iy][start][0] += 1;
							tiles[iy][ix] = uvec2(0, 0);
						}
					},
					_ => unreachable!(),
				}
			}
		}
	}

	// Combine vertically adjacent strips of the same length into larger rectangles.
	// In the input / output array, each element
	// signifies the size of the tile starting at that index.
	//
	// E.g.:
	// input (only horizontal strips present):
	//   +---+---+---+---+---+---+---+---+---+
	//   |3x1| . | . |   |2x1| . |   |   |   |
	//   +---+---+---+---+---+---+---+---+---+
	//   |3x1| . | . |   |1x1|   |   |1x1|   |
	//   +---+---+---+---+---+---+---+---+---+
	//
	// output:
	//   +---+---+---+---+---+---+---+---+---+
	//   |3x2| . | . |   |2x1| . |   |   |   |
	//   +---+---+---+---+---+---+---+---+---+
	//   | . | . | . |   |1x1|   |   |1x1|   |
	//   +---+---+---+---+---+---+---+---+---+
	//
	fn merge_vertical(tiles: &mut [[uvec2; CS]; CS]) {
		for ix in 0..tiles.len() {
			let mut run_start: Option<usize> = None;
			for iy in 0..tiles.len() {
				match tiles[iy][ix].into() {
					(0, 0) => run_start = None,
					(len, 1) => match run_start {
						None => run_start = Some(iy),
						Some(start) => {
							if len == tiles[start][ix].x() {
								// strip lengths match: remove this strip, increase the height of the current run.
								tiles[start][ix][1] += 1;
								tiles[iy][ix] = uvec2(0, 0);
							} else {
								// strip lengths don't match: cannot fuse, new run starts.
								run_start = Some(iy);
							}
						}
					},
					_ => unreachable!(),
				}
			}
		}
	}
}
