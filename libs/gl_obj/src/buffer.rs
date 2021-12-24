use core::any::TypeId;
use gl_safe::*;
use std::marker::PhantomData;
use std::mem::size_of;
use std::rc::Rc;

pub struct Buffer {
	handle: GLuint,
	len: usize,
	stride: u32,
	typeid: TypeId,

	// OpenGL functions may only be called from the thread that was used to initialize GL.
	// Therefore, mark as !Send, !Sync to avoid accidental access from other threads
	// (this would segfault).
	not_send: PhantomData<Rc<()>>,
}

impl Buffer {
	/// Create a buffer object.
	/// http://docs.gl/gl4/glCreateBuffers
	pub fn create() -> Self {
		let handle = glCreateBuffer();
		//debug!("create Buffer {}", handle);
		Self {
			handle,
			len: 0,
			stride: 0,
			typeid: TypeId::of::<()>(),
			not_send: PhantomData,
		}
	}

	/// Creates and initializes a buffer object's immutable data store.
	/// http://docs.gl/gl4/glBufferStorage
	pub fn storage<T>(mut self, data: &[T], flags: GLbitfield) -> Self
	where
		T: Sized + Copy + 'static,
	{
		glNamedBufferStorage(self.handle, data, flags);
		self.typeid = TypeId::of::<T>();
		self.stride = size_of::<T>() as u32;
		self.len = data.len();
		self
	}

	pub fn stride(&self) -> i32 {
		self.stride as i32
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn bytes(&self) -> usize {
		self.len * (self.stride as usize)
	}

	pub fn handle(&self) -> GLuint {
		self.handle
	}

	//pub fn gl_type(&self) -> GLenum {
	//	match self.typeid {
	//		TypeId::of::<f32>() => gl::FLOAT,
	//	}
	//}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		//debug!("drop Buffer {}", self.handle);
		glDeleteBuffer(self.handle);
	}
}
