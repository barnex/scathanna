use rand::prelude::*;
use std::ops::Range;
use std::time::Instant;

use super::internal::*;

// Size of lightmap textures.
// TODO: avoid out-of-bounds if somehow too small.
pub const LIGHTMAP_SIZE: u32 = 512;
static SUN_DIR: vec3 = vec3(0.304855380424846, 0.609710760849692, 0.731652913019631);

pub fn bake_lightmap(lm: &mut Lightmap, voxels: &Voxels, mapped_faces: &HashMap<VoxelType, Vec<(Rectangle, uvec2)>>, quality: bool) {
	//let mut cache: Cache = default();
	let img = lm.image_mut();
	let start = Instant::now();
	for (&voxeltype, mapped_rects) in mapped_faces.iter() {
		for (rect, lm_idx) in mapped_rects {
			bake_face(voxels, img, *lm_idx, rect, voxeltype, quality);
		}
	}
	println!("baked in {:.1} ms", start.elapsed().as_secs_f32() * 1000.0);
	*img = add_borders(img)
}

pub fn bake_face(voxels: &Voxels, img: &mut Image<Color>, lm_idx: uvec2, rect: &Rectangle, voxeltype: VoxelType, quality: bool) {
	// NOTE: these are INCLUSIVE bounds.
	// a lightmap island is 1 texel larger than its corresponding
	// (e.g. a 2x4 face is embedded in a 3x5 island)
	//
	// The face is UV mapped to the island with a 0.5 texel offset
	// so that we always stay 0.5 texels away from the island border.
	// This allows linear interpolation without shadow bleeding.
	//
	// E.g.: this is how a 1x1 and 2x2 face are mapped to
	// 2x2 and 3x3 lightmap islands, respectively:
	//
	//  texel 0   1   2   3   4   5   6   7
	//      0 .   .   .   .   .   .   .   .
	//          +---+           +---+---+
	//      1 . | . | .   .   . | . | . | .
	//          +---+           +---+---+
	//      2 .   .   .   .   . | . | . | .
	//                          +---+---+
	//      3 .   .   .   .   .   .   .   .
	//
	// In addition, the islands themselves are given yet an extra 1 texel margin. See fn `add_borders`.
	let min = lm_idx;
	let max = min + rect.size;

	for iy in min.y..=max.y {
		for ix in min.x..=max.x {
			let dy = (iy - min.y) as i32;
			let dx = (ix - min.x) as i32;

			let [tan_x, tan_y] = rect.direction.tangents();
			let world_pos = rect.position + dx * tan_x.ivec() + dy * tan_y.ivec();

			*img.at_mut((ix, iy)) = match voxeltype {
				VoxelType(2) => Color::WHITE,
				VoxelType(10) => Color::WHITE,
				_ => match quality {
					false => bake_point_fast(voxels, world_pos, rect.direction),
					true => bake_point_traced(voxels, world_pos, rect.direction),
				},
			}
		}
	}
}

pub fn bake_point_fast(voxels: &Voxels, pos: ivec3, normal: Direction) -> Color {
	let sun = (with_staggered_probe(direct_sunlight_point))(voxels, pos.to_f32(), normal);
	let amb = simple_occlusion(voxels, pos, normal) * fake_oriented_ambient(normal);
	let amb_frac = 0.3;
	let l = (1.0 - amb_frac) * sun + amb_frac * amb;
	Color::new(l, l, l)
}

pub fn bake_point_traced(voxels: &Voxels, pos: ivec3, normal: Direction) -> Color {
	let mut rng = rand::thread_rng();
	let pos = pos.to_f32();

	let direct = integrate(&mut rng, voxels, pos, normal, 32..256, 0.002, sample_direct_light);
	let ambient = integrate(&mut rng, voxels, pos, normal, 300..9000, 0.002, sample_ambient_light);

	direct + ambient
}

fn integrate<F>(rng: &mut ThreadRng, voxels: &Voxels, pos: vec3, normal: Direction, n: Range<u32>, max_err: f64, f: F) -> Color
where
	F: Fn(&mut ThreadRng, u32, (f32, f32), &Voxels, vec3, Direction) -> Color,
{
	let mut halton_i = 0;
	let halton_shift = (rng.gen(), rng.gen());
	raytrace::integrate(n, max_err, &mut || {
		halton_i += 1;
		f(rng, halton_i, halton_shift, voxels, pos, normal)
	})
}

fn sample_ambient_light(rng: &mut ThreadRng, i: u32, (sh1, sh2): (f32, f32), voxels: &Voxels, pos: vec3, normal: Direction) -> Color {
	let u = rng.gen::<f32>() - 0.5;
	let v = rng.gen::<f32>() - 0.5;
	let normal_vec = normal.ivec().to_f32();
	let [du, dv] = normal.tangents().map(Direction::ivec).map(ivec3::to_f32);
	let pos = pos + u * du + v * dv;
	let (r, s) = halton23_scrambled(i, (sh1, sh2));
	let dir = cosine_sphere((r, s), normal_vec);
	let ray = DRay::new(pos.into(), dir.into()).offset(0.01);

	match voxels.intersect(&ray) {
		None => Color::new(0.85, 0.85, 1.0),                   // sky
		Some((VoxelType(2), _)) => Color::new(1.0, 0.2, 0.2),  // lava
		Some((VoxelType(10), _)) => Color::new(1.0, 1.0, 0.7), // light
		Some((_, t)) => {
			// world.
			let secundary_sunlight = direct_sunlight_point(voxels, ray.at(t as f64 - 0.01).map(|v| v as f32), normal);
			let spatial_occlusion = f32::min(1.0, square(t as f32 / 32.0));
			((0.5 * secundary_sunlight) + (0.03 * spatial_occlusion)) * Color::WHITE
		}
	}
}

fn square(x: f32) -> f32 {
	x * x
}

fn sample_direct_light(rng: &mut ThreadRng, i: u32, (sh1, sh2): (f32, f32), voxels: &Voxels, pos: vec3, normal: Direction) -> Color {
	let u = rng.gen::<f32>() - 0.5;
	let v = rng.gen::<f32>() - 0.5;
	let normal_vec = normal.ivec().to_f32();
	let [du, dv] = normal.tangents().map(Direction::ivec).map(ivec3::to_f32);
	let pos = pos + u * du + v * dv;

	// direct
	let (r, s) = halton23_scrambled(i, (sh1, sh2));
	let dir = (SUN_DIR + 0.04 * cosine_sphere((r, s), normal_vec)).normalized();
	let ray = DRay::new(pos.into(), dir.into()).offset(0.01);
	if !voxels.intersects(&ray) {
		f32::max(0.0, normal_vec.dot(SUN_DIR)) * Color::new(1.0, 0.98, 0.9)
	} else {
		Color::BLACK
	}
}

// Direct sunlight from a single point probe.
pub fn direct_sunlight_point(voxels: &Voxels, pos: vec3, normal: Direction) -> f32 {
	let ambient = 0.0;
	let cos_theta = normal.vec().dot(SUN_DIR);
	if cos_theta > 0.0 {
		let ray = DRay::new(pos.into(), SUN_DIR.into()).offset(0.01);
		if voxels.intersects(&ray) {
			ambient
		} else {
			f32::max(cos_theta, ambient)
		}
	} else {
		ambient
	}
}

fn fake_oriented_ambient(normal: Direction) -> f32 {
	0.5 * (1.0 + normal.vec().dot(SUN_DIR))
}

fn simple_occlusion(voxels: &Voxels, pos: ivec3, normal: Direction) -> f32 {
	// sum occlusion over 4 probes
	let tan = normal.tangents().map(Direction::ivec);
	let n = match normal.side_index() {
		1 => ivec3::ZERO,
		0 => normal.ivec(),
		_ => unreachable!(),
	};
	let probes = [
		n + pos, //
		n + pos - tan[0],
		n + pos - tan[1],
		n + pos - tan[0] - tan[1],
	];

	let mut ambient = -0.2;
	for probe in probes {
		if voxels.at(probe) == VoxelType::EMPTY {
			ambient += 0.25;
		}
	}

	ambient
}

// Populate the 1-texel border around lightmap islands, with color equal to the average over the nearest neighbors.
// In principle this is redundant
// as those texels are not supposed to be ever accessed. However, triangles seen
// at a large distance and under grazing incidence cause these texels to be accessed nevertheless
// (presumably due to round-off errors). This causes subtle but noticeable artifacts at island edges.
pub fn add_borders(src: &Image<Color>) -> Image<Color> {
	let mut dst = src.clone();

	for iy in 0..(src.width() as i32) {
		for ix in 0..(src.height() as i32) {
			if src.at((ix as u32, iy as u32)) == Color::BLACK {
				let mut acc = Color::BLACK;
				let mut n = 0;
				for dx in -1..=1 {
					for dy in -1..=1 {
						let px = ix + dx;
						let py = iy + dy;
						if px >= 0 && px < src.width() as i32 && py >= 0 && py < src.height() as i32 {
							let src = src.at((px as u32, py as u32));
							if src != Color::BLACK {
								n += 1;
								acc += src;
							}
						}
					}
				}
				let avg = acc / (n as f32);
				dst.set((ix as u32, iy as u32), avg);
			}
		}
	}
	dst
}

fn with_staggered_probe<F>(f: F) -> impl Fn(&Voxels, vec3, Direction) -> f32
where
	F: Fn(&Voxels, vec3, Direction) -> f32,
{
	move |voxels, pos, normal| staggered_probe(&f, voxels, pos, normal)
}

#[inline]
pub fn staggered_probe<F>(f: F, voxels: &Voxels, pos: vec3, normal: Direction) -> f32
where
	F: Fn(&Voxels, vec3, Direction) -> f32,
{
	let mut acc = 0.0;

	let [u, v] = normal.tangents().map(Direction::vec);
	for (du, dv) in [(-0.5, -0.5), (-0.5, 0.5), (0.5, -0.5), (0.5, 0.5)] {
		let probe = pos + u * du + v * dv;
		acc += f(voxels, probe, normal);
	}

	0.25 * acc
}

/*
fn with_anti_alias<F>(f: F) -> impl Fn(&Voxels, vec3, Direction) -> f32
where
	F: Fn(vec3, &Voxels, vec3, Direction) -> f32,
{
	move |voxels, pos, normal| anti_alias(&f, voxels, pos, normal)
}

#[inline]
fn anti_alias<F>(f: F, voxels: &Voxels, pos: vec3, normal: Direction) -> f32
where
	F: Fn(vec3, &Voxels, vec3, Direction) -> f32,
{
	let mut acc = 0.0;

	let w = [
		[1.0 / 16.0, 1.0 / 08.0, 1.0 / 16.0], //
		[1.0 / 08.0, 1.0 / 04.0, 1.0 / 08.0],
		[1.0 / 16.0, 1.0 / 08.0, 1.0 / 16.0], //
	];

	let [u, v] = normal.tangents().map(Direction::vec);
	for (i, &du) in [-1.0, 0.0, 1.0].iter().enumerate() {
		for (j, &dv) in [-1.0, 0.0, 1.0].iter().enumerate() {
			let probe = pos + u * du + v * dv;
			acc += w[j][i] * f(SUN_DIR, voxels, probe, normal);
		}
	}

	acc
}
*/