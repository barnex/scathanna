use std::fmt;

#[derive(Debug)]
pub struct ErrorMessage(String);

impl fmt::Display for ErrorMessage {
	fn fmt(&self, w: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(w, "{}", &self.0)
	}
}

impl std::error::Error for ErrorMessage {}
