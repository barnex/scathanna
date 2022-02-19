use super::number::*;
use crate::Vector3;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Index;
use std::ops::*;

/// Generic 2-component vector.
#[derive(Clone, Copy, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Vector2<T>([T; 2]);

impl<T> Vector2<T> {
	#[inline]
	pub const fn new(x: T, y: T) -> Self {
		Self([x, y])
	}
}

impl<T> Vector2<T>
where
	T: Copy,
{
	/// X-component.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec2(1,2).x(), 1);
	/// ```
	#[inline]
	pub fn x(&self) -> T {
		self[0]
	}

	/// Y-component.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec2(1,2).y(), 2);
	/// ```
	#[inline]
	pub fn y(&self) -> T {
		self[1]
	}
}

impl<T> Index<usize> for Vector2<T> {
	type Output = T;

	#[inline]
	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index]
	}
}

impl<T> IndexMut<usize> for Vector2<T> {
	#[inline]
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0[index]
	}
}

// --------------------------------------------------------------- higher functionality

impl<T> Vector2<T>
where
	T: Copy,
{
	/// Apply a function to each element.
	/// ```
	/// # use vector::*;
	/// assert_eq!(vec2(1.0, 4.0).map(f32::sqrt), vec2(1.0, 2.0));
	/// ```
	#[must_use]
	#[inline]
	pub fn map<F, U>(&self, f: F) -> Vector2<U>
	where
		F: Fn(T) -> U,
	{
		Vector2::<U>([f(self[0]), f(self[1])])
	}

	/// Reduces the elements to a single one, by repeatedly applying a reducing operation.
	/// ```
	/// # use vector::*;
	/// # use std::ops::Add;
	/// assert_eq!(ivec2(1, 2).reduce(i32::add), 3);
	/// ```
	#[must_use]
	#[inline]
	pub fn reduce<F>(&self, f: F) -> T
	where
		F: Fn(T, T) -> T,
	{
		f(self[0], self[1])
	}

	/// Applies a function to pairs of elements.
	/// ```
	/// # use vector::*;
	/// # use std::ops::Mul;
	/// assert_eq!(ivec2(2, 3).zip(ivec2(10, 100), i32::mul), ivec2(20, 300));
	/// ```
	#[must_use]
	#[inline]
	pub fn zip<F, U>(self, rhs: Self, f: F) -> Vector2<U>
	where
		F: Fn(T, T) -> U,
	{
		Vector2::<U>([f(self[0], rhs[0]), f(self[1], rhs[1])])
	}

	#[inline]
	fn zip_assign<F>(&mut self, rhs: Self, f: F)
	where
		F: Fn(&mut T, T),
	{
		f(&mut self[0], rhs[0]);
		f(&mut self[1], rhs[1]);
	}

	pub fn iter(self) -> impl ExactSizeIterator<Item = T> {
		self.0.into_iter()
	}
}

// --------------------------------------------------------------- operators

impl<T> Add for Vector2<T>
where
	T: Add<T, Output = T> + Copy,
{
	type Output = Self;

	/// Element-wise sum.
	/// ```
	/// # use vector::*;
	///	assert_eq!(ivec2(1, 2) + ivec2(3, 4), ivec2(4, 6));
	/// ```
	#[inline]
	fn add(self, rhs: Self) -> Self::Output {
		self.zip(rhs, T::add)
	}
}

impl<T> AddAssign for Vector2<T>
where
	T: AddAssign + Copy,
{
	/// Element-wise sum.
	/// ```
	/// # use vector::*;
	/// let mut a = ivec2(1, 2);
	/// a += ivec2(4, 5);
	///	assert_eq!(a, ivec2(5, 7));
	/// ```
	#[inline]
	fn add_assign(&mut self, rhs: Self) {
		self.zip_assign(rhs, T::add_assign)
	}
}

impl<T> Div<T> for Vector2<T>
where
	T: Div<T, Output = T> + Copy,
{
	type Output = Self;

	/// Element-wise division by a constant.
	/// ```
	/// use vector::*;
	///	assert_eq!(ivec2(10, 20) / 2, ivec2(5, 10));
	/// ```
	#[inline]
	fn div(self, rhs: T) -> Self::Output {
		self.map(|v| v / rhs)
	}
}

impl<T> Vector2<T>
where
	T: Div<T, Output = T> + Copy,
{
	/// Element-wise division.
	/// ```
	/// # use vector::*;
	///	assert_eq!(ivec2(3, 8).div2(ivec2(3, 2)), ivec2(1, 4));
	/// ```
	pub fn div2(self, rhs: Self) -> Self {
		self.zip(rhs, T::div)
	}
}

impl<T> Mul<T> for Vector2<T>
where
	T: Mul<T, Output = T> + Copy,
{
	type Output = Self;

	/// Element-wise multiplication by a constant.
	/// ```
	/// # use vector::*;
	///	assert_eq!(ivec2(2, 3) * 4, ivec2(8, 12));
	/// ```
	#[inline]
	fn mul(self, rhs: T) -> Self::Output {
		self.map(|v| v * rhs)
	}
}

impl<T> Vector2<T>
where
	T: Mul<T, Output = T> + Copy,
{
	/// Element-wise multiplication.
	/// ```
	/// # use vector::*;
	///	assert_eq!(ivec2(2, 3).mul2(ivec2(4, 5)), ivec2(8, 15));
	/// ```
	#[inline]
	pub fn mul2(self, rhs: Self) -> Self {
		self.zip(rhs, T::mul)
	}
}

impl<T> MulAssign<T> for Vector2<T>
where
	T: MulAssign + Copy,
{
	/// Multiply all elements by a constant.
	/// ```
	/// # use vector::*;
	/// let mut v = ivec2(1, 2);
	/// v *= 3;
	/// assert_eq!(v, ivec2(3, 6));
	/// ```
	#[inline]
	fn mul_assign(&mut self, rhs: T) {
		self[0] *= rhs;
		self[1] *= rhs;
	}
}

impl<T> Neg for Vector2<T>
where
	T: Neg<Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn neg(self) -> Self::Output {
		self.map(T::neg)
	}
}

impl<T> Sub for Vector2<T>
where
	T: Sub<T, Output = T> + Copy,
{
	type Output = Self;

	/// Element-wise subtraction.
	/// ```
	/// # use vector::*;
	///	assert_eq!(ivec2(4, 6) - ivec2(1, 2), ivec2(3, 4));
	/// ```
	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		self.zip(rhs, T::sub)
	}
}

impl<T> SubAssign for Vector2<T>
where
	T: SubAssign + Copy,
{
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.zip_assign(rhs, T::sub_assign)
	}
}

impl<T> Display for Vector2<T>
where
	T: Copy + Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({}, {})", self[0], self[1])
	}
}

impl<T> Debug for Vector2<T>
where
	T: Copy + Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({:?}, {:?})", self[0], self[1])
	}
}

impl<T> Vector2<T>
where
	T: Add<T, Output = T> + Mul<T, Output = T> + Sub<T, Output = T> + Copy,
{
	/// Dot (inner) product.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec2(1,2).dot(ivec2(3,4)), 11);
	/// ```
	#[inline]
	pub fn dot(self, rhs: Vector2<T>) -> T {
		self.zip(rhs, T::mul).reduce(T::add)
	}

	/// Length squared (norm squared).
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec2(3,4).len2(), 25);
	/// ```
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
	/// ```
	/// # use vector::*;
	/// assert_eq!(vec2(3.0, 4.0).len(), 5.0);
	/// ```
	#[inline]
	pub fn len(self) -> T {
		self.len2().sqrt()
	}

	/// Returns a vector with the same direction but unit length.
	/// ```
	/// # use vector::*;
	/// assert_eq!(vec2(2.0, 0.0).normalized(), vec2(1.0, 0.0));
	/// ```
	#[inline]
	#[must_use]
	pub fn normalized(self) -> Self {
		self / self.len()
	}

	/// Normalizes a vector, unless it has length 0
	/// (which would result in NaN's).
	/// ```
	/// # use vector::*;
	/// assert_eq!(dvec2(2.0, 0.0).safe_normalized(), dvec2(1.0, 0.0));
	/// assert_eq!(dvec2(0.0, 0.0).safe_normalized(), dvec2(0.0, 0.0));
	/// ```
	#[inline]
	#[must_use]
	pub fn safe_normalized(self) -> Self {
		let len = self.len();
		if len == T::ZERO {
			Self::ZERO
		} else {
			self / len
		}
	}

	/// Re-scale the vector to unit length.
	/// ```
	/// use vector::*;
	/// let mut v = vec2(2.0, 0.0);
	/// v.normalize();
	/// assert_eq!(v, vec2(1.0, 0.0));
	/// ```
	#[inline]
	pub fn normalize(&mut self) {
		*self = self.normalized()
	}

	pub fn is_normalized(self) -> bool {
		self == self.normalized()
	}

	pub fn is_finite(self) -> bool {
		self.iter().all(T::is_finite)
	}

	/// The zero vector.
	pub const ZERO: Self = Self([T::ZERO, T::ZERO]);

	/// All ones.
	pub const ONES: Self = Self([T::ONE, T::ONE]);

	/// Unit vector along X.
	pub const EX: Self = Self([T::ONE, T::ZERO]);

	/// Unit vector along Y.
	pub const EY: Self = Self([T::ZERO, T::ONE]);
}

impl<T> Into<(T, T)> for Vector2<T>
where
	T: Copy,
{
	#[inline]
	fn into(self) -> (T, T) {
		(self[0], self[1])
	}
}

impl<T> From<(T, T)> for Vector2<T> {
	#[inline]
	fn from(t: (T, T)) -> Self {
		Self([t.0, t.1])
	}
}

impl<T> Into<[T; 2]> for Vector2<T>
where
	T: Copy,
{
	#[inline]
	fn into(self) -> [T; 2] {
		self.0
	}
}

impl<T> From<[T; 2]> for Vector2<T> {
	#[inline]
	fn from(v: [T; 2]) -> Self {
		Self(v)
	}
}

impl<T> Vector2<T>
where
	T: PartialOrd,
{
	/// Index of the largest element.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec2(1,0).argmax(), 0);
	/// assert_eq!(ivec2(0,1).argmax(), 1);
	/// ```
	pub fn argmax(&self) -> usize {
		if self[1] > self[0] {
			1
		} else {
			0
		}
	}
}

impl<T> Vector2<T>
where
	T: Copy,
{
	/// Insert element at index.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec2(1,2).insert(0, 42), ivec3(42, 1, 2));
	/// assert_eq!(ivec2(1,2).insert(1, 42), ivec3(1, 42, 2));
	/// assert_eq!(ivec2(1,2).insert(2, 42), ivec3(1, 2, 42));
	/// ```
	#[inline]
	#[must_use]
	pub fn insert(self, index: usize, value: T) -> Vector3<T> {
		match index {
			0 => Vector3::new(value, self[0], self[1]),
			1 => Vector3::new(self[0], value, self[1]),
			2 => Vector3::new(self[0], self[1], value),
			i => panic!("index out of bounds: {}", i),
		}
	}
}
