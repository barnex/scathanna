/// Apply is automatically implemented for every (sized) type,
/// and provides a method `with` that applies a function. E.g.
///
///  Player::new().with(|p| p.position = vec3(1.0, 2.0, 3.0))
///
/// This avoids the need for explicit mutability like
///
///  let mut player = Player::new();
///  player.position = x;
///  player
///
pub trait Apply {
	/// Applies function `f` to `self`, in-place.
	fn apply<F: FnOnce(&mut Self)>(&mut self, f: F);

	/// Applies function `f` to `self`, returns the modified `self`.
	#[inline]
	fn with<F: FnOnce(&mut Self)>(mut self, f: F) -> Self
	where
		Self: Sized,
	{
		f(&mut self);
		self
	}
}

impl<T> Apply for T {
	#[inline]
	fn apply<F: FnOnce(&mut Self)>(&mut self, f: F) {
		f(self)
	}
}
