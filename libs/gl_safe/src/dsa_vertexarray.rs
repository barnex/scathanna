use super::*;
use gl::types::*;

/// https://www.khronos.org/registry/OpenGL/extensions/ARB/ARB_direct_state_access.txt

/// Create a vertex array object.
/// http://docs.gl/gl4/glCreateVertexArrays
#[allow(non_snake_case)]
pub fn glCreateVertexArray() -> GLuint {
	let mut result = 0;
	unsafe { gl::CreateVertexArrays(1, &mut result) };
	check::gl_error();
	result
}

/// Delete a vertex array object.
/// http://docs.gl/gl4/glDeleteVertexArrays
#[allow(non_snake_case)]
pub fn glDeleteVertexArray(buffer: GLuint) {
	unsafe { gl::DeleteVertexArrays(1, &buffer) };
	check::gl_error()
}

/// Enable a generic vertex attribute array.
/// http://docs.gl/gl4/glEnableVertexAttribArray
#[allow(non_snake_case)]
pub fn glEnableVertexArrayAttrib(vaobj: GLuint, index: u32) {
	unsafe { gl::EnableVertexArrayAttrib(vaobj, index) };
	check::gl_error();
}

/// Associate a vertex attribute and a vertex buffer binding for a vertex array object
/// http://docs.gl/gl4/glVertexAttribBinding
#[allow(non_snake_case)]
pub fn glVertexArrayAttribBinding(vaobj: GLuint, attribindex: u32, bindingindex: u32) {
	unsafe { gl::VertexArrayAttribBinding(vaobj, attribindex, bindingindex) };
	check::gl_error();
}

/// Specify the organization of vertex arrays.
/// http://docs.gl/gl4/glVertexAttribFormat
#[allow(non_snake_case)]
pub fn glVertexArrayAttribFormat(vaobj: GLuint, attribindex: u32, size: i32, typ: GLenum, normalized: bool, relativeoffset: u32) {
	unsafe { gl::VertexArrayAttribFormat(vaobj, attribindex, size, typ, normalized as GLboolean, relativeoffset) };
	check::gl_error();
}

/// Specify the organization of vertex arrays.
/// http://docs.gl/gl4/glVertexAttribFormat
#[allow(non_snake_case)]
pub fn glVertexArrayAttribIFormat(vaobj: GLuint, attribindex: u32, size: i32, typ: GLenum, relativeoffset: u32) {
	unsafe { gl::VertexArrayAttribIFormat(vaobj, attribindex, size, typ, relativeoffset) };
	check::gl_error();
}

/// Specify the organization of vertex arrays.
/// http://docs.gl/gl4/glVertexAttribFormat
#[allow(non_snake_case)]
pub fn glVertexArrayAttribLFormat(vaobj: GLuint, attribindex: u32, size: i32, typ: GLenum, relativeoffset: u32) {
	unsafe { gl::VertexArrayAttribLFormat(vaobj, attribindex, size, typ, relativeoffset) };
	check::gl_error();
}

/// Configures element array buffer binding of a vertex array object.
/// http://docs.gl/gl4/glVertexArrayElementBuffer
#[allow(non_snake_case)]
pub fn glVertexArrayElementBuffer(vaobj: GLuint, buffer: GLuint) {
	unsafe { gl::VertexArrayElementBuffer(vaobj, buffer) };
	check::gl_error();
}

/// Bind a buffer to a vertex buffer bind point.
/// https://khronos.org/registry/OpenGL-Refpages/gl4/html/glBindVertexBuffer.xhtml
#[allow(non_snake_case)]
pub fn glVertexArrayVertexBuffer(vaobj: GLuint, bindingindex: GLuint, buffer: GLuint, offset: GLintptr, stride: GLsizei) {
	unsafe { gl::VertexArrayVertexBuffer(vaobj, bindingindex, buffer, offset, stride) };
	check::gl_error();
}
