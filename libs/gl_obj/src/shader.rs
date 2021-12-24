use gl_safe::*;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct Shader {
	handle: GLuint,
	// OpenGL functions may only be called from the thread that was used to initialize GL.
	// Therefore, mark as !Send, !Sync to avoid accidental access from other threads
	// (this would segfault).
	not_send: PhantomData<Rc<()>>,
}

impl Shader {
	/// Creates a shader object.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glCreateShader.xhtml
	pub fn create(shader_type: GLenum) -> Self {
		Self {
			handle: glCreateShader(shader_type),
			not_send: PhantomData,
		}
	}

	/// Creates a vertex shader from source.
	pub fn new_vertex(src: &str) -> Self {
		Self::create(gl::VERTEX_SHADER).source(src).compile().expect("compile vertex shader")
	}

	/// Creates a fragment shader from source.
	pub fn new_fragment(src: &str) -> Self {
		Self::create(gl::FRAGMENT_SHADER).source(src).compile().expect("compile fragment shader")
	}

	/// Creates a compute shader from source.
	pub fn new_compute(src: &str) -> Self {
		Self::create(gl::COMPUTE_SHADER).source(src).compile().expect("compile compute shader")
	}

	/// Replaces the source code in a shader object.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glShaderSource.xhtml
	pub fn source(self, src: &str) -> Self {
		glShaderSource(self.handle, src);
		self
	}

	/// Compiles a shader object.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glCompileShader.xhtml
	#[must_use]
	pub fn compile(self) -> Result<Self, String> {
		glCompileShader(self.handle);
		let status = self.get_iv(gl::COMPILE_STATUS);
		if status != (gl::TRUE as GLint) {
			Err(self.info_log())
		} else {
			Ok(self)
		}
	}

	/// Returns a parameter from a shader object.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glGetShader.xhtml
	/// TODO: iv is vector!
	pub fn get_iv(&self, pname: GLenum) -> i32 {
		glGetShaderiv(self.handle, pname)
	}

	/// Returns the information log for a shader object.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glGetShaderInfoLog.xhtml
	pub fn info_log(&self) -> String {
		glGetShaderInfoLog(self.handle)
	}

	pub fn handle(&self) -> GLuint {
		self.handle
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		//debug!("drop Shader {}", self.handle);
		glDeleteShader(self.handle);
	}
}
