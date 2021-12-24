use super::internal::*;

pub struct CellBuffer {
	pub lightmap: Lightmap,
	pub models: Vec<(VoxelType, MeshBuffer)>,
}

/// Turn a VoxelStore cell into a set of meshes (lightmap + (voxel, MeshBuffer) pairs).
pub fn bake_cell_buffer(voxels: &Voxels, cell_pos: ivec3, quality: bool) -> Option<CellBuffer> {
	debug_assert!(Voxels::is_aligned(cell_pos));
	// Early return for empty cell avoids allocating an unused lightmap, etc.
	if voxels.cell_at(cell_pos).is_none() {
		return None;
	}

	let mapped_faces = map_faces(voxels, cell_pos);

	// Bake lightmap texture for this cell
	let mut lightmap = Lightmap::new(LIGHTMAP_SIZE);
	bake_lightmap(&mut lightmap, voxels, &mapped_faces, quality);

	let models = triangulate_faces(&mapped_faces);

	Some(CellBuffer { lightmap, models })
}

pub fn load_cell_buffer(voxels: &Voxels, cell_pos: ivec3, dir: &Path) -> Option<CellBuffer> {
	debug_assert!(Voxels::is_aligned(cell_pos));
	// Early return for empty cell avoids allocating an unused lightmap, etc.
	if voxels.cell_at(cell_pos).is_none() {
		return None;
	}

	let mapped_faces = map_faces(voxels, cell_pos);

	let img = match imageutil::load_rgb(VoxelWorld::lightmap_file(dir, cell_pos)) {
		Ok(img) => img,
		Err(e) => {
			eprintln!("ERROR loading lightmap {:?}: {}", dir, e);
			return bake_cell_buffer(voxels, cell_pos, false);
		}
	};

	let models = triangulate_faces(&mapped_faces);
	Some(CellBuffer {
		lightmap: Lightmap::from_rgb(&img),
		models,
	})
}

// Add lightmap coordinates to faces.
fn map_faces(voxels: &Voxels, cell_pos: ivec3) -> HashMap<VoxelType, Vec<(Rectangle, uvec2)>> {
	// faces with added lightmap coordinates
	let mut lm_pack = LightmapAllocator::new(LIGHTMAP_SIZE);

	// faces indexed by voxel type and simplified where possible.
	let faces_by_voxel = simplified_faces_by_voxel(voxels, cell_pos);
	faces_by_voxel //
		.map(|(v, faces)| (v, lm_pack.alloc_all(faces)))
		.collect()
}

fn triangulate_faces(mapped_faces: &HashMap<VoxelType, Vec<(Rectangle, uvec2)>>) -> Vec<(VoxelType, MeshBuffer)> {
	mapped_faces //
		.iter()
		.map(|(v, faces)| (*v, triangulate_all(faces)))
		.collect::<Vec<_>>()
}

// faces indexed by voxel type and simplified where possible.
fn simplified_faces_by_voxel(voxels: &Voxels, cell_pos: ivec3) -> impl Iterator<Item = (VoxelType, Vec<Rectangle>)> {
	voxels //
		.collect_faces_by_voxel(cell_pos)
		.into_iter()
		.map(move |(voxel, faces)| (voxel, join_squares(faces, cell_pos)))
}

/// Convert 2D tiles to a 3D, triangulated, mesh
/// complete with texture coordinates.
///   +---+       *---*
///   |1x3|       |\  |
///   |   |  =>   | \ |
///   |   |       |  \|
///   +---+       +---*
pub fn triangulate_all(mapped_rectangles: &[(Rectangle, uvec2)]) -> MeshBuffer {
	// TODO: this pushes unused vertex normals.
	// remove from this push and from voxel shader?.
	// (or use normals for lightmap reflections).
	let mut buf = MeshBuffer::new();
	for face in mapped_rectangles {
		buf.push_quad_lightcoords(face_to_quad(face))
	}
	buf
}

fn face_to_quad((rect, lm_idx): &(Rectangle, uvec2)) -> [(vec3, vec2, vec3, vec2); 4] {
	const TEXTURE_PITCH: f32 = 1.0 / 16.0;

	let axis = rect.direction.unoriented_axis();
	let side = rect.direction.side_index();
	let size3d = rect.size.to_i32().insert(axis, 1);
	let pos3d = rect.position;
	let size2d = rect.size;
	let pos2d = pos3d.remove(axis);

	// A face's lightmap coordinates are offset by 0.5 texels with respect to their island
	// (the island being 1 texel bigger than the face).
	// This ensures that during rendering, texture lookups always stay at least 0.5 texels from
	// the island border so that we get no shadow bleeding.
	const LM_UV_OFFSET: vec2 = vec2(0.5, 0.5);

	// scale and translate a unit square to the needed size and position.
	UNIT_SQUARES[axis][side].map(|(pos, tex)| {
		(
			(pos.mul3(size3d) + pos3d).to_f32(),                                                     // pos
			(tex.mul2(size2d.to_f32()) + pos2d.to_f32()) * TEXTURE_PITCH,                            // tex
			rect.direction.ivec().to_f32(),                                                          // normal
			(lm_idx.to_f32() + (tex).mul2(size2d.to_f32()) + LM_UV_OFFSET) / (LIGHTMAP_SIZE as f32), // light coord
		)
	})
}

// 1x1 Squares oriented along +x,-x,+y,-y,+z,-z:
// array of (vertex position, UV, normal), indexed by axis, side, vertex.
const UNIT_SQUARES: [[[(ivec3, vec2); 4]; 2]; 3] = [
	[
		// x, left-facing
		[
			(ivec3(0, 0, 0), vec2(0.0, 0.0)), //
			(ivec3(0, 0, 1), vec2(0.0, 1.0)), //
			(ivec3(0, 1, 1), vec2(1.0, 1.0)), //
			(ivec3(0, 1, 0), vec2(1.0, 0.0)), //
		],
		// x, right-facing
		[
			(ivec3(0, 0, 0), vec2(0.0, 0.0)), //
			(ivec3(0, 1, 0), vec2(1.0, 0.0)), //
			(ivec3(0, 1, 1), vec2(1.0, 1.0)), //
			(ivec3(0, 0, 1), vec2(0.0, 1.0)), //
		],
	],
	[
		// y, bottom-facing
		[
			(ivec3(0, 0, 0), vec2(0.0, 0.0)), //
			(ivec3(1, 0, 0), vec2(1.0, 0.0)), //
			(ivec3(1, 0, 1), vec2(1.0, 1.0)), //
			(ivec3(0, 0, 1), vec2(0.0, 1.0)), //
		],
		// y, top-facing
		[
			(ivec3(0, 0, 0), vec2(0.0, 0.0)), //
			(ivec3(0, 0, 1), vec2(0.0, 1.0)), //
			(ivec3(1, 0, 1), vec2(1.0, 1.0)), //
			(ivec3(1, 0, 0), vec2(1.0, 0.0)), //
		],
	],
	[
		// z, back-facing
		[
			(ivec3(0, 0, 0), vec2(0.0, 0.0)), //
			(ivec3(0, 1, 0), vec2(0.0, 1.0)), //
			(ivec3(1, 1, 0), vec2(1.0, 1.0)), //
			(ivec3(1, 0, 0), vec2(1.0, 0.0)), //
		],
		// z, front-facing
		[
			(ivec3(0, 0, 0), vec2(0.0, 0.0)), //
			(ivec3(1, 0, 0), vec2(1.0, 0.0)), //
			(ivec3(1, 1, 0), vec2(1.0, 1.0)), //
			(ivec3(0, 1, 0), vec2(0.0, 1.0)), //
		],
	],
];
