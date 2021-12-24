use super::internal::*;
use smallvec;
use std::str::FromStr;

/// A single line of a wavefront file, in parsed from.
#[derive(Debug, PartialEq)]
pub enum ParsedLine {
	Comment(String),
	F(VIndices),
	Mtllib(String),
	O(String),
	S(bool),
	Usemtl(String),
	V(vec3),
	Vt(vec2),
	Vn(vec3),
	Unknown(String),
}

pub type VIndices = SmallVec<VIndex>;
type SmallVec<T> = smallvec::SmallVec<[T; 4]>;

/// Parsed vertex indices: (position, texture coordinate, normal).
/// https://en.wikipedia.org/wiki/Wavefront_.obj_file#Vertex_indices.
#[derive(Debug, PartialEq)]
pub struct VIndex(pub i32, pub Option<i32>, pub Option<i32>);

impl FromStr for ParsedLine {
	type Err = any_result::Error;

	/// Parse a single line from a wavefront obj file.
	/// https://en.wikipedia.org/wiki/Wavefront_.obj_file.
	fn from_str(line: &str) -> Result<Self> {
		let (head, tail) = split_once(line);

		use ParsedLine::*;
		Ok(match head {
			"v" => V(vector3(tail)?),
			"vn" => Vn(vector3(tail)?),
			"vt" => Vt(vector2(tail)?),
			"f" => F(face(tail)?),
			"o" => O(tail.into()),
			"s" => S(smooth(tail)?),
			"mtllib" => Mtllib(tail.into()),
			"usemtl" => Usemtl(tail.into()),
			"#" => Comment(tail.into()),
			_ => Unknown(line.into()),
		})
	}
}

impl FromStr for VIndex {
	type Err = any_result::Error;

	/// Parse vertex indices like `1/2/3`, `3//4`, `5`.
	/// https://en.wikipedia.org/wiki/Wavefront_.obj_file#Vertex_indices.
	fn from_str(value: &str) -> Result<Self> {
		let split = value.split("/").collect::<SmallVec<&str>>();
		match split.len() {
			1 => Ok(VIndex(split[0].parse()?, None, None)),
			2 => Ok(VIndex(split[0].parse()?, Some(split[1].parse()?), None)),
			3 => Ok(VIndex(
				split[0].parse()?,
				match split[1] {
					"" => None,
					v => Some(v.parse()?),
				},
				Some(split[2].parse()?),
			)),
			_ => syntax_error(),
		}
	}
}

// Parse face indices like `1//2 3//4 5//6`.
fn face(text: &str) -> Result<VIndices> {
	let vertices = text.split_whitespace().map(VIndex::from_str).collect::<Result<VIndices>>()?;
	if vertices.len() < 3 {
		syntax_error()
	} else {
		Ok(vertices)
	}
}

// Parse 2 space-separated numbers like `0.5 0.6`.
fn vector2(text: &str) -> Result<vec2> {
	let args = split(text, 2)?;
	Ok(vec2(
		args[0].parse()?, //
		args[1].parse()?,
	))
}

// Parse 3 space-separated numbers like `1.0 2.0 3.0`.
fn vector3(text: &str) -> Result<vec3> {
	let args = split(text, 3)?;
	Ok(vec3(
		args[0].parse()?, //
		args[1].parse()?,
		args[2].parse()?,
	))
}

// Split on whitespace, expecting `n` words
// (error out otherwise).
fn split(text: &str, n: usize) -> Result<SmallVec<&str>> {
	let split: SmallVec<&str> = text.split_ascii_whitespace().collect();
	if split.len() == n {
		Ok(split)
	} else {
		syntax_error()
	}
}

// Split on first whitespace and trim results.
// Absent values are represented as `""`.
fn split_once(text: &str) -> (&str, &str) {
	match text.trim().split_once(" ") {
		Some((a, b)) => (a.trim(), b.trim()),
		None => match text {
			"" => ("", ""),
			c => (c, ""),
		},
	}
}

// Parse the argument of an `s` statement.
fn smooth(arg: &str) -> Result<bool> {
	match arg {
		"off" | "0" => Ok(false),
		"on" | "1" => Ok(true),
		_ => syntax_error(),
	}
}

fn syntax_error<T>() -> Result<T> {
	any_result::err("syntax error")
}
