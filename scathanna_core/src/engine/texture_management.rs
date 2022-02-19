use super::internal::*;

pub fn load_mipmapped(levels: u32, src: &Path, replacement_color: vec3) -> Texture {
	match load_texture(levels, &src) {
		Err(e) => {
			eprintln!("load {:?}: {}", src, e);
			fallback_texture(replacement_color)
		}
		Ok(tex) => tex
			.wrap_repeat()
			.generate_mipmap()
			.parameteri(gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32)
			.parameteri(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32),
	}
}

/*
fn load_nearest(src: &str, replacement_color: vec3) -> Texture {
	const NO_MIPMAP_LEVELS: u32 = 1;
	match load_texture(NO_MIPMAP_LEVELS, &src) {
		Err(e) => {
			eprintln!("load {}: {}", src, e);
			fallback_texture(replacement_color)
		}
		Ok(tex) => tex.parameteri(gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32).parameteri(gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32),
	}
}
*/

// simple 2x2 texture to be used when loading the real texture failed.
pub fn fallback_texture(replacement_color: vec3) -> Texture {
	let size = uvec2(2, 2);
	let c = replacement_color.map(|v| (255.0 * v) as u8);
	let (r, g, b) = c.into();
	let data: [[u8; 4]; 4] = [
		[r, g, b, 255], //
		[r, g, b, 255], //
		[r, g, b, 255], //
		[r, g, b, 255], //
	];

	let levels = 1;
	Texture::new2d(gl::RGBA8, levels, size) //
		.sub_image2d(0, 0, 0, size.x(), size.y(), gl::RGBA, gl::UNSIGNED_BYTE, &data)
		.wrap_repeat()
		.parameteri(gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32)
		.parameteri(gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32)
}

fn load_texture(levels: u32, fname: &Path) -> Result<Texture> {
	println!("loading {}", fname.to_string_lossy());
	let src = imageutil::load_rgba(fname)?;
	let size = uvec2(src.width(), src.height());
	Ok(image_texture(levels, size, src.pixels()))
}

fn image_texture(levels: u32, size: uvec2, raw_bgra: &[[u8; 4]]) -> Texture {
	debug_assert!(levels > 0);
	Texture::new2d(gl::RGBA8, levels, size) //
		.sub_image2d(0, 0, 0, size.x(), size.y(), gl::RGBA, gl::UNSIGNED_BYTE, &raw_bgra)
}
