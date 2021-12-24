/*
use super::{internal::*, meshbuffer};

/// CPU counterpart of Model.
/// Used to construct a Model (possibly off the GL thread)
/// before uploading it to the GPU.
/// TODO: currently for voxel models only
pub struct VoxelModelBuffer {
	pub meshbuffer: MeshBuffer,
	pub texture: Image<Color>,
	pub double_sided: bool,
}

impl ModelBuffer {
	pub fn new(meshbuffer: MeshBuffer, texture: Image<Color>) -> Self {
		Self {
			meshbuffer, //
			texture,
			double_sided: false,
		}
	}

	/// Enable double-sided rendering (no backface culling).
	pub fn double_sided(mut self) -> Self {
		self.double_sided = true;
		self
	}

	/// Upload contents to the GPU.
	pub fn build(self, ctx: &Context) -> Model {
		let NO_MIPMAP = 1; // TODO: TextureBuffer / MaterialBuffer
		Model {
			vao: Rc::new(self.meshbuffer.build(ctx)),
			material: Material::Lightmap(
				texture_from_image(ctx, NO_MIPMAP, &self.texture),
			),
			double_sided: self.double_sided,
		}
	}
}
*/
