use super::number::*;
use super::vector2::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::*;

/// Generic 3-component vector.
#[derive(Clone, Copy, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Vector3<T>([T; 3]);

impl<T> Vector3<T> {
	#[inline]
	pub const fn new(x: T, y: T, z: T) -> Self {
		Self([x, y, z])
	}
}

impl<T> Vector3<T>
where
	T: Copy,
{
	/// X-component.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec3(1,2,3).x(), 1);
	/// ```
	#[inline]
	pub fn x(&self) -> T {
		self[0]
	}

	/// Y-component.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec3(1,2,3).y(), 2);
	/// ```
	#[inline]
	pub fn y(&self) -> T {
		self[1]
	}

	/// Z-component.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec3(1,2,3).z(), 3);
	/// ```
	#[inline]
	pub fn z(&self) -> T {
		self[2]
	}

	/// X and Y components
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec3(1,2,3).xy(), ivec2(1,2));
	/// ```
	#[inline]
	pub fn xy(&self) -> Vector2<T> {
		Vector2::new(self.x(), self.y())
	}

	/// X and Z components
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec3(1,2,3).xz(), ivec2(1,3));
	/// ```
	#[inline]
	pub fn xz(&self) -> Vector2<T> {
		Vector2::new(self.x(), self.z())
	}

	/// Y and Z components
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec3(1,2,3).yz(), ivec2(2,3));
	/// ```
	#[inline]
	pub fn yz(&self) -> Vector2<T> {
		Vector2::new(self.y(), self.z())
	}
}

impl<T> Index<usize> for Vector3<T> {
	type Output = T;

	#[inline]
	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index]
	}
}

impl<T> IndexMut<usize> for Vector3<T> {
	#[inline]
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0[index]
	}
}

// --------------------------------------------------------------- higher functionality

impl<T> Vector3<T>
where
	T: Copy,
{
	/// Apply a function to each element.
	/// ```
	/// # use vector::*;
	/// assert_eq!(vec3(1.0, 4.0, 9.0).map(f32::sqrt), vec3(1.0, 2.0, 3.0));
	/// ```
	#[must_use]
	#[inline]
	pub fn map<F, U>(&self, f: F) -> Vector3<U>
	where
		F: Fn(T) -> U,
	{
		Vector3::<U>([f(self[0]), f(self[1]), f(self[2])])
	}

	/// Reduces the elements to a single one, by repeatedly applying a reducing operation.
	/// ```
	/// # use vector::*;
	/// # use std::ops::Add;
	/// assert_eq!(ivec3(1, 2, 3).reduce(i32::add), 6);
	/// ```
	#[must_use]
	#[inline]
	pub fn reduce<F>(&self, f: F) -> T
	where
		F: Fn(T, T) -> T,
	{
		f(f(self[0], self[1]), self[2])
	}

	/// Applies a function to pairs of elements.
	/// ```
	/// # use vector::*;
	/// # use std::ops::Mul;
	/// assert_eq!(ivec3(1, 2, 3).zip(ivec3(1, 10, 100), i32::mul), ivec3(1, 20, 300));
	/// ```
	#[must_use]
	#[inline]
	pub fn zip<F, U>(self, rhs: Self, f: F) -> Vector3<U>
	where
		F: Fn(T, T) -> U,
	{
		Vector3::<U>([f(self[0], rhs[0]), f(self[1], rhs[1]), f(self[2], rhs[2])])
	}

	#[inline]
	fn zip_assign<F>(&mut self, rhs: Self, f: F)
	where
		F: Fn(&mut T, T),
	{
		f(&mut self[0], rhs[0]);
		f(&mut self[1], rhs[1]);
		f(&mut self[2], rhs[2]);
	}

	pub fn iter(self) -> impl ExactSizeIterator<Item = T> {
		self.0.into_iter()
	}
}

// --------------------------------------------------------------- operators

impl<T> Add for Vector3<T>
where
	T: Add<T, Output = T> + Copy,
{
	type Output = Self;

	/// Element-wise sum.
	/// ```
	/// # use vector::*;
	///	assert_eq!(ivec3(1, 2, 3) + ivec3(3, 4, 5), ivec3(4, 6, 8));
	/// ```
	#[inline]
	fn add(self, rhs: Self) -> Self::Output {
		self.zip(rhs, T::add)
	}
}

impl<T> AddAssign for Vector3<T>
where
	T: AddAssign + Copy,
{
	/// Element-wise sum.
	/// ```
	/// # use vector::*;
	/// let mut a = ivec3(1, 2, 3);
	/// a += ivec3(4, 5, 6);
	///	assert_eq!(a, ivec3(5, 7, 9));
	/// ```
	#[inline]
	fn add_assign(&mut self, rhs: Self) {
		self.zip_assign(rhs, T::add_assign)
	}
}

impl<T> Div<T> for Vector3<T>
where
	T: Div<T, Output = T> + Copy,
{
	type Output = Self;

	/// Element-wise division by a constant.
	/// ```
	/// # use vector::*;
	/// assert_eq!(uvec3(2, 4, 8) / 2, uvec3(1, 2, 4));
	/// ```
	#[inline]
	fn div(self, rhs: T) -> Self::Output {
		self.map(|v| v / rhs)
	}
}

impl<T> Vector3<T>
where
	T: Div<T, Output = T> + Copy,
{
	/// Element-wise division.
	/// ```
	/// # use vector::*;
	///	assert_eq!(ivec3(3, 8, 10).div3(ivec3(3, 2, 1)), ivec3(1, 4, 10));
	/// ```
	pub fn div3(self, rhs: Self) -> Self {
		self.zip(rhs, T::div)
	}
}

impl<T> Mul<T> for Vector3<T>
where
	T: Mul<T, Output = T> + Copy,
{
	type Output = Self;

	/// Element-wise multiplication by a constant.
	/// ```
	/// # use vector::*;
	///	assert_eq!(ivec3(1, 2, 3) * 2, ivec3(2, 4, 6));
	/// ```
	#[inline]
	fn mul(self, rhs: T) -> Self::Output {
		self.map(|v| v * rhs)
	}
}

impl<T> Vector3<T>
where
	T: Mul<T, Output = T> + Copy,
{
	/// Element-wise multiplication.
	/// ```
	/// # use vector::*;
	///	assert_eq!(ivec3(1,2,3).mul3(ivec3(3,4,5)), ivec3(3,8,15));
	/// ```
	#[inline]
	pub fn mul3(self, rhs: Self) -> Self {
		self.zip(rhs, T::mul)
	}
}

impl<T> MulAssign<T> for Vector3<T>
where
	T: MulAssign + Copy,
{
	/// Multiply all elements by a constant.
	/// ```
	/// # use vector::*;
	/// let mut v = ivec3(1, 2, 3);
	/// v *= 2;
	/// assert_eq!(v, ivec3(2, 4, 6));
	/// ```
	#[inline]
	fn mul_assign(&mut self, rhs: T) {
		self[0] *= rhs;
		self[1] *= rhs;
		self[2] *= rhs;
	}
}

impl<T> Neg for Vector3<T>
where
	T: Neg<Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn neg(self) -> Self::Output {
		self.map(T::neg)
	}
}

impl<T> Sub for Vector3<T>
where
	T: Sub<T, Output = T> + Copy,
{
	type Output = Self;

	/// Element-wise subtraction.
	/// ```
	/// # use vector::*;
	///	assert_eq!(ivec3(4, 6, 8) - ivec3(1, 2, 3), ivec3(3, 4, 5));
	/// ```
	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		self.zip(rhs, T::sub)
	}
}

impl<T> SubAssign for Vector3<T>
where
	T: SubAssign + Copy,
{
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.zip_assign(rhs, T::sub_assign)
	}
}

impl<T> Display for Vector3<T>
where
	T: Copy + Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({}, {}, {})", self[0], self[1], self[2])
	}
}

impl<T> Debug for Vector3<T>
where
	T: Copy + Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({:?}, {:?}, {:?})", self[0], self[1], self[2])
	}
}

impl<T> Vector3<T>
where
	T: Add<T, Output = T> + Mul<T, Output = T> + Sub<T, Output = T> + Copy,
{
	/// Dot (inner) product.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec3(0,0,1).dot(ivec3(2,3,4)), 4);
	/// ```
	#[inline]
	pub fn dot(self, rhs: Vector3<T>) -> T {
		self.zip(rhs, T::mul).reduce(T::add)
	}

	/// Length squared (norm squared).
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec3(1,3,4).len2(), 26);
	/// ```
	#[inline]
	pub fn len2(self) -> T {
		self.dot(self)
	}

	/// Cross product.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec3(1,0,0).cross(ivec3(0,1,0)), ivec3(0,0,1));
	/// ```
	#[inline]
	pub fn cross(self, rhs: Self) -> Self {
		Self([
			self.y() * rhs.z() - self.z() * rhs.y(),
			self.z() * rhs.x() - self.x() * rhs.z(),
			self.x() * rhs.y() - self.y() * rhs.x(),
		])
	}
}

impl<T> Vector3<T>
where
	T: Float,
{
	/// Length (norm).
	/// ```
	/// # use vector::*;
	/// assert_eq!(vec3(0.0, 3.0, 4.0).len(), 5.0);
	/// ```
	#[inline]
	pub fn len(self) -> T {
		self.len2().sqrt()
	}

	/// Returns a vector with the same direction but unit length.
	/// ```
	/// # use vector::*;
	/// assert_eq!(dvec3(0.0, 2.0, 0.0).normalized(), dvec3(0.0, 1.0, 0.0));
	/// ```
	#[inline]
	#[must_use]
	pub fn normalized(self) -> Self {
		self / self.len()
	}

	pub fn is_normalized(self) -> bool {
		self == self.normalized()
	}

	pub fn is_finite(self) -> bool {
		self.iter().all(T::is_finite)
	}

	/// Normalizes a vector, unless it has length 0
	/// (which would result in NaN's).
	/// ```
	/// # use vector::*;
	/// assert_eq!(dvec3(0.0, 2.0, 0.0).safe_normalized(), dvec3(0.0, 1.0, 0.0));
	/// assert_eq!(dvec3(0.0, 0.0, 0.0).safe_normalized(), dvec3(0.0, 0.0, 0.0));
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
	/// let mut v = vec4(2.0, 0.0, 0.0, 0.0);
	/// v.normalize();
	/// assert_eq!(v, vec4(1.0, 0.0, 0.0, 0.0));
	/// ```
	#[inline]
	pub fn normalize(&mut self) {
		*self = self.normalized()
	}

	/// The zero vector.
	pub const ZERO: Self = Self([T::ZERO, T::ZERO, T::ZERO]);

	/// All ones.
	pub const ONES: Self = Self([T::ONE, T::ONE, T::ONE]);

	/// Unit vector along X.
	pub const EX: Self = Self([T::ONE, T::ZERO, T::ZERO]);

	/// Unit vector along Y.
	pub const EY: Self = Self([T::ZERO, T::ONE, T::ZERO]);

	/// Unit vector along Z.
	pub const EZ: Self = Self([T::ZERO, T::ZERO, T::ONE]);
}

impl<T> Into<(T, T, T)> for Vector3<T>
where
	T: Copy,
{
	#[inline]
	fn into(self) -> (T, T, T) {
		(self[0], self[1], self[2])
	}
}

impl<T> From<(T, T, T)> for Vector3<T> {
	#[inline]
	fn from(t: (T, T, T)) -> Self {
		Self([t.0, t.1, t.2])
	}
}

impl<T> Vector3<T>
where
	T: PartialOrd,
{
	/// Index of the largest element.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec3(1,0,0).argmax(), 0);
	/// assert_eq!(ivec3(0,1,0).argmax(), 1);
	/// assert_eq!(ivec3(0,0,1).argmax(), 2);
	/// ```
	pub fn argmax(&self) -> usize {
		let mut arg = 0;
		for i in 1..=2 {
			if self[i] > self[arg] {
				arg = i
			}
		}
		arg
	}
}

impl<T> Vector3<T>
where
	T: Copy,
{
	/// Remove element at index.
	/// ```
	/// # use vector::*;
	/// assert_eq!(ivec3(10,20,30).remove(0), ivec2(20, 30));
	/// assert_eq!(ivec3(10,20,30).remove(1), ivec2(10, 30));
	/// assert_eq!(ivec3(10,20,30).remove(2), ivec2(10, 20));
	/// ```
	#[inline]
	#[must_use]
	pub fn remove(&self, i: usize) -> Vector2<T> {
		let i: usize = i.into();
		match i {
			0 => Vector2::new(self.y(), self.z()),
			1 => Vector2::new(self.x(), self.z()),
			2 => Vector2::new(self.x(), self.y()),
			_ => panic!("index out of bounds: {}", i),
		}
	}
}
