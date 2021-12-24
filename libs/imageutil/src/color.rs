use super::internal::*;
use std::fmt;
use std::ops::*;
use std::result::Result;

/// Color with floating-point accuracy.
///
/// Represents either a reflectivity or intensity.
///
/// In case of reflectivity, values should be [0..1],
/// 1 meaning 100% reflectivity for that color.
///
/// The color space is linear.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color(vec3);

impl Color {
	/// Color with Red, Green, Blue components.
	#[inline]
	pub fn new(r: f32, g: f32, b: f32) -> Self {
		debug_assert!(r >= 0.0);
		debug_assert!(g >= 0.0);
		debug_assert!(b >= 0.0);
		Self(vec3(r, g, b))
	}

	/// Multiply color by `2^ev` (2 to-the-power-of the Exposure Value).
	pub fn ev(self, ev: f64) -> Self {
		self * f32::powf(2.0, ev as f32)
	}

	#[inline]
	pub fn r(self) -> f32 {
		self[0]
	}
	#[inline]
	pub fn g(self) -> f32 {
		self[1]
	}
	#[inline]
	pub fn b(self) -> f32 {
		self[2]
	}

	/// Convert to 8-bit color, SRGB color space.
	pub fn srgb(&self) -> [u8; 3] {
		[
			(Color::linear_to_srgb(self[0]) * 255.0) as u8,
			(Color::linear_to_srgb(self[1]) * 255.0) as u8,
			(Color::linear_to_srgb(self[2]) * 255.0) as u8,
		]
	}

	pub fn bgra(&self) -> [u8; 4] {
		[
			(Color::linear_to_srgb(self[2]) * 255.0) as u8,
			(Color::linear_to_srgb(self[1]) * 255.0) as u8,
			(Color::linear_to_srgb(self[0]) * 255.0) as u8,
			255,
		]
	}

	pub fn rgba(&self) -> [u8; 4] {
		[
			(Color::linear_to_srgb(self[0]) * 255.0) as u8,
			(Color::linear_to_srgb(self[1]) * 255.0) as u8,
			(Color::linear_to_srgb(self[2]) * 255.0) as u8,
			255,
		]
	}

	/// linear to sRGB conversion
	/// https://en.wikipedia.org/wiki/SRGB
	pub fn linear_to_srgb(c: f32) -> f32 {
		let c = clip(c);
		if c <= 0.0031308 {
			return 12.92 * c;
		}
		let c = 1.055 * c.powf(1. / 2.4) - 0.05;
		if c > 1.0 {
			return 1.0;
		}
		c
	}

	/// Apply f to each component.
	#[inline]
	#[must_use]
	pub fn map<F: Fn(f32) -> f32>(&self, f: F) -> Self {
		Color(self.0.map(f))
	}

	pub const BLACK: Color = Color(vec3(0., 0., 0.));
	pub const BLUE: Color = Color(vec3(0., 0., 1.));
	pub const GREEN: Color = Color(vec3(0., 1., 0.));
	pub const RED: Color = Color(vec3(1., 0., 0.));
	pub const YELLOW: Color = Color(vec3(1., 1., 0.));
	pub const ORANGE: Color = Color(vec3(1., 0.5, 0.));
	pub const MAGENTA: Color = Color(vec3(1., 0., 1.));
	pub const CYAN: Color = Color(vec3(0., 1., 1.));
	pub const WHITE: Color = Color(vec3(1., 1., 1.));
	pub const GRAY: Color = Color(vec3(0.5, 0.5, 0.5));
}

impl Into<vec3> for Color {
	fn into(self) -> vec3 {
		self.0
	}
}

impl Into<Color> for [u8; 3] {
	fn into(self) -> Color {
		Color::new(SRGB_TO_LINEAR[self[0] as usize], SRGB_TO_LINEAR[self[1] as usize], SRGB_TO_LINEAR[self[2] as usize])
	}
}

impl Default for Color {
	#[inline]
	fn default() -> Self {
		Self::BLACK
	}
}

impl Add for Color {
	type Output = Self;
	#[inline]
	fn add(self, rhs: Self) -> Self {
		Self(self.0 + rhs.0)
	}
}

impl AddAssign for Color {
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0
	}
}

impl Mul<Color> for Color {
	type Output = Self;
	#[inline]
	fn mul(self, rhs: Self) -> Self {
		Self(self.0.zip(rhs.0, f32::mul))
	}
}

impl Mul<f32> for Color {
	type Output = Self;
	#[inline]
	fn mul(self, rhs: f32) -> Self {
		Self(self.0 * rhs)
	}
}

impl Div<f32> for Color {
	type Output = Self;
	#[inline]
	fn div(self, rhs: f32) -> Self {
		Self(self.0 / rhs)
	}
}

impl Mul<Color> for f32 {
	type Output = Color;
	#[inline]
	fn mul(self, rhs: Color) -> Color {
		Color(self * rhs.0)
	}
}

impl Index<usize> for Color {
	type Output = f32;
	#[inline]
	fn index(&self, i: usize) -> &f32 {
		&self.0[i]
	}
}

impl std::fmt::Display for Color {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({}, {}, {})", self[0], self[1], self[2])
	}
}

// clip color value between 0 and 1
#[inline]
fn clip(v: f32) -> f32 {
	if v < 0.0 {
		return 0.0;
	}
	if v > 1.0 {
		return 1.0;
	}
	v
}

/// SRGB to linear gamma conversion.
pub const SRGB_TO_LINEAR: [f32; 256] = [
	0.00000000, 0.00030353, 0.00060705, 0.00091058, 0.00121411, 0.00151763, 0.00182116, 0.00212469, 0.00242822, 0.00273174, 0.00303527, 0.00334654, 0.00367651, 0.00402472, 0.00439144, 0.00477695,
	0.00518152, 0.00560539, 0.00604883, 0.00651209, 0.00699541, 0.00749903, 0.00802319, 0.00856813, 0.00913406, 0.00972122, 0.01032982, 0.01096009, 0.01161225, 0.01228649, 0.01298303, 0.01370208,
	0.01444384, 0.01520851, 0.01599629, 0.01680738, 0.01764195, 0.01850022, 0.01938236, 0.02028856, 0.02121901, 0.02217388, 0.02315337, 0.02415763, 0.02518686, 0.02624122, 0.02732089, 0.02842604,
	0.02955683, 0.03071344, 0.03189603, 0.03310477, 0.03433981, 0.03560131, 0.03688945, 0.03820437, 0.03954624, 0.04091520, 0.04231141, 0.04373503, 0.04518620, 0.04666509, 0.04817182, 0.04970657,
	0.05126946, 0.05286065, 0.05448028, 0.05612849, 0.05780543, 0.05951124, 0.06124605, 0.06301002, 0.06480327, 0.06662594, 0.06847817, 0.07036010, 0.07227185, 0.07421357, 0.07618538, 0.07818742,
	0.08021982, 0.08228271, 0.08437621, 0.08650046, 0.08865559, 0.09084171, 0.09305896, 0.09530747, 0.09758735, 0.09989873, 0.10224173, 0.10461648, 0.10702310, 0.10946171, 0.11193243, 0.11443537,
	0.11697067, 0.11953843, 0.12213877, 0.12477182, 0.12743768, 0.13013648, 0.13286832, 0.13563333, 0.13843162, 0.14126329, 0.14412847, 0.14702727, 0.14995979, 0.15292615, 0.15592646, 0.15896084,
	0.16202938, 0.16513219, 0.16826940, 0.17144110, 0.17464740, 0.17788842, 0.18116424, 0.18447499, 0.18782077, 0.19120168, 0.19461783, 0.19806932, 0.20155625, 0.20507874, 0.20863687, 0.21223076,
	0.21586050, 0.21952620, 0.22322796, 0.22696587, 0.23074005, 0.23455058, 0.23839757, 0.24228112, 0.24620133, 0.25015828, 0.25415209, 0.25818285, 0.26225066, 0.26635560, 0.27049779, 0.27467731,
	0.27889426, 0.28314874, 0.28744084, 0.29177065, 0.29613827, 0.30054379, 0.30498731, 0.30946892, 0.31398871, 0.31854678, 0.32314321, 0.32777810, 0.33245154, 0.33716362, 0.34191442, 0.34670406,
	0.35153260, 0.35640014, 0.36130678, 0.36625260, 0.37123768, 0.37626212, 0.38132601, 0.38642943, 0.39157248, 0.39675523, 0.40197778, 0.40724021, 0.41254261, 0.41788507, 0.42326767, 0.42869050,
	0.43415364, 0.43965717, 0.44520119, 0.45078578, 0.45641102, 0.46207700, 0.46778380, 0.47353150, 0.47932018, 0.48514994, 0.49102085, 0.49693300, 0.50288646, 0.50888132, 0.51491767, 0.52099557,
	0.52711513, 0.53327640, 0.53947949, 0.54572446, 0.55201140, 0.55834039, 0.56471151, 0.57112483, 0.57758044, 0.58407842, 0.59061884, 0.59720179, 0.60382734, 0.61049557, 0.61720656, 0.62396039,
	0.63075714, 0.63759687, 0.64447968, 0.65140564, 0.65837482, 0.66538730, 0.67244316, 0.67954247, 0.68668531, 0.69387176, 0.70110189, 0.70837578, 0.71569350, 0.72305513, 0.73046074, 0.73791041,
	0.74540421, 0.75294222, 0.76052450, 0.76815115, 0.77582222, 0.78353779, 0.79129794, 0.79910274, 0.80695226, 0.81484657, 0.82278575, 0.83076988, 0.83879901, 0.84687323, 0.85499261, 0.86315721,
	0.87136712, 0.87962240, 0.88792312, 0.89626935, 0.90466117, 0.91309865, 0.92158186, 0.93011086, 0.93868573, 0.94730654, 0.95597335, 0.96468625, 0.97344529, 0.98225055, 0.99110210, 1.00000000,
];
