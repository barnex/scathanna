use super::*;

pub trait Convert<U> {
	fn convert(self) -> U;
}

impl<T> Vector2<T>
where
	T: Copy,
{
	#[inline]
	pub fn convert<U>(self) -> Vector2<U>
	where
		T: Convert<U>,
	{
		self.map(T::convert)
	}
}

impl<T> Vector3<T>
where
	T: Copy,
{
	#[inline]
	pub fn convert<U>(self) -> Vector3<U>
	where
		T: Convert<U>,
	{
		self.map(T::convert)
	}
}

impl<T> Vector4<T>
where
	T: Copy,
{
	#[inline]
	pub fn convert<U>(self) -> Vector4<U>
	where
		T: Convert<U>,
	{
		self.map(T::convert)
	}
}

impl Convert<f32> for f64 {
	fn convert(self) -> f32 {
		self as f32
	}
}

impl Convert<i32> for f64 {
	fn convert(self) -> i32 {
		self as i32
	}
}

impl Convert<f64> for f32 {
	fn convert(self) -> f64 {
		self as f64
	}
}

impl Convert<i32> for f32 {
	fn convert(self) -> i32 {
		self as i32
	}
}

impl Convert<i32> for u32 {
	fn convert(self) -> i32 {
		self as i32
	}
}
