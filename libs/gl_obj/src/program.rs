use super::*;
use gl_safe::*;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(PartialEq)]
pub struct Program {
	handle: GLuint,

	// OpenGL functions may only be called from the thread that was used to initialize GL.
	// Therefore, mark as !Send, !Sync to avoid accidental access from other threads
	// (this would segfault).
	not_send: PhantomData<Rc<()>>,
}

impl Program {
	/// Creates a program object.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glCreateProgram.xhtml
	pub fn create() -> Self {
		Self {
			handle: glCreateProgram(),
			not_send: PhantomData,
		}
	}

	pub fn new(shaders: &[&Shader]) -> Self {
		let mut p = Self::create();
		for s in shaders {
			p.attach_shader(s);
		}
		p.link().expect("link program")
	}

	pub fn handle(&self) -> GLuint {
		self.handle
	}

	/// Attaches a shader object to a program object.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glAttachShader.xhtml
	pub fn attach_shader(&mut self, shader: &Shader) {
		glAttachShader(self.handle(), shader.handle());
	}

	/// Links a program object.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glLinkProgram.xhtml
	#[must_use]
	pub fn link(self) -> Result<Self, String> {
		glLinkProgram(self.handle());
		let status = glGetProgramiv(self.handle(), gl::LINK_STATUS, 1)[0];
		if status != (gl::TRUE as GLint) {
			Err(self.info_log())
		} else {
			Ok(self)
		}
	}

	/// Returns a parameter from a program object.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glGetProgram.xhtml.
	pub fn get_iv(&self, pname: GLenum, n: usize) -> Vec<i32> {
		glGetProgramiv(self.handle(), pname, n)
	}

	/// Returns the information log for a program object.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glGetProgramInfoLog.xhtml
	pub fn info_log(&self) -> String {
		glGetProgramInfoLog(self.handle())
	}

	/// Returns the location of an attribute variable.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glGetAttribLocation.xhtml
	pub fn attrib_location(&self, name: &str) -> Option<u32> {
		let result = glGetAttribLocation(self.handle(), name);
		if result < 0 {
			None
		} else {
			Some(result as u32)
		}
	}

	/// Returns the location of a uniform variable.
	/// http://docs.gl/gl4/glGetUniformLocation
	pub fn uniform_location(&self, name: &str) -> u32 {
		let result = glGetUniformLocation(self.handle(), name);
		if result < 0 {
			panic!("uniform location `{}` not found", name);
		} else {
			result as u32
		}
	}

	pub fn compute_work_group_size(&self) -> (u32, u32, u32) {
		let s = self.get_iv(gl::COMPUTE_WORK_GROUP_SIZE, 3);
		(s[0] as u32, s[1] as u32, s[2] as u32)
	}

	pub fn compute_and_sync(&self, global_size: uvec3) {
		// TODO: glGetIntegerv(gl:CURRENT_PROGRAM) + restore
		glUseProgram(self.handle());
		let wgs = self.compute_work_group_size();
		glDispatchCompute(global_size.x() / wgs.0, global_size.y() / wgs.1, global_size.z() / wgs.2);
		glMemoryBarrier(gl::ALL_BARRIER_BITS);
	}

	/// Specify the value of a uniform variable for a specified program object.
	/// http://docs.gl/gl4/glProgramUniform
	pub fn uniform4f(&self, location: u32, v0: f32, v1: f32, v2: f32, v3: f32) {
		//glUseProgram(self.handle()); // Weird that this is needed?
		glProgramUniform4f(self.handle(), location as i32, v0, v1, v2, v3);
	}

	/// Specify the value of a uniform variable for a specified program object.
	/// http://docs.gl/gl4/glProgramUniform
	pub fn uniform3f(&self, location: u32, v0: f32, v1: f32, v2: f32) {
		//glUseProgram(self.handle()); // Weird that this is needed?
		glProgramUniform3f(self.handle(), location as i32, v0, v1, v2);
	}

	/// Specify the value of a uniform variable for a specified program object.
	/// http://docs.gl/gl4/glProgramUniform
	pub fn uniform2f(&self, location: u32, v0: f32, v1: f32) {
		//glUseProgram(self.handle()); // Weird that this is needed?
		glProgramUniform2f(self.handle(), location as i32, v0, v1);
	}

	/// Specify the value of a uniform variable for a specified program object.
	/// http://docs.gl/gl4/glProgramUniform
	pub fn uniform1f(&self, location: u32, v0: f32) {
		//glUseProgram(self.handle()); // Weird that this is needed?
		glProgramUniform1f(self.handle(), location as i32, v0);
	}

	/// Specify the value of a uniform variable for a specified program object.
	/// http://docs.gl/gl4/glProgramUniform
	pub fn uniform4i(&self, location: u32, v0: i32, v1: i32, v2: i32, v3: i32) {
		//glUseProgram(self.handle()); // Weird that this is needed?
		glProgramUniform4i(self.handle(), location as i32, v0, v1, v2, v3);
	}

	/// Specify the value of a uniform variable for a specified program object.
	/// http://docs.gl/gl4/glProgramUniform
	pub fn uniform3i(&self, location: u32, v0: i32, v1: i32, v2: i32) {
		//glUseProgram(self.handle()); // Weird that this is needed?
		glProgramUniform3i(self.handle(), location as i32, v0, v1, v2);
	}

	/// Specify the value of a uniform variable for a specified program object.
	/// http://docs.gl/gl4/glProgramUniform
	pub fn uniform2i(&self, location: u32, v0: i32, v1: i32) {
		//glUseProgram(self.handle()); // Weird that this is needed?
		glProgramUniform2i(self.handle(), location as i32, v0, v1);
	}

	/// Specify the value of a uniform variable for a specified program object.
	/// http://docs.gl/gl4/glProgramUniform
	pub fn uniform1i(&self, location: u32, v0: i32) {
		//glUseProgram(self.handle()); // Weird that this is needed?
		glProgramUniform1i(self.handle(), location as i32, v0);
	}

	/// Specify the value of a uniform variable for a specified program object.
	/// http://docs.gl/gl4/glProgramUniform
	pub fn uniform_matrix4f(&self, location: u32, transpose: bool, v: &[f32; 16]) {
		//assert_eq!(transpose, true); //  for my matrix format. TODO: remove.
		//glUseProgram(self.handle()); // Weird that this is needed?
		glProgramUniformMatrix4f(self.handle(), location as i32, transpose, v);
	}

	//pub fn set1f(self, attrib: &str, v: f32) -> Self {
	//	let loc = self.uniform_location(attrib);
	//	self.uniform1f(loc, v);
	//	self
	//}

	//pub fn set1i(self, attrib: &str, v: i32) -> Self {
	//	let loc = self.uniform_location(attrib);
	//	self.uniform1i(loc, v);
	//	self
	//}

	//pub fn set2i(self, attrib: &str, v0: i32, v1: i32) -> Self {
	//	let loc = self.uniform_location(attrib);
	//	self.uniform2i(loc, v0, v1);
	//	self
	//}

	/// Installs a program object as part of current rendering state.
	/// https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glUseProgram.xhtml
	pub fn use_program(&self) {
		glUseProgram(self.handle())
	}
}

impl Drop for Program {
	fn drop(&mut self) {
		//debug!("drop Program {}", self.handle);
		glDeleteProgram(self.handle());
	}
}
