use super::internal::*;

pub struct Lightmap {
	// TODO: should not convert to srgb in GL thread.
	// should not convert back and forth from srgb when loading from file
	image: Image<Color>,
}

impl Lightmap {
	pub fn new(size: u32) -> Self {
		Self { image: Image::new((size, size)) } // TODO: init with grey
	}

	/*
	pub fn from(image: Image<Color>) -> Self {
		Self { image }
	}
	*/

	pub fn from_rgb(img: &Image<[u8; 3]>) -> Self {
		Self { image: img.map(|rgb| rgb.into()) }
	}

	pub fn image(&self) -> &Image<Color> {
		&self.image
	}

	pub fn image_mut(&mut self) -> &mut Image<Color> {
		&mut self.image
	}
}
