use std::fmt;

/// Alias for "any type of error".
pub type Error = Box<dyn std::error::Error>;

/// Convieniently constructs an Error from a message. E.g.:
///	  error("launch thrusters offline")
pub fn error<S: Into<String>>(msg: S) -> Error {
	Box::new(ErrorMessage(msg.into()))
}

#[derive(Debug)]
pub struct ErrorMessage(String);

impl fmt::Display for ErrorMessage {
	fn fmt(&self, w: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(w, "{}", &self.0)
	}
}

impl std::error::Error for ErrorMessage {}
