//! Convert wavefront object files into game maps.
//use game::wavefrontobj;
use std::io::prelude::*;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
	/// Map size (voxels)
	#[structopt(short, long, default_value = "256")]
	pub size: u32,

	/// Fog distance.
	#[structopt(long, default_value = "100")]
	pub fog_dist: f32,

	/// Fog color.
	#[structopt(long, default_value = "ffffff")]
	pub fog_color: RGB,

	/// Wavefront Object file to open.
	pub obj_file: PathBuf,
}

fn main() -> Result<()> {
	let args = Args::from_args();

	let mat = wavefrontobj::parse_with_material(&args.obj_file)?;
	let mesh = wavefrontobj::parse_file(&args.obj_file)?;

	let vertices = mesh.vertex_positions();
	let mesh_bb = BoundingBox::from(vertices.iter());
	let mesh_hsize = f32::max(mesh_bb.size().x, mesh_bb.size().z);
	println!("{} vertices", mesh.len());

	let mut map = Map::new(uvec3(args.size, args.size, args.size));
	map.fog_dist = args.fog_dist;
	map.background_color = args.fog_color.into();
	let world_size = args.size as f32;

	let transform = |pos: vec3| ((pos / mesh_hsize) + vec3(0.5, 0.0, 0.5)) * world_size;

	let mut add_triangle = |v0: vec3, v1: vec3, v2: vec3, voxel: Voxel| {
		let o = v0;
		let a = v1 - v0;
		let b = v2 - v0;

		let res = 0.3;
		let n_u = (a.len() / res) as usize + 1;
		let n_v = (b.len() / res) as usize + 1;

		for u in 0..(n_u + 1) {
			for v in 0..(n_v + 1) {
				let u = (u as f32) / (n_u as f32);
				let v = (v as f32) / (n_v as f32);

				if u + v > 1.0 {
					continue;
				}

				let pos = o + u * a + v * b;
				let ipos = pos.map(|v| (v + 0.5) as i32);
				map.set(ipos, voxel);
			}
		}
	};

	for (mat, b) in &mat {
		println!("{}", mat);
		let voxel = parse_mat(mat);
		let vertices = b.vertex_positions();
		let mut i = 0;
		while i < vertices.len() {
			add_triangle(
				transform(vertices[i + 0]), //
				transform(vertices[i + 1]),
				transform(vertices[i + 2]),
				voxel,
			);

			i += 3;
		}
	}

	println!("filling...");
	fill(&mut map);

	let output_file = args.obj_file.with_extension("bincode.gz");
	map.save(output_file)?;

	let dump_file = args.obj_file.with_extension("txt");
	dump_txt(&map, &dump_file)?;

	Ok(())
}

fn dump_txt(map: &Map, fname: &Path) -> Result<()> {
	let mut w = BufWriter::new(File::create(fname)?);

	let size = map.size();
	for z in 0..size.z {
		for y in 0..size.y {
			for x in 0..size.x {
				match map.at(ivec3(x as i32, y as i32, z as i32)) {
					Voxel::EMPTY => (),
					voxel => write!(w, "{} {} {} {}\n", x, y, z, voxel.id())?,
				}
			}
		}
	}

	Ok(())
}

fn fill(map: &mut Map) {
	let start = map.size().as_ivec() - ivec3(1, 1, 1);

	let mut stack = vec![start];
	let mark: Voxel = Voxel::from(255);

	while let Some(pos) = stack.pop() {
		map.set(pos, mark);

		for &delta in &[
			ivec3(-1, 0, 0), //
			ivec3(1, 0, 0),
			ivec3(0, -1, 0),
			ivec3(0, 1, 0),
			ivec3(0, 0, -1),
			ivec3(0, 0, 1),
		] {
			let neighbor = pos + delta;
			if map.valid_index(neighbor) && map.at(neighbor) == Voxel::EMPTY {
				stack.push(neighbor);
			}
		}
	}

	let mut last = Voxel::EMPTY;
	let (nx, ny, nz) = map.size().as_ivec().into();
	for ix in 0..nx {
		for iz in 0..nz {
			for iy in 0..ny {
				let iy = ny - 1 - iy;
				let index = ivec3(ix, iy, iz);
				let v = map.at(index);
				if v == Voxel::EMPTY {
					map.set(index, last)
				}
				if v != Voxel::EMPTY && v != mark {
					last = v
				}
				if v == mark {
					map.set(index, Voxel::EMPTY)
				}
			}
		}
	}
}

fn parse_mat(mat: &str) -> Voxel {
	let mut i = PathBuf::from(mat).extension().unwrap_or_default().to_string_lossy().clone().parse::<i32>().unwrap_or_default();
	if i == 0 {
		i = 1;
	}
	Voxel::from(i as u8)
}

use std::fs::File;
use std::io::BufWriter;
use std::str::FromStr;

/// https://rust-lang-nursery.github.io/rust-cookbook/text/string_parsing.html
#[derive(Debug, PartialEq)]
struct RGB {
	r: u8,
	g: u8,
	b: u8,
}

impl FromStr for RGB {
	type Err = std::num::ParseIntError;

	// Parses a color hex code of the form 'rRgGbB..' into an
	// instance of 'RGB'
	fn from_str(hex_code: &str) -> std::result::Result<Self, Self::Err> {
		// u8::from_str_radix(src: &str, radix: u32) converts a string
		// slice in a given base to u8
		let r: u8 = u8::from_str_radix(&hex_code[0..2], 16)?;
		let g: u8 = u8::from_str_radix(&hex_code[2..4], 16)?;
		let b: u8 = u8::from_str_radix(&hex_code[4..6], 16)?;

		Ok(RGB { r, g, b })
	}
}

impl Into<vec3> for RGB {
	fn into(self) -> vec3 {
		vec3((self.r as f32) / 255.0, (self.g as f32) / 255.0, (self.b as f32) / 255.0)
	}
}
