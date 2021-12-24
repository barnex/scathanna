use super::error::*;

/// Alias for "result with any type of error".
pub type Result<T> = std::result::Result<T, Error>;

/// Convieniently constructs a Result from a message. E.g.:
///	err("launch thrusters offline")
pub fn err<S: Into<String>, T>(x: S) -> Result<T> {
	Err(error(x.into()))
}
