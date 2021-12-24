use super::internal::*;

/// Temporary storage for building a VertexArray.
/// Vertex attributes are indexed as per the `ATTRIB_*` constants in `Shaders`.
/// Vertex positions are mandatory, all other attributes are optional and lazily initialized on first push.
#[derive(Default)]
pub struct MeshBuffer {
	pub positions: Vec<vec3>,
	pub texcoords: Option<Vec<vec2>>,
	pub lightcoords: Option<Vec<vec2>>,
	pub normals: Option<Vec<vec3>>,
}

impl MeshBuffer {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_positions(positions: Vec<vec3>) -> Self {
		Self { positions, ..default() }
	}

	pub fn with_texcoords(mut self, texcoords: Vec<vec2>) -> Self {
		self.texcoords = Some(texcoords);
		self
	}

	pub fn with_normals(mut self, normals: Vec<vec3>) -> Self {
		self.normals = Some(normals);
		self
	}

	pub fn push_position(&mut self, position: vec3) {
		self.positions.push(position);
	}

	pub fn push_vertex(&mut self, position: vec3, texcoord: vec2, normal: vec3) {
		debug_assert!(self.lightcoords.is_none());
		self.positions.push(position);
		self.texcoords_mut().push(texcoord);
		self.normals_mut().push(normal);
	}

	pub fn push_vertex_lightcoords(&mut self, position: vec3, texcoord: vec2, normal: vec3, lightcoord: vec2) {
		self.positions.push(position);
		self.texcoords_mut().push(texcoord);
		self.normals_mut().push(normal);
		self.lightcoords_mut().push(lightcoord);
	}

	pub fn push_quad(&mut self, vertices: [(vec3, vec2, vec3); 4]) {
		debug_assert!(self.lightcoords.is_none());
		for i in [0, 1, 2, 2, 3, 0] {
			self.push_vertex(vertices[i].0, vertices[i].1, vertices[i].2);
		}
	}

	pub fn push_quad_lightcoords(&mut self, vertices: [(vec3, vec2, vec3, vec2); 4]) {
		for i in [0, 1, 2, 2, 3, 0] {
			self.push_vertex_lightcoords(vertices[i].0, vertices[i].1, vertices[i].2, vertices[i].3);
		}
	}

	pub fn append(&mut self, other: MeshBuffer) {
		let mut other = other;
		self.positions.append(&mut other.positions);
		if other.normals.is_some() {
			self.normals_mut().append(other.normals_mut());
		}
		if other.texcoords.is_some() {
			self.texcoords_mut().append(other.texcoords_mut());
		}
		if other.lightcoords.is_some() {
			self.lightcoords_mut().append(other.lightcoords_mut());
		}
	}

	pub fn positions(&self) -> &[vec3] {
		&self.positions
	}

	pub fn normals(&self) -> &[vec3] {
		&self.normals.as_ref().expect("MeshBuffer: normals enabled")
	}

	pub fn normals_mut(&mut self) -> &mut Vec<vec3> {
		Self::ensure_some(&mut self.normals)
	}

	pub fn texcoords(&self) -> &[vec2] {
		&self.texcoords.as_ref().expect("MeshBuffer: texture coordinates enabled")
	}

	pub fn texcoords_mut(&mut self) -> &mut Vec<vec2> {
		Self::ensure_some(&mut self.texcoords)
	}

	pub fn lightcoords(&self) -> Option<&[vec2]> {
		self.lightcoords.as_deref()
	}

	pub fn lightcoords_mut(&mut self) -> &mut Vec<vec2> {
		Self::ensure_some(&mut self.lightcoords)
	}

	fn ensure_some<T>(v: &mut Option<Vec<T>>) -> &mut Vec<T> {
		if v.is_none() {
			*v = Some(Vec::new());
		}
		v.as_mut().unwrap()
	}

	pub fn len(&self) -> usize {
		self.positions.len()
	}
}
