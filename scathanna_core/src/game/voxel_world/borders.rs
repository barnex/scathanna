use super::internal::*;

/// Wireframe models that may be drawn over the voxels to highlight their edges,
/// resulting in comic book style graphics. One per cell, indexed by cell position.
pub fn highlighted_edges(engine: &Engine, voxels: &Voxels) -> HashMap<ivec3, Model> {
	// Cut edges into strands of length 1.
	// If the same strand (including normal direction) is present twice,
	// then both instances must come from adjacent faces (i.e. be "inner" edges)
	// and we don't want to render them.
	// We only want to see borders where the adjacent faces make a 90 degree angle.
	// This has to happen across all cells, else we would render borders at cell edge.
	//
	// E.g.: for the two adjacent cubes below, the edges marked with "X" would be removed.
	//
	//         +-----+-----+
	//        /     X     /|
	//       +-----+-----+ |
	//       |     X     | +
	//       |     X     |/
	//       +-----+-----+
	//
	let mut eliminate_inner = HashSet::default();
	for cell_pos in voxels.iter_cell_positions() {
		for (rect, _) in voxels.iter_faces(cell_pos) {
			for edge in edges(&rect) {
				for edge in edge.cut() {
					if eliminate_inner.contains(&edge) {
						eliminate_inner.remove(&edge);
					} else {
						eliminate_inner.insert(edge);
					}
				}
			}
		}
	}

	// At this point each of the remaining edges is present twice,
	// with different normal vectors. Deduplicate them, ignoring the normals.
	//
	//         +-----+-----+
	//        /           /|
	//       +-----+-----+ |
	//       |           | +
	//       |           |/
	//       +-----+-----+
	//
	let mut dedup = HashSet::default();
	for edge in eliminate_inner.into_iter() {
		let edge = Edge {
			normal: Direction::X, // Unused dummy
			..edge
		};
		dedup.insert(edge);
	}

	// Strands that meet up head-to-tail can now be combined
	// (but not across cells, which have to be renderable independently):
	//
	//         +-----------+
	//        /           /|
	//       +-----------+ |
	//       |           | +
	//       |           |/
	//       +-----------+
	//

	// Visit strands left-to-right so that we only have to join left's tail to right's head.
	let mut sorted = dedup.into_iter().collect::<Vec<_>>();
	sorted.sort_unstable_by_key(|e| (e.tangent, -e.start.x(), -e.start.y(), -e.start.z()));
	// Strands searchable by starting position, so that we can efficiently find neighbors.
	let mut by_start: HashMap<(Direction, ivec3), Edge> = HashMap::default();
	for edge in sorted.into_iter() {
		if let Some(after_me) = by_start.remove(&(edge.tangent, edge.end())) {
			by_start.insert((edge.tangent, edge.start), edge.join(after_me));
		} else {
			by_start.insert((edge.tangent, edge.start), edge);
		}
	}

	// Convert edges to a MeshBuffer per cell.
	let mut buf_by_cell: HashMap<ivec3, MeshBuffer> = HashMap::default();
	for (_, edge) in by_start.into_iter() {
		let cell_pos = edge.start.map(Voxels::align_down);
		let buf = buf_by_cell.entry(cell_pos).or_default();
		buf.push_position(edge.start.to_f32());
		buf.push_position(edge.end().to_f32());
	}

	// Build MeshBuffers into Models, still one per cell.
	let mut result = HashMap::default();
	for (cell_pos, buf) in buf_by_cell {
		if buf.len() != 0 {
			result.insert(cell_pos, Model::new(engine.build_vao(&buf), Material::UniformColor(BLACK)).with_lines());
		}
	}

	result
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
struct Edge {
	start: ivec3,
	len: u32,
	tangent: Direction,
	normal: Direction,
}

impl Edge {
	fn end(&self) -> ivec3 {
		self.start + self.tangent.ivec() * self.len as i32
	}

	// Cut this Edge into strands of length 1.
	fn cut(self) -> impl Iterator<Item = Edge> {
		let tan = self.tangent.ivec();
		(0..self.len).into_iter().map(move |t| Edge {
			start: self.start + (t as i32) * tan,
			len: 1,
			tangent: self.tangent,
			normal: self.normal,
		})
	}

	fn join(self, rhs: Self) -> Self {
		debug_assert!(self.end() == rhs.start);
		debug_assert!(self.tangent == rhs.tangent);
		Self {
			start: self.start,
			len: self.len + rhs.len,
			tangent: self.tangent,
			normal: self.normal,
		}
	}
}

fn edges(rect: &Rectangle) -> [Edge; 4] {
	let normal = rect.direction;
	let [du, dv] = rect.direction.tangents();
	let o = rect.position;
	[
		Edge {
			start: o,
			len: rect.size.x(),
			tangent: du,
			normal,
		},
		Edge {
			start: o + (dv.ivec() * rect.size.y() as i32),
			len: rect.size.x(),
			tangent: du,
			normal,
		},
		Edge {
			start: o,
			len: rect.size.y(),
			tangent: dv,
			normal,
		},
		Edge {
			start: o + (du.ivec() * rect.size.x() as i32),
			len: rect.size.y(),
			tangent: dv,
			normal,
		},
	]
}

/*
/// Borders for each visible voxel.
/// Handy to discern the voxel size and alignment,
/// but not otherwise useful for production rendering.
pub fn all_voxel_borders(engine: &mut Engine, voxels: &Voxels) -> HashMap<ivec3, Model> {
	let mut result = HashMap::default();
	for cell_pos in voxels.iter_cell_positions() {
		if let Some(borders) = cell_voxel_borders(engine, voxels, cell_pos) {
			result.insert(cell_pos, borders);
		}
	}
	result
}

fn cell_voxel_borders(engine: &mut Engine, voxels: &Voxels, cell_pos: ivec3) -> Option<Model> {
	let mut buf = MeshBuffer::new();
	for (rect, _) in voxels.iter_faces(cell_pos) {
		let vert = rect.vertices();
		for i in 0..4 {
			buf.push_vertex(vert[i].to_f32(), vec2::ZERO, vec3::ZERO);
			buf.push_vertex(vert[(i + 1) % 4].to_f32(), vec2::ZERO, vec3::ZERO);
		}
	}

	match buf.len() {
		0 => None,
		_ => Some(Model::new(engine.build_vao(&buf), Material::UniformColor(WHITE)).with_lines()),
	}
}
*/
