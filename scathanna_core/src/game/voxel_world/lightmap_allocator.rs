use super::internal::*;

pub struct LightmapAllocator {
	size: u32,
	curr: uvec2,
	next_y: u32,
}

/// Additional margin between islands.
/// An absolute minimum of 1 is needed to avoid light bleeding.
/// 1 additional texel is added to avoid minuscule light bleeding due to round-off under grazing incidence
/// (see fn VoxelWorld::add_borders).
const MARGIN: u32 = 2;

impl LightmapAllocator {
	pub fn new(size: u32) -> Self {
		Self { size, curr: uvec2(1, 1), next_y: 1 }
	}

	/// Call `alloc` on each face.
	/// Does not necessarily conserve the original ordering.
	pub fn alloc_all(&mut self, mut faces: Vec<Rectangle>) -> Vec<(Rectangle, uvec2)> {
		faces.sort_by_key(|r| r.size.y); // reduces lightmap fullness 3-10x
		faces
			.into_iter()
			.map(move |face| {
				let uv = self.alloc(face.size);
				(face, uv)
			})
			.collect()
	}

	/// Allocate a `(W+1)x(H+1)` island for a `WxH` face.
	///
	/// texel 0   1   2   3   4   5   6...
	/// .   0 .   .   .   .   .   .   .   
	/// .       +---+       +-------+     
	/// .   1 . | . | .   . | .   . | .   
	/// .       +---+       |       |     
	/// .   2 .   .   .   . | .   . | .   
	/// .                   +-------+        
	/// .   3 .   .   .   .   .   .   .   
	/// .                                 
	/// .   4 .   .   .   .   .   .   .   
	pub fn alloc(&mut self, size: uvec2) -> uvec2 {
		debug_assert!(size.x > 0 && size.y > 0);

		let size = size + uvec2(1, 1);

		if self.curr.x + size.x >= self.size {
			// next line
			self.curr.x = MARGIN;
			self.curr.y = self.next_y;
		}

		self.next_y = u32::max(self.next_y, self.curr.y + size.y + MARGIN);

		let result = self.curr;
		self.curr.x += size.x + MARGIN;
		result
	}

	/*
	pub fn fullness(&self) -> f32 {
		(self.curr.y as f32) / (self.size as f32)
	}
	*/
}
