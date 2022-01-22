use super::internal::*;
use std::{mem::take, str::FromStr};

pub fn parse<R: Read>(r: R) -> Result<ObjSet> {
	Parser::new().parse(r)
}

#[derive(Default)]
struct Parser {
	curr_mtllib: Option<String>,
	curr_mtl: Option<String>,
	curr_o: String,
	curr_f: Vec<VIndices>,

	v: Vec<vec3>,
	vt: Vec<vec2>,
	vn: Vec<vec3>,

	objects: Vec<Object>,
}

impl Parser {
	fn new() -> Self {
		Self::default()
	}

	fn parse<R: Read>(mut self, r: R) -> Result<ObjSet> {
		let reader = BufReader::new(r);
		for (line_num, line) in reader.lines().enumerate() {
			let line = line?;
			self.parse_line(&line) //
				.map_err(|e| anyhow!("line {}: {}: {}", line_num, line, e))?;
		}

		self.flush_curr_object()?;

		Ok(ObjSet {
			mtllib: self.curr_mtllib,
			objects: self.objects,
		})
	}

	fn parse_line(&mut self, line: &str) -> Result<()> {
		use ParsedLine::*;
		match ParsedLine::from_str(line)? {
			V(v) => Ok(self.v.push(v)),
			Vn(vn) => Ok(self.vn.push(vn)),
			Vt(vt) => Ok(self.vt.push(vt)),
			O(name) => self.start_object(name),
			Mtllib(m) => Ok(self.curr_mtllib = Some(m)),
			Usemtl(m) => Ok(self.curr_mtl = Some(m)),
			F(i) => Ok(self.curr_f.push(i)),
			Comment(_) => Ok(()),
			Unknown(_) => Ok(()),
			S(_) => Ok(()),
		}
	}

	// flush the currently pending object and start a new one.
	fn start_object(&mut self, new_name: String) -> Result<()> {
		self.flush_curr_object()?;
		self.curr_o = new_name;
		Ok(())
	}

	// push the currently pending faces to a new object.
	fn flush_curr_object(&mut self) -> Result<()> {
		if self.curr_f.len() != 0 {
			let name = take(&mut self.curr_o);
			let mtl = self.curr_mtl.clone();
			let curr_f = take(&mut self.curr_f);
			let faces = curr_f.iter().map(|i| self.make_face(i)).collect::<Result<Vec<Face>>>()?;
			self.objects.push(Object { name, mtl, faces });
		};
		Ok(())
	}

	// turn face indices (e.g. `1//2 3//4 5//6`) into a Face.
	fn make_face(&self, indices: &[VIndex]) -> Result<Face> {
		indices.iter().map(|i| self.make_vertex(i)).collect::<Result<Face>>()
	}

	// turn vertex indices (e.g. `2//4`) into a Vertex.
	fn make_vertex(&self, index: &VIndex) -> Result<Vertex> {
		Ok(Vertex {
			position: index_1(&self.v, index.0 as usize)?,

			texture: match index.1 {
				None => vec2(0.0, 0.0),
				Some(i) => index_1(&self.vt, i as usize)?,
			},

			normal: match index.2 {
				None => vec3(0.0, 0.0, 0.0),
				Some(i) => index_1(&self.vn, i as usize)?,
			},
		})
	}
}

// 1-based indexing, return error when out of bounds.
fn index_1<T: Clone>(v: &Vec<T>, i: usize) -> Result<T> {
	Ok(v.get(i - 1).ok_or_else(|| anyhow!("invalid index"))?.clone())
}
