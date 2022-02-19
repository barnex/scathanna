use std::fmt::Debug;
use std::fmt::Display;
use std::ops::*;

pub trait Number: Sized + Copy + PartialOrd + PartialEq + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self> + Display + Debug + 'static {
	fn partial_min(self, other: Self) -> Self;
	fn partial_max(self, other: Self) -> Self;
	const ZERO: Self;
	const ONE: Self;
}

impl Number for f32 {
	#[inline]
	fn partial_min(self, other: Self) -> Self {
		Self::min(self, other)
	}
	#[inline]
	fn partial_max(self, other: Self) -> Self {
		Self::max(self, other)
	}
	const ZERO: Self = 0.0;
	const ONE: Self = 1.0;
}

impl Number for f64 {
	#[inline]
	fn partial_min(self, other: Self) -> Self {
		Self::min(self, other)
	}
	#[inline]
	fn partial_max(self, other: Self) -> Self {
		Self::max(self, other)
	}
	const ZERO: Self = 0.0;
	const ONE: Self = 1.0;
}

impl Number for i32 {
	#[inline]
	fn partial_min(self, other: Self) -> Self {
		Ord::min(self, other)
	}
	#[inline]
	fn partial_max(self, other: Self) -> Self {
		Ord::max(self, other)
	}
	const ZERO: Self = 0;
	const ONE: Self = 1;
}

impl Number for u32 {
	#[inline]
	fn partial_min(self, other: Self) -> Self {
		Ord::min(self, other)
	}
	#[inline]
	fn partial_max(self, other: Self) -> Self {
		Ord::max(self, other)
	}
	const ZERO: Self = 0;
	const ONE: Self = 1;
}

/// 'Any' floating point type. I.e. f32 or f64.
pub trait Float: Number + Neg<Output = Self> {
	fn sqrt(self) -> Self;
	fn is_finite(self) -> bool;
	fn as_f64(self) -> f64;
}

impl Float for f32 {
	#[inline]
	fn sqrt(self) -> Self {
		self.sqrt()
	}
	fn is_finite(self) -> bool {
		self.is_finite()
	}
	fn as_f64(self) -> f64 {
		self as f64
	}
}

impl Float for f64 {
	#[inline]
	fn sqrt(self) -> Self {
		self.sqrt()
	}
	fn is_finite(self) -> bool {
		self.is_finite()
	}
	fn as_f64(self) -> f64 {
		self
	}
}
