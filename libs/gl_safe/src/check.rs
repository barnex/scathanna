use super::glenum;
use gl::types::*;
use std::mem;

pub fn image_size<T>(dim: &[i32], format: GLenum, typ: GLenum, pixels: &[T]) {
	let have_pix = dim.iter().map(|x| *x as usize).fold(1, |a, b| a * b);
	if have_pix != pixels.len() {
		panic!("image size: want {} pixels ({:?}), have: {}", have_pix, dim, pixels.len());
	}
	let fel = format_num_el(format);
	let tyb = type_num_bytes(typ);
	if (fel * tyb) != mem::size_of::<T>() {
		panic!(
			"image format {} + {} does not match pixel type {}",
			glenum::to_str(format),
			glenum::to_str(typ),
			std::any::type_name::<T>()
		);
	}
}

fn format_num_el(format: GLenum) -> usize {
	match format {
		gl::RED => 1,
		gl::RG => 2,
		gl::RGB => 3,
		gl::RGBA => 4,
		x => panic!("unknown pixel format: {}", x),
	}
}

fn type_num_bytes(typ: GLenum) -> usize {
	match typ {
		gl::FLOAT => 4,
		gl::UNSIGNED_BYTE => 1,
		x => panic!("unknown pixel type: {}", x),
	}
}

#[cfg(debug_assertions)]
pub fn gl_error() {
	let code = unsafe { gl::GetError() };
	if code != 0 {
		let msg = match code {
			gl::INVALID_ENUM => "INVALID_ENUM",
			gl::INVALID_VALUE => "INVALID_VALUE",
			gl::INVALID_OPERATION => "INVALID_OPERATION",
			gl::INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
			gl::OUT_OF_MEMORY => "OUT_OF_MEMORY",
			gl::STACK_UNDERFLOW => "STACK_UNDERFLOW",
			gl::STACK_OVERFLOW => "STACK_OVERFLOW",
			_ => panic!("GL error code {}", code),
		};
		panic!("GL error {}", msg);
	}
}

#[cfg(not(debug_assertions))]
#[inline]
pub fn gl_error() {}
