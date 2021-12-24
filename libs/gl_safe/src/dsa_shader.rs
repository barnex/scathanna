use super::*;
use gl::types::*;
use std::ffi::CString;
use std::ptr;

/// Creates a shader object.
/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glCreateShader.xhtml
#[allow(non_snake_case)]
pub fn glCreateShader(shader_type: GLenum) -> GLuint {
	let shader = unsafe { gl::CreateShader(shader_type) };
	check::gl_error();
	shader
}

///  Deletes a shader object.
/// http://docs.gl/gl4/glDeleteShader
#[allow(non_snake_case)]
pub fn glDeleteShader(shader: GLenum) {
	unsafe { gl::DeleteShader(shader) };
	check::gl_error();
}

/// Replaces the source code in a shader object.
/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glShaderSource.xhtml
#[allow(non_snake_case)]
pub fn glShaderSource(shader: GLuint, src: &str) {
	let count = 1;
	let c_str = CString::new(src.as_bytes()).unwrap();
	let strings = &c_str.as_ptr();
	let length = ptr::null();
	unsafe { gl::ShaderSource(shader, count, strings, length) };
	check::gl_error()
}

/// Compiles a shader object.
/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glCompileShader.xhtml
#[allow(non_snake_case)]
pub fn glCompileShader(shader: GLuint) {
	unsafe { gl::CompileShader(shader) };
	check::gl_error()
}

/// Returns a parameter from a shader object.
/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glGetShader.xhtml
#[allow(non_snake_case)]
pub fn glGetShaderiv(shader: GLuint, pname: GLenum) -> i32 {
	let mut params = 0;
	unsafe { gl::GetShaderiv(shader, pname, &mut params) };
	check::gl_error();
	params
}

/// Returns the information log for a shader object.
/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glGetShaderInfoLog.xhtml
#[allow(non_snake_case)]
pub fn glGetShaderInfoLog(shader: GLuint) -> String {
	let max_length = glGetShaderiv(shader, gl::INFO_LOG_LENGTH);
	if max_length == 0 {
		return "".into();
	}
	let mut buf = Vec::with_capacity(max_length as usize);
	let length = ptr::null_mut();
	unsafe {
		buf.set_len((max_length as usize) - 1); // skip nul terminator
		let info_log = buf.as_mut_ptr() as *mut GLchar;
		gl::GetShaderInfoLog(shader, max_length, length, info_log);
	};
	String::from_utf8(buf).expect("invalid utf-8")
}
