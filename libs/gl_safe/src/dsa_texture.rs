use super::check;
use super::*;
use std::mem;

/// Create a texture object.
/// http://docs.gl/gl4/glCreateTextures
#[allow(non_snake_case)]
pub fn glCreateTexture(target: GLenum) -> GLuint {
	let mut result = 0;
	unsafe { gl::CreateTextures(target, 1, &mut result) };
	check::gl_error();
	result
}

///  Deletes a texture object.
/// http://docs.gl/gl4/glDeleteTextures
#[allow(non_snake_case)]
pub fn glDeleteTexture(texture: GLuint) {
	unsafe { gl::DeleteTextures(1, &texture) };
	check::gl_error();
}

/// Generate mipmaps for a specified texture object.
/// http://docs.gl/gl4/glGenerateMipmap
#[allow(non_snake_case)]
pub fn glGenerateTextureMipmap(texture: GLuint) {
	unsafe { gl::GenerateTextureMipmap(texture) };
	check::gl_error();
}

/// Simultaneously specify storage for all levels of a one-dimensional texture.
/// http://docs.gl/gl4/glTexStorage1D
#[allow(non_snake_case)]
pub fn glTextureStorage1D(texture: GLuint, levels: i32, internalformat: GLenum, width: i32) {
	unsafe { gl::TextureStorage1D(texture, levels, internalformat, width) };
	check::gl_error()
}

/// Simultaneously specify storage for all levels of a two-dimensional or one-dimensional array texture.
/// http://docs.gl/gl4/glTexStorage2D
#[allow(non_snake_case)]
pub fn glTextureStorage2D(texture: GLuint, levels: i32, internalformat: GLenum, width: i32, height: i32) {
	unsafe { gl::TextureStorage2D(texture, levels, internalformat, width, height) };
	check::gl_error()
}

/// Simultaneously specify storage for all levels of a three-dimensional, two-dimensional array or cube-map array texture.
/// http://docs.gl/gl4/glTexStorage3D
#[allow(non_snake_case)]
pub fn glTextureStorage3D(texture: GLuint, levels: i32, internalformat: GLenum, width: i32, height: i32, depth: i32) {
	unsafe { gl::TextureStorage3D(texture, levels, internalformat, width, height, depth) };
	check::gl_error()
}

/// Specify a one-dimensional texture subimage.
/// http://docs.gl/gl4/glTexSubImage1D
#[allow(non_snake_case)]
pub fn glTextureSubImage1D<T>(texture: GLuint, level: i32, xoffset: i32, width: i32, format: GLenum, typ: GLenum, pixels: &[T])
where
	T: Sized + Copy + 'static,
{
	check::image_size(&[width], format, typ, pixels);
	unsafe { gl::TextureSubImage1D(texture, level, xoffset, width, format, typ, mem::transmute(&pixels[0])) };
	check::gl_error()
}

/// Specify a two-dimensional texture subimage.
/// http://docs.gl/gl4/glTexSubImage2D
#[allow(non_snake_case)]
pub fn glTextureSubImage2D<T>(texture: GLuint, level: i32, xoffset: i32, yoffset: i32, width: i32, height: i32, format: GLenum, typ: GLenum, pixels: &[T])
where
	T: Sized + Copy + 'static,
{
	check::image_size(&[width, height], format, typ, pixels);
	unsafe { gl::TextureSubImage2D(texture, level, xoffset, yoffset, width, height, format, typ, mem::transmute(&pixels[0])) };
	check::gl_error()
}

/// Specify a three-dimensional texture subimage.
/// http://docs.gl/gl4/glTexSubImage3D
#[allow(non_snake_case)]
pub fn glTextureSubImage3D<T>(texture: GLuint, level: i32, xoffset: i32, yoffset: i32, zoffset: i32, width: i32, height: i32, depth: i32, format: GLenum, typ: GLenum, pixels: &[T])
where
	T: Sized + Copy + 'static,
{
	check::image_size(&[width, height, depth], format, typ, pixels);
	unsafe { gl::TextureSubImage3D(texture, level, xoffset, yoffset, zoffset, width, height, depth, format, typ, mem::transmute(&pixels[0])) };
	check::gl_error()
}

/// Set texture parameters.
/// http://docs.gl/gl4/glTexParameter
#[allow(non_snake_case)]
pub fn glTextureParameterf(texture: GLuint, pname: GLenum, param: f32) {
	unsafe { gl::TextureParameterf(texture, pname, param) };
	check::gl_error()
}

/// Set texture parameters.
/// http://docs.gl/gl4/glTexParameter
#[allow(non_snake_case)]
pub fn glTextureParameterfv(texture: GLuint, pname: GLenum, param: &[f32]) {
	unsafe { gl::TextureParameterfv(texture, pname, &param[0]) };
	check::gl_error()
}

/// Set texture parameters.
/// http://docs.gl/gl4/glTexParameter
#[allow(non_snake_case)]
pub fn glTextureParameteri(texture: GLuint, pname: GLenum, param: i32) {
	unsafe { gl::TextureParameteri(texture, pname, param) };
	check::gl_error()
}

/// Set texture parameters.
/// http://docs.gl/gl4/glTexParameter
#[allow(non_snake_case)]
pub fn glTextureParameteriv(texture: GLuint, pname: GLenum, param: &[i32]) {
	unsafe { gl::TextureParameteriv(texture, pname, &param[0]) };
	check::gl_error()
}

// //pub fn glTextureParameterIiv (texture: GLuint, pname: GLenum, const int *params){}
// //pub fn glTextureParameterIuiv(texture: GLuint, pname: GLenum, const uint *params){}

/// Bind an existing texture object to the specified texture unit.
/// http://docs.gl/gl4/glBindTextureUnit
#[allow(non_snake_case)]
pub fn glBindTextureUnit(unit: GLuint, texture: GLuint) {
	unsafe { gl::BindTextureUnit(unit, texture) };
	check::gl_error()
}
