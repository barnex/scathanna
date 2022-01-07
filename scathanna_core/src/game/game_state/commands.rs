use super::internal::*;

// expect exactly one argument, return error otherwise.
pub fn one_arg<'a>(args: &'a [&str]) -> Result<&'a str> {
	match args {
		&[arg] => Ok(arg),
		_ => err("need 1 argument"),
	}
}
