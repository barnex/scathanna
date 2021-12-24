use super::check;
use super::*;

/// Render primitives from array data.
/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glDrawArrays.xhtml
#[allow(non_snake_case)]
pub fn glDrawArrays(mode: GLenum, first: i32, count: i32) {
	unsafe { gl::DrawArrays(mode, first, count) };
	check::gl_error()
}

/// select a polygon rasterization mode.
/// http://docs.gl/gl4/glPolygonMode
#[allow(non_snake_case)]
pub fn glPolygonMode(face: GLenum, mode: GLenum) {
	unsafe { gl::PolygonMode(face, mode) };
	check::gl_error()
}

/// Set the scale and units used to calculate depth values.
/// http://docs.gl/gl4/glPolygonOffset
#[allow(non_snake_case)]
pub fn glPolygonOffset(factor: f32, units: f32) {
	unsafe { gl::PolygonOffset(factor, units) };
	check::gl_error()
}

/// Specify the width of rasterized lines.
/// http://docs.gl/gl4/glLineWidth
#[allow(non_snake_case)]
pub fn glLineWidth(width: f32) {
	unsafe { gl::LineWidth(width) };
	check::gl_error()
}

/// Define front- and back-facing polygons.
/// http://docs.gl/gl4/glFrontFace
#[allow(non_snake_case)]
pub fn glFrontFace(mode: GLenum) {
	unsafe { gl::FrontFace(mode) };
	check::gl_error()
}

/// Specify clear values for the color buffers.
/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glClearColor.xhtml
#[allow(non_snake_case)]
pub fn glClearColor(red: f32, green: f32, blue: f32, alpha: f32) {
	unsafe { gl::ClearColor(red, green, blue, alpha) };
	check::gl_error()
}

/// Clear buffers to preset values.
/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glClear.xhtml
#[allow(non_snake_case)]
pub fn glClear(mask: GLbitfield) {
	unsafe { gl::Clear(mask) };
	check::gl_error()
}

/// Specify mapping of depth values from normalized device coordinates to window coordinates.
/// http://docs.gl/gl4/glDepthRange
#[allow(non_snake_case)]
pub fn glDepthRangef(nearVal: f32, farVal: f32) {
	unsafe { gl::DepthRangef(nearVal, farVal) };
	check::gl_error()
}

/// Set the viewport.
/// http://docs.gl/gl4/glViewport
#[allow(non_snake_case)]
pub fn glViewport(x: i32, y: i32, width: i32, height: i32) {
	unsafe { gl::Viewport(x, y, width, height) };
	check::gl_error()
}

/// Enable server-side GL capabilities
/// http://docs.gl/gl4/glEnable
#[allow(non_snake_case)]
pub fn glEnable(cap: GLenum) {
	unsafe { gl::Enable(cap) };
	check::gl_error()
}

/// Specify pixel arithmetic.
/// http://docs.gl/gl4/glBlendFunc
#[allow(non_snake_case)]
pub fn glBlendFunc(sfactor: GLenum, dfactor: GLenum) {
	unsafe { gl::BlendFunc(sfactor, dfactor) };
	check::gl_error()
}

/// Disable server-side GL capabilities
/// http://docs.gl/gl4/glEnable
#[allow(non_snake_case)]
pub fn glDisable(cap: GLenum) {
	unsafe { gl::Disable(cap) };
	check::gl_error()
}
