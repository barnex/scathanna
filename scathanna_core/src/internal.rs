pub use any_result::*;
pub use imageutil::*;
pub use matrix::*;
pub use raytrace::*;
pub use vector::*;

pub use flate2::read::GzDecoder;
pub use flate2::write::GzEncoder;
pub use serde::de::DeserializeOwned;
pub use serde::{Deserialize, Serialize};

pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;
pub type SmallVec<T> = smallvec::SmallVec<[T; 16]>;
pub use smallvec::smallvec;

pub use rand::{prelude::StdRng, Rng, SeedableRng};

pub use crate::apply::Apply;

pub use super::config::*;
pub use super::engine::*;
pub use super::event_handler::*;
pub use super::game::*;
pub use super::input::*;
pub use super::util::*;

pub use std::cell::Cell;
pub use std::cell::RefCell;
pub use std::fmt;
pub use std::fs;
pub use std::fs::File;
pub use std::io::prelude::*;
pub use std::io::BufRead;
pub use std::io::BufReader;
pub use std::io::BufWriter;
pub use std::path::Path;
pub use std::path::PathBuf;
pub use std::sync::Arc;
pub use std::time::Duration;
pub use std::time::Instant;

/// Shorthand for `Default::default()`.
#[inline]
pub fn default<T: Default>() -> T {
	T::default()
}
