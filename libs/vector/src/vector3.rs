use super::float::*;
use super::vector2::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct Vector3<T> {
	pub x: T,
	pub y: T,
	pub z: T,
}

impl<T> Vector3<T> {
	#[inline]
	pub const fn new(x: T, y: T, z: T) -> Self {
		Self { x, y, z }
	}
}

impl<T> Vector3<T>
where
	T: Copy,
{
	#[must_use]
	#[inline]
	pub fn map<F, U>(&self, f: F) -> Vector3<U>
	where
		F: Fn(T) -> U,
	{
		Vector3 {
			x: f(self.x),
			y: f(self.y),
			z: f(self.z),
		}
	}

	#[must_use]
	#[inline]
	pub fn zip<F, U>(self, rhs: Self, f: F) -> Vector3<U>
	where
		F: Fn(T, T) -> U,
	{
		Vector3 {
			x: f(self.x, rhs.x),
			y: f(self.y, rhs.y),
			z: f(self.z, rhs.z),
		}
	}

	#[must_use]
	#[inline]
	pub fn fold<F>(self, init: T, f: F) -> T
	where
		F: Fn(T, T) -> T,
	{
		f(f(f(init, self.x), self.y), self.z)
	}

	#[must_use]
	#[inline]
	pub fn reduce<F>(self, f: F) -> T
	where
		F: Fn(T, T) -> T,
	{
		f(f(self.x, self.y), self.z)
	}
}

impl<T> Add for Vector3<T>
where
	T: Add<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn add(self, rhs: Self) -> Self::Output {
		self.zip(rhs, T::add)
	}
}

impl<T> AddAssign for Vector3<T>
where
	T: AddAssign + Copy,
{
	#[inline]
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
	}
}

impl<T> Div<T> for Vector3<T>
where
	T: Div<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn div(self, rhs: T) -> Self::Output {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
			z: self.z / rhs,
		}
	}
}

impl<T> Vector3<T>
where
	T: Div<T, Output = T> + Copy,
{
	pub fn div3(self, rhs: Self) -> Self {
		Self {
			x: self.x / rhs.x,
			y: self.y / rhs.y,
			z: self.z / rhs.z,
		}
	}
}

impl<T> Mul<T> for Vector3<T>
where
	T: Mul<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn mul(self, rhs: T) -> Self::Output {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs,
		}
	}
}

impl<T> Vector3<T>
where
	T: Mul<T, Output = T> + Copy,
{
	/// Pointwise multiplication.
	#[inline]
	pub fn mul3(self, rhs: Self) -> Self {
		self.zip(rhs, T::mul)
	}
}

impl<T> MulAssign<T> for Vector3<T>
where
	T: MulAssign + Copy,
{
	#[inline]
	fn mul_assign(&mut self, rhs: T) {
		self.x *= rhs;
		self.y *= rhs;
		self.z *= rhs;
	}
}

impl<T> Neg for Vector3<T>
where
	T: Neg<Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn neg(self) -> Self::Output {
		Self { x: -self.x, y: -self.y, z: -self.z }
	}
}

impl<T> Sub for Vector3<T>
where
	T: Sub<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
		}
	}
}

impl<T> SubAssign for Vector3<T>
where
	T: SubAssign + Copy,
{
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
	}
}

impl<T> Display for Vector3<T>
where
	T: Copy + Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({}, {}, {})", self.x, self.y, self.z)
	}
}

impl<T> Debug for Vector3<T>
where
	T: Copy + Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({:?}, {:?}, {:?})", self.x, self.y, self.z)
	}
}

impl<T> Vector3<T>
where
	T: Add<T, Output = T> + Mul<T, Output = T> + Sub<T, Output = T> + Copy,
{
	/// Dot (inner) product.
	#[inline]
	pub fn dot(self, rhs: Vector3<T>) -> T {
		self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
	}

	/// Length squared (norm squared).
	#[inline]
	pub fn len2(self) -> T {
		self.dot(self)
	}

	/// Cross product.
	#[inline]
	pub fn cross(self, rhs: Self) -> Self {
		Self {
			x: self.y * rhs.z - self.z * rhs.y,
			y: self.z * rhs.x - self.x * rhs.z,
			z: self.x * rhs.y - self.y * rhs.x,
		}
	}
}

impl<T> Vector3<T>
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

	/// Returns a vector with the same direction but unit length.
	#[inline]
	#[must_use]
	pub fn safe_normalized(self) -> Self {
		if self == Self::ZERO {
			Self::ZERO
		} else {
			self * (T::ONE / self.len())
		}
	}

	/// Re-scale the vector to unit length.
	#[inline]
	pub fn normalize(&mut self) {
		*self = self.normalized()
	}

	/// The zero vector.
	pub const ZERO: Self = Self { x: T::ZERO, y: T::ZERO, z: T::ZERO };

	/// All ones.
	pub const ONES: Self = Self { x: T::ONE, y: T::ONE, z: T::ONE };

	/// Unit vector along X.
	pub const EX: Self = Self { x: T::ONE, y: T::ZERO, z: T::ZERO };

	/// Unit vector along Y.
	pub const EY: Self = Self { x: T::ZERO, y: T::ONE, z: T::ZERO };

	/// Unit vector along Z.
	pub const EZ: Self = Self { x: T::ZERO, y: T::ZERO, z: T::ONE };
}

impl<T> Into<(T, T, T)> for Vector3<T> {
	#[inline]
	fn into(self) -> (T, T, T) {
		(self.x, self.y, self.z)
	}
}

impl<T> From<(T, T, T)> for Vector3<T> {
	#[inline]
	fn from(t: (T, T, T)) -> Self {
		Self { x: t.0, y: t.1, z: t.2 }
	}
}

impl<T> Index<usize> for Vector3<T> {
	type Output = T;

	#[inline]
	fn index(&self, index: usize) -> &Self::Output {
		match index {
			0 => &self.x,
			1 => &self.y,
			2 => &self.z,
			_ => panic!("index out of bounds"),
		}
	}
}

impl<T> IndexMut<usize> for Vector3<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		match index {
			0 => &mut self.x,
			1 => &mut self.y,
			2 => &mut self.z,
			_ => panic!("index out of bounds: {}", index),
		}
	}
}

impl<T> Vector3<T>
where
	T: PartialOrd,
{
	pub fn argmax(&self) -> usize {
		let mut arg = 0;
		for i in 1..2 {
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
	#[inline]
	#[must_use]
	pub fn remove(&self, i: usize) -> Vector2<T> {
		let i: usize = i.into();
		match i {
			0 => Vector2::new(self.y, self.z),
			1 => Vector2::new(self.x, self.z),
			2 => Vector2::new(self.x, self.y),
			_ => panic!("index out of bounds: {}", i),
		}
	}

	#[inline]
	#[must_use]
	pub fn yz(&self) -> Vector2<T> {
		self.remove(0)
	}

	#[inline]
	#[must_use]
	pub fn xz(&self) -> Vector2<T> {
		self.remove(1)
	}

	#[inline]
	#[must_use]
	pub fn xy(&self) -> Vector2<T> {
		self.remove(2)
	}
}

impl<T> Serialize for Vector3<T>
where
	T: Serialize + Copy,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		[self.x, self.y, self.z].serialize(serializer)
	}
}

impl<'de, T> Deserialize<'de> for Vector3<T>
where
	T: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let [x, y, z] = <[T; 3]>::deserialize(deserializer)?;
		Ok(Self { x, y, z })
	}
}
