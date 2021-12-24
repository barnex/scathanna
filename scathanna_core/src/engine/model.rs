use super::internal::*;

/// VertexArray + Material + metadata on the GPU.
/// Ready to be drawn by `Engine`.
pub struct Model {
	pub vao: Rc<VertexArray>,
	pub material: Material,
	pub double_sided: bool,
	pub lines: bool,
}

impl Model {
	pub fn new(vao: Rc<VertexArray>, material: Material) -> Self {
		Self {
			vao, //
			material,
			double_sided: false,
			lines: false,
		}
	}

	/// Create a Model with the same mesh but different material.
	pub fn with_material(&self, material: Material) -> Self {
		Self {
			vao: self.vao.clone(),
			material,
			double_sided: self.double_sided,
			lines: self.lines,
		}
	}

	/// Enable double-sided rendering (no backface culling).
	pub fn double_sided(mut self) -> Self {
		self.double_sided = true;
		self
	}

	/// Render lines instead of triangles
	pub fn with_lines(mut self) -> Self {
		self.lines = true;
		self
	}
}
