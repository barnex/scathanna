pub fn sizeof<T: Sized>(_v: T) -> i32 {
	std::mem::size_of::<T>() as i32
}
