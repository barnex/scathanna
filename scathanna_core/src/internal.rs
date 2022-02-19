pub use anyhow::anyhow;
pub use anyhow::Error;
pub use anyhow::Result;
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

pub use std::borrow::Borrow;
pub use std::borrow::BorrowMut;
pub use std::borrow::Cow;
pub use std::cell::Cell;
pub use std::cell::RefCell;
pub use std::fmt;
pub use std::fs;
pub use std::fs::File;
pub use std::io::prelude::*;
pub use std::io::BufRead;
pub use std::io::BufReader;
pub use std::io::BufWriter;
pub use std::mem;
pub use std::path::Path;
pub use std::path::PathBuf;
pub use std::str::FromStr;
pub use std::sync::mpsc::Receiver;
pub use std::sync::mpsc::Sender;
pub use std::sync::Arc;
pub use std::time::Duration;
pub use std::time::Instant;

pub const X: usize = 0;
pub const Y: usize = 1;
pub const Z: usize = 2;

pub type CowStr = Cow<'static, str>;

/// Shorthand for `Default::default()`.
#[inline]
pub fn default<T: Default>() -> T {
	T::default()
}
