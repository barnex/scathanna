/// 'Any' floating point type. I.e. f32 or f64.
pub trait Float: PartialEq {
	fn sqrt(self) -> Self;
	const ZERO: Self;
	const ONE: Self;
}

impl Float for f32 {
	#[inline]
	fn sqrt(self) -> Self {
		f32::sqrt(self)
	}
	const ZERO: Self = 0.0f32;
	const ONE: Self = 1.0f32;
}

impl Float for f64 {
	#[inline]
	fn sqrt(self) -> Self {
		f64::sqrt(self)
	}
	const ZERO: Self = 0.0f64;
	const ONE: Self = 1.0f64;
}
