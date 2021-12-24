use crate::Vector3;

use super::float::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::*;

#[derive(Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq, Default)]
pub struct Vector2<T> {
	pub x: T,
	pub y: T,
}

impl<T> Vector2<T> {
	#[inline]
	pub const fn new(x: T, y: T) -> Self {
		Self { x, y }
	}
}

impl<T> Add for Vector2<T>
where
	T: Add<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn add(self, rhs: Self) -> Self::Output {
		Self { x: self.x + rhs.x, y: self.y + rhs.y }
	}
}

impl<T> AddAssign for Vector2<T>
where
	T: AddAssign + Copy,
{
	#[inline]
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl<T> Div<T> for Vector2<T>
where
	T: Div<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn div(self, rhs: T) -> Self::Output {
		Self { x: self.x / rhs, y: self.y / rhs }
	}
}

impl<T> Mul<T> for Vector2<T>
where
	T: Mul<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn mul(self, rhs: T) -> Self::Output {
		Self { x: self.x * rhs, y: self.y * rhs }
	}
}

impl<T> Vector2<T>
where
	T: Mul<T, Output = T> + Copy,
{
	/// Pointwise multiplication.
	#[inline]
	pub fn mul2(self, rhs: Self) -> Self {
		self.zip(rhs, T::mul)
	}
}

impl<T> Vector2<T>
where
	T: Mul<T, Output = T> + Copy,
{
	#[must_use]
	#[inline]
	pub fn zip<F, U>(self, rhs: Self, f: F) -> Vector2<U>
	where
		F: Fn(T, T) -> U,
	{
		Vector2 {
			x: f(self.x, rhs.x),
			y: f(self.y, rhs.y),
		}
	}
}

impl<T> MulAssign<T> for Vector2<T>
where
	T: MulAssign + Copy,
{
	#[inline]
	fn mul_assign(&mut self, rhs: T) {
		self.x *= rhs;
		self.y *= rhs;
	}
}

impl<T> Neg for Vector2<T>
where
	T: Neg<Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn neg(self) -> Self::Output {
		Self { x: -self.x, y: -self.y }
	}
}

impl<T> Sub for Vector2<T>
where
	T: Sub<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		Self { x: self.x - rhs.x, y: self.y - rhs.y }
	}
}

impl<T> SubAssign for Vector2<T>
where
	T: SubAssign + Copy,
{
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
	}
}

impl<T> Display for Vector2<T>
where
	T: Copy + Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({}, {})", self.x, self.y)
	}
}

impl<T> Debug for Vector2<T>
where
	T: Copy + Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({:?}, {:?})", self.x, self.y)
	}
}

impl<T> Vector2<T>
where
	T: Add<T, Output = T> + Mul<T, Output = T> + Sub<T, Output = T> + Copy,
{
	/// Dot (inner) product.
	#[inline]
	pub fn dot(self, rhs: Vector2<T>) -> T {
		self.x * rhs.x + self.y * rhs.y
	}

	/// Length squared (norm squared).
	#[inline]
	pub fn len2(self) -> T {
		self.dot(self)
	}
}

impl<T> Vector2<T>
where
	T: Add<T, Output = T> + Mul<T, Output = T> + Sub<T, Output = T> + Div<T, Output = T> + Copy + Float,
{
	/// Length (norm).
	#[inline]
	pub fn len(self) -> T {
		self.len2().sqrt()
	}

	/// Returns a vector with the same direction but unit length.
	#[inline]
	#[must_use]
	pub fn normalized(self) -> Self {
		self * (T::ONE / self.len())
	}

	/// Re-scale the vector to unit length.
	#[inline]
	pub fn normalize(&mut self) {
		*self = self.normalized()
	}

	/// The zero vector.
	pub const ZERO: Self = Self { x: T::ZERO, y: T::ZERO };

	/// All ones.
	pub const ONES: Self = Self { x: T::ONE, y: T::ONE };

	/// Unit vector along X.
	pub const EX: Self = Self { x: T::ONE, y: T::ZERO };

	/// Unit vector along Y.
	pub const EY: Self = Self { x: T::ZERO, y: T::ONE };
}

impl<T> Into<(T, T)> for Vector2<T> {
	#[inline]
	fn into(self) -> (T, T) {
		(self.x, self.y)
	}
}

impl<T> From<(T, T)> for Vector2<T> {
	#[inline]
	fn from(t: (T, T)) -> Self {
		Self { x: t.0, y: t.1 }
	}
}

impl<T> Vector2<T>
where
	T: Copy,
{
	#[must_use]
	#[inline]
	pub fn map<F, U>(&self, f: F) -> Vector2<U>
	where
		F: Fn(T) -> U,
	{
		Vector2 { x: f(self.x), y: f(self.y) }
	}
}

impl<T> Vector2<T>
where
	T: PartialOrd + Copy,
{
	pub fn min(a: Self, b: Self) -> Self {
		Self { x: min(a.x, b.x), y: min(a.y, b.y) }
	}
}

impl<T> Vector2<T>
where
	T: PartialOrd + Copy,
{
	pub fn max(a: Self, b: Self) -> Self {
		Self { x: max(a.x, b.x), y: max(a.y, b.y) }
	}
}

fn min<T: PartialOrd + Copy>(a: T, b: T) -> T {
	if a < b {
		a
	} else {
		b
	}
}

fn max<T: PartialOrd + Copy>(a: T, b: T) -> T {
	if a > b {
		a
	} else {
		b
	}
}

impl<T> Vector2<T>
where
	T: Copy,
{
	#[must_use]
	pub fn insert<U: Into<usize>>(self, index: U, value: T) -> Vector3<T> {
		let index: usize = index.into();
		match index {
			0 => Vector3::new(value, self.x, self.y),
			1 => Vector3::new(self.x, value, self.y),
			2 => Vector3::new(self.x, self.y, value),
			i => panic!("index out of bounds: {}", i),
		}
	}
}
