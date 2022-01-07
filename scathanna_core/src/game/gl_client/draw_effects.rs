use rand::Rng;

use super::internal::*;

// A VertexArray containing a "particle explosion" consisting of `n` triangles
// with random orientations, and random velocities pointing away from the origin.
// To be rendered with `shaders::Particles`.
pub fn particle_explosion_vao(engine: &Engine, n: usize) -> VertexArray {
	let pos = |_i| vec3(0.0, 0.0, 0.0);
	let vel = |_i| 15.0;
	triangle_particles_vao(engine, n, pos, vel)
}

pub fn particle_beam_vao(engine: &Engine) -> VertexArray {
	let max_dist = 500;
	let n = PARTICLE_BEAM_DENSITY * max_dist;
	let pos = |i| {
		let dist = (i as f32) / (PARTICLE_BEAM_DENSITY as f32);
		vec3(0.0, dist, 0.0)
	};
	let vel = |i| 7.0 * (1.0 - (i as f32) / (max_dist as f32));
	triangle_particles_vao(engine, n, pos, vel)
}

fn triangle_particles_vao(_engine: &Engine, n: usize, pos: impl Fn(usize) -> vec3, vel: impl Fn(usize) -> f32) -> VertexArray {
	let mut positions = Vec::new();
	let mut tex_coords = Vec::new();
	let mut colors = Vec::new();
	let mut velocities = Vec::new();

	let mut rng = rand::thread_rng();

	let palette = [vec3(1.0, 0.5, 0.5), vec3(1.0, 1.0, 0.5), vec3(0.5, 1.0, 0.5), vec3(0.5, 0.5, 1.0)];

	for i in 0..n {
		// isotropic orientation
		let norm = sample_isotropic_direction(&mut rng);
		let basis = make_basis(norm);
		for &vert in &TRIANGLE_VERTICES {
			tex_coords.push(vert.xy());
			positions.push(basis * vert + pos(i));
		}

		let color = palette[rng.gen_range(0..palette.len())];
		colors.push(color);
		colors.push(color);
		colors.push(color);

		// isotropic velocity
		let v_dir = sample_isotropic_direction(&mut rng);
		let v = vel(i) * v_dir;
		velocities.push(v);
		velocities.push(v);
		velocities.push(v);
	}

	VertexArray::create() //.
		.with_vec3_attrib(Shaders::ATTRIB_POSITIONS, &positions)
		.with_vec2_attrib(Shaders::ATTRIB_TEXCOORDS, &tex_coords)
		.with_vec3_attrib(Shaders::ATTRIB_VERTEX_COLORS, &colors)
		.with_vec3_attrib(Shaders::ATTRIB_VERTEX_VELOCITY, &velocities)
}

// Vertices of an equilateral triangle centered at (0,0,0).
// "Prototype" for all particle triangles.
const TRIANGLE_VERTICES: [vec3; 3] = [
	vec3(-0.5, -SIN_60 / 2.0, 0.0), //.
	vec3(0.5, -SIN_60 / 2.0, 0.0),
	vec3(0.0, SIN_60 / 2.0, 0.0),
];

const SIN_60: f32 = 0.86602540378;

/// Number of particles per unit of particle beam length.
const PARTICLE_BEAM_DENSITY: usize = 2;

pub fn draw_particle_beam_effect(engine: &Engine, vao: &VertexArray, start: vec3, orientation: Orientation, len: f32, color_filter: vec3, ttl: f32) {
	let time = PARTICLES_TTL - ttl;
	let gravity = 20.0;

	let pitch_mat = pitch_matrix(-90.0 * DEG - orientation.pitch);
	let yaw_mat = yaw_matrix(-orientation.yaw);
	let location_mat = translation_matrix(start);
	let mat = location_mat * yaw_mat * pitch_mat;

	engine.set_cull_face(false);
	let alpha = 1.0;
	engine.shaders().use_particles(color_filter, alpha, gravity, time, &mat);

	// pick the number of triangles to match the desired beam length.
	// number of vertices = 3*number of triangles.
	let n = 3 * (len as usize + 1) * PARTICLE_BEAM_DENSITY;
	let n = clamp(n, 1, vao.len());
	engine.draw_with_mode_n(vao, gl::TRIANGLES, n);
}

pub fn draw_particle_explosion(engine: &Engine, vao: &VertexArray, pos: vec3, color: vec3, ttl: f32) {
	let time = PARTICLES_TTL - ttl;
	let gravity = 20.0;

	let location_mat = translation_matrix(pos);

	engine.set_cull_face(false);
	let alpha = 1.0;
	engine.shaders().use_particles(color, alpha, gravity, time, &location_mat);
	engine.draw_triangles(vao);
}
