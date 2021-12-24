//! Utilities for handling Errors and Results with error type `Box<dyn Error>`.
mod error;
mod result;

pub use error::*;
pub use result::*;
