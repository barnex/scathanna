use super::internal::*;
use std::ops::{Index, IndexMut};

/// An Image is a rectangular 2D array of color values
/// (RGB, grayscale, ...)
#[derive(Debug, PartialEq, Clone)]
pub struct Image<C> {
	dim: (u32, u32),
	values: Vec<C>,
}

impl<'a, C> Image<C>
where
	C: Copy + Default,
{
	/// new constructs an image with given width and height.
	pub fn new((w, h): (u32, u32)) -> Image<C> {
		Image {
			dim: (w, h),
			values: vec![C::default(); w as usize * h as usize],
		}
	}

	pub fn from_fn<F: FnMut(u32, u32) -> C>((w, h): (u32, u32), mut f: F) -> Self {
		let mut img = Self::new((w, h));
		for iy in 0..h {
			for ix in 0..w {
				img[iy as usize][ix as usize] = f(ix, iy);
			}
		}
		img
	}

	/// Draw img over this image.
	pub fn draw(&mut self, pos: (u32, u32), img: &Image<C>) {
		for y in 0..img.height() {
			for x in 0..img.width() {
				let dst = (pos.0 + x, pos.1 + y);
				if dst.0 < self.width() && dst.1 < self.height() {
					self.set(dst, img.at((x, y)));
				}
			}
		}
	}

	pub fn map<F, T>(&self, f: F) -> Image<T>
	where
		T: Copy + Default,
		F: Fn(C) -> T,
	{
		Image::<T> {
			dim: self.dim,
			values: self.values.iter().copied().map(f).collect(),
		}
	}

	pub fn at<P: Into<uvec2>>(&self, p: P) -> C {
		let p: uvec2 = p.into();
		self[p.y() as usize][p.x() as usize]
	}

	pub fn at_mut<P: Into<uvec2>>(&mut self, p: P) -> &mut C {
		let p: uvec2 = p.into();
		&mut self[p.y() as usize][p.x() as usize]
	}

	pub fn set(&mut self, p: (u32, u32), c: C) {
		self[p.1 as usize][p.0 as usize] = c;
	}

	pub fn try_set(&mut self, p: (u32, u32), c: C) {
		if p.0 < self.width() && p.1 < self.height() {
			self[p.1 as usize][p.0 as usize] = c;
		}
	}

	/// width of the image, in pixels
	pub fn width(&self) -> u32 {
		self.dim.0
	}

	/// height of the image, in pixels
	pub fn height(&self) -> u32 {
		self.dim.1
	}

	/// width and height of the image
	pub fn dimensions(&self) -> (u32, u32) {
		self.dim
	}

	/// pixels in row-major order, iterable.
	pub fn pixels(&self) -> &[C] {
		&self.values
	}

	/// pixels in row-major order, iterable.
	pub fn pixels_mut(&mut self) -> &mut [C] {
		&mut self.values
	}
}

impl<C> Default for Image<C>
where
	C: Copy + Default,
{
	fn default() -> Self {
		Self { dim: (0, 0), values: Vec::new() }
	}
}

impl<C> Index<usize> for Image<C>
where
	C: Copy + Default,
{
	type Output = [C];

	fn index(&self, i: usize) -> &[C] {
		let l = i * self.width() as usize;
		let h = l + self.width() as usize;
		&self.values[l..h]
	}
}

impl<C> IndexMut<usize> for Image<C>
where
	C: Copy + Default,
{
	fn index_mut(&mut self, i: usize) -> &mut [C] {
		let l = i * self.width() as usize;
		let h = l + self.width() as usize;
		&mut self.values[l..h]
	}
}

impl Image<Color> {
	/// Convert the image to raw BGRA bytes.
	/// Used to create SDL textures (brilliance-ui).
	//pub fn raw_bgra(&self) -> Vec<u8> {
	//	let (w, h) = self.dimensions();
	//	let mut raw = Vec::with_capacity((w * h * 4) as usize);
	//	for iy in 0..h {
	//		for ix in 0..w {
	//			let c = self[iy as usize][ix as usize].bgra();
	//			raw.extend_from_slice(&c);
	//		}
	//	}
	//	raw
	//}

	/// Convert the image to raw RGBA bytes.
	/// Used to create SDL textures (brilliance-ui).
	pub fn raw_rgba(&self) -> Vec<[u8; 4]> {
		let (w, h) = self.dimensions();
		let mut raw = Vec::with_capacity((w * h) as usize);
		for iy in 0..h {
			for ix in 0..w {
				raw.push(self[iy as usize][ix as usize].rgba())
			}
		}
		raw
	}

	/// Convert the image to raw RGB bytes.
	/// Used to save as image.
	pub fn raw_rgb(&self) -> Vec<u8> {
		let (w, h) = self.dimensions();
		let mut raw = Vec::with_capacity((w * h * 3) as usize);
		for iy in 0..h {
			for ix in 0..w {
				let c = self[iy as usize][ix as usize].srgb();
				raw.extend_from_slice(&c);
			}
		}
		raw
	}
}
