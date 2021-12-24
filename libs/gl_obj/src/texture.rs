use std::{marker::PhantomData, rc::Rc};

use super::*;
use gl_safe::*;

pub struct Texture {
	handle: GLuint,
	internalformat: GLenum,
	size: uvec3,
	//dimensionality: u8,

	// OpenGL functions may only be called from the thread that was used to initialize GL.
	// Therefore, mark as !Send, !Sync to avoid accidental access from other threads
	// (this would segfault).
	not_send: PhantomData<Rc<()>>,
}

// TODO: when stable:
//impl !Sync for Texture {}

impl Texture {
	/// Create a buffer object.
	/// http://docs.gl/gl4/glCreateTextures
	pub fn create(target: GLenum) -> Self {
		Self {
			handle: glCreateTexture(target),
			internalformat: 0,
			size: uvec3(0, 0, 0),
			not_send: PhantomData,
			//dimensionality: 0,
		}
	}

	pub fn new1d(internalformat: GLenum, levels: u32, width: u32) -> Self {
		Self::create(gl::TEXTURE_1D).storage1d(levels as i32, internalformat, width)
	}

	pub fn new2d(internalformat: GLenum, levels: u32, size: uvec2) -> Self {
		Self::create(gl::TEXTURE_2D).storage2d(levels as i32, internalformat, size.x, size.y)
	}

	pub fn new3d(internalformat: GLenum, levels: u32, size: uvec3) -> Self {
		Self::create(gl::TEXTURE_3D).storage3d(levels as i32, internalformat, size.x, size.y, size.z)
	}

	/// Simultaneously specify storage for all levels of a one-dimensional texture.
	/// http://docs.gl/gl4/glTexStorage1D
	#[allow(non_snake_case)]
	pub fn storage1d(mut self, levels: i32, internalformat: GLenum, width: u32) -> Self {
		glTextureStorage1D(self.handle, levels, internalformat, width as i32);
		//self.dimensionality= 1,
		self.internalformat = internalformat;
		self.size = uvec3(width, 0, 0);
		self
	}

	/// Simultaneously specify storage for all levels of a two-dimensional or one-dimensional array texture.
	/// http://docs.gl/gl4/glTexStorage2D
	#[allow(non_snake_case)]
	pub fn storage2d(mut self, levels: i32, internalformat: GLenum, width: u32, height: u32) -> Self {
		glTextureStorage2D(self.handle, levels, internalformat, width as i32, height as i32);
		//self.dimensionality: 2,
		self.internalformat = internalformat;
		self.size = uvec3(width, height, 0);
		self
	}

	/// Simultaneously specify storage for all levels of a three-dimensional, two-dimensional array or cube-map array texture.
	/// http://docs.gl/gl4/glTexStorage3D
	#[allow(non_snake_case)]
	pub fn storage3d(mut self, levels: i32, internalformat: GLenum, width: u32, height: u32, depth: u32) -> Self {
		glTextureStorage3D(self.handle, levels, internalformat, width as i32, height as i32, depth as i32);
		//self.dimensionality= 3,
		self.internalformat = internalformat;
		self.size = uvec3(width, height, depth);
		self
	}

	pub fn generate_mipmap(self) -> Self {
		glGenerateTextureMipmap(self.handle);
		self
	}

	pub fn internalformat(&self) -> GLenum {
		self.internalformat
	}

	pub fn size(&self) -> uvec3 {
		self.size
	}

	pub fn bind_image_unit(&self, unit: u32, access: GLenum) {
		glBindImageTexture(unit, self.handle, 0, false, 0, access, self.internalformat());
	}

	pub fn bind_texture_unit(&self, unit: u32) {
		glBindTextureUnit(unit, self.handle)
	}

	/// Specify a one-dimensional texture subimage.
	/// http://docs.gl/gl4/glTexSubImage1D
	pub fn sub_image1d<T>(self, level: i32, xoffset: i32, width: i32, format: GLenum, typ: GLenum, pixels: &[T]) -> Self
	where
		T: Sized + Copy + 'static,
	{
		glTextureSubImage1D(self.handle, level, xoffset, width, format, typ, pixels);
		self
	}

	/// Specify a two-dimensional texture subimage.
	/// http://docs.gl/gl4/glTexSubImage2D
	pub fn sub_image2d<T>(self, level: u32, xoffset: u32, yoffset: u32, width: u32, height: u32, format: GLenum, typ: GLenum, pixels: &[T]) -> Self
	where
		T: Sized + Copy + 'static,
	{
		glTextureSubImage2D(self.handle, level as i32, xoffset as i32, yoffset as i32, width as i32, height as i32, format, typ, pixels);
		self
	}

	/// Specify a three-dimensional texture subimage.
	/// http://docs.gl/gl4/glTexSubImage3D
	pub fn sub_image3d<T>(self, level: u32, xoffset: u32, yoffset: u32, zoffset: u32, width: u32, height: u32, depth: u32, format: GLenum, typ: GLenum, pixels: &[T]) -> Self
	where
		T: Sized + Copy + 'static,
	{
		glTextureSubImage3D(
			self.handle,
			level as i32,
			xoffset as i32,
			yoffset as i32,
			zoffset as i32,
			width as i32,
			height as i32,
			depth as i32,
			format,
			typ,
			pixels,
		);
		self
	}

	/// Set texture parameters.
	/// http://docs.gl/gl4/glTexParameter
	pub fn parameterf(self, pname: GLenum, param: f32) -> Self {
		glTextureParameterf(self.handle, pname, param);
		self
	}

	/// Set texture parameters.
	/// http://docs.gl/gl4/glTexParameter
	pub fn parameterfv(self, pname: GLenum, param: &[f32]) -> Self {
		glTextureParameterfv(self.handle, pname, param);
		self
	}

	/// Set texture parameters.
	/// http://docs.gl/gl4/glTexParameter
	pub fn parameteri(self, pname: GLenum, param: i32) -> Self {
		glTextureParameteri(self.handle, pname, param);
		self
	}

	/// Set texture parameters.
	/// http://docs.gl/gl4/glTexParameter
	pub fn parameteriv(self, pname: GLenum, param: &[i32]) -> Self {
		glTextureParameteriv(self.handle, pname, param);
		self
	}

	pub fn filter_linear(self) -> Self {
		self //
			.parameteri(gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32)
			.parameteri(gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32)
	}

	pub fn filter_nearest(self) -> Self {
		self //
			.parameteri(gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32)
			.parameteri(gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32)
	}

	pub fn clamp_to_edge(self) -> Self {
		self //
			.parameteri(gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32)
			.parameteri(gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32)
	}

	pub fn wrap_repeat(self) -> Self {
		self //
			.parameteri(gl::TEXTURE_WRAP_S, gl::REPEAT as i32)
			.parameteri(gl::TEXTURE_WRAP_T, gl::REPEAT as i32)
	}

	pub fn wrap_mirrored_repeat(self) -> Self {
		self //
			.parameteri(gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT as i32)
			.parameteri(gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT as i32)
	}
}

impl Drop for Texture {
	fn drop(&mut self) {
		//debug!("drop Texture {}", self.handle);
		glDeleteTexture(self.handle);
	}
}
