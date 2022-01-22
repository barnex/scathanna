use super::internal::*;
use image::jpeg::JpegEncoder;
use std::fs::File;
use std::path::Path;

#[must_use]
pub fn save<P: AsRef<Path>>(img: &Image<Color>, path: P) -> Result<()> {
	let path = path.as_ref();
	let (w, h) = img.dimensions();
	image::save_buffer(path, &img.raw_rgb(), w as u32, h as u32, image::ColorType::Rgb8) //
		.map_err(|e| anyhow!("save {}: {}", path.to_string_lossy(), e))
}

#[must_use]
pub fn save_jpg(img: &Image<Color>, fname: &str, qual: u8) -> Result<()> {
	let mut f = File::create(fname)?;
	let mut enc = JpegEncoder::new_with_quality(&mut f, qual);
	let (w, h) = img.dimensions();
	match enc.encode(&img.raw_rgb(), w as u32, h as u32, image::ColorType::Rgb8) {
		Err(e) => Err(anyhow!("save {}: {}", fname, e.to_string())),
		Ok(()) => Ok(()),
	}
}

pub fn load_rgb<P: AsRef<Path>>(fname: P) -> Result<Image<[u8; 3]>> {
	let fname = fname.as_ref();
	let orig = match image::open(fname) {
		Ok(img) => img,
		Err(e) => return Err(anyhow!("load {:?}: {}", fname, e.to_string())),
	};
	let rgb = orig.into_rgb8();

	let img = Image::<[u8; 3]>::from_fn(rgb.dimensions(), |x, y| {
		let pix = rgb.get_pixel(x, y);
		[pix[0], pix[1], pix[2]]
	});
	Ok(img)
}

pub fn load_rgba<P: AsRef<Path>>(fname: P) -> Result<Image<[u8; 4]>> {
	let fname = fname.as_ref();
	let orig = match image::open(fname) {
		Ok(img) => img,
		Err(e) => return Err(anyhow!("load {:?}: {}", fname, e.to_string())),
	};
	let rgb = orig.into_rgba8();

	let img = Image::<[u8; 4]>::from_fn(rgb.dimensions(), |x, y| {
		let pix = rgb.get_pixel(x, y);
		[pix[0], pix[1], pix[2], pix[3]]
	});
	Ok(img)
}
