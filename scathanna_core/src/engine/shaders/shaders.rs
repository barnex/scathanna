use super::internal::*;
use gl::types::*;

/// Type-safe access to a collection of shaders.
///
/// This struct bridges the "impedance mismatch" between the rather unsafe OpenGL API
/// and type safe Rust:
///
///   * Shaders can only be used if their uniforms are set
///   * set_camera sets the projection matrix for all shaders,
///     so that none can be forgotten
///
/// In addition, expensive shader switching is avoided if possible.
///
/// All shaders in this crates adhere to the convention that
///
///  vertex positions are attribute 0
///  vertex normals are attribute 1
///  vertex texture coordinates are attribute 2
///  vertex lightmap coordinates are attribute 3
///
/// All but the positions are optional.
pub struct Shaders {
	// Currently active shader, used to avoid unnecessary switches.
	curr_shader: Cell<GLuint>,
	cam_pos: Cell<vec3>,

	uniform_color: UniformColor,
	vertex_color: SimpleShader,
	flat_texture: SimpleShader,
	matte_texture: MatteTexture,
	glossy: Glossy,
	lightmap: Lightmap,
	particles: Particles,
	font: Font,

	projection_matrix: Cell<mat4>,
	isometric_matrix: Cell<mat4>,

	pub shader_switches: Counter,
}

impl Shaders {
	// These uniform locations must correspond to the locations used by all the *.vert files.
	// E.g. all vertex shaders use uniform location 0 for vertex position (ATTRIB_POSITIONS)..
	pub const ATTRIB_POSITIONS: u32 = 0;
	pub const ATTRIB_NORMALS: u32 = 1;
	pub const ATTRIB_TEXCOORDS: u32 = 2;
	pub const ATTRIB_LIGHTCOORDS: u32 = 3;
	pub const ATTRIB_VERTEX_COLORS: u32 = 4;
	pub const ATTRIB_VERTEX_VELOCITY: u32 = 5;

	pub fn new() -> Self {
		Self {
			curr_shader: Cell::new(0),
			uniform_color: UniformColor::new(),
			vertex_color: SimpleShader::new(include_str!("vertex_color.vert"), include_str!("vertex_color.frag")),
			flat_texture: SimpleShader::new(include_str!("flat_texture.vert"), include_str!("flat_texture.frag")),
			matte_texture: MatteTexture::new(),
			glossy: Glossy::new(),
			lightmap: Lightmap::new(),
			particles: Particles::new(),
			font: Font::new(),
			projection_matrix: Cell::new(mat4::UNIT),
			isometric_matrix: Cell::new(mat4::UNIT),
			cam_pos: Cell::new(vec3::ZERO),

			shader_switches: Counter::new(),
		}
	}

	/// Set the viewport `(width, height)`, camera position and orientation for all shaders.
	///
	/// This is done for all shaders at once because:
	///   (1) usually nearly all shaders are used for each frame anyway
	///   (2) set_camera is called only once per frame, so this is not expensive
	///   (3) it would be too easy to overlook some shader otherwise
	///
	pub fn set_camera(&self, (width, height): (u32, u32), camera: &Camera) {
		self.projection_matrix.set(camera.matrix((width, height)));
		self.isometric_matrix.set(isometric_matrix((width, height)));
	}

	/// Use `uniform_color.{vert, frag}` with a uniform fragment color.
	pub fn use_uniform_color(&self, color: vec3, transf: &mat4) {
		let prog = &self.uniform_color.base.prog;
		self.lazy_switch(prog);
		prog.uniform3f(self.uniform_color.uniform_color, color.x, color.y, color.z);
		prog.uniform_matrix4f(self.uniform_color.base.model, false, transf.as_array());
		prog.uniform_matrix4f(self.uniform_color.base.proj, false, self.projection_matrix.get().as_array());
	}

	/// Use `vertex_color.{vert, frag}` with per-vertex colors.
	pub fn use_vertex_color(&self, transf: &mat4) {
		let prog = &self.vertex_color.prog;
		self.lazy_switch(prog);
		prog.uniform_matrix4f(self.vertex_color.model, false, transf.as_array());
		prog.uniform_matrix4f(self.vertex_color.proj, false, self.projection_matrix.get().as_array());
	}

	/// Use `flat_texture.{vert, frag}`.
	pub fn use_flat_texture(&self, transf: &mat4) {
		let prog = &self.flat_texture.prog;
		self.lazy_switch(prog);
		prog.uniform_matrix4f(self.flat_texture.model, false, transf.as_array());
		prog.uniform_matrix4f(self.flat_texture.proj, false, self.projection_matrix.get().as_array());
	}

	/// Use `matte_texture.{vert, frag}`.
	pub fn use_matte_texture(&self, sun_dir: vec3, ambient: f32, transf: &mat4) {
		let prog = &self.matte_texture.base.prog;
		self.lazy_switch(prog);
		prog.uniform3f(self.matte_texture.sun_dir, sun_dir.x, sun_dir.y, sun_dir.z);
		prog.uniform1f(self.matte_texture.ambient, ambient);
		prog.uniform_matrix4f(self.matte_texture.base.model, false, transf.as_array());
		prog.uniform_matrix4f(self.matte_texture.base.proj, false, self.projection_matrix.get().as_array());
	}

	/// Use `glossy.{vert, frag}`.
	pub fn use_glossy(&self, sun_dir: vec3, sun_intens: f32, ambient: f32, transf: &mat4) {
		let shader = &self.glossy;
		let prog = &shader.base.prog;
		self.lazy_switch(prog);
		let cam_pos = self.cam_pos.get();
		prog.uniform3f(shader.sun_dir, sun_dir.x, sun_dir.y, sun_dir.z);
		prog.uniform1f(shader.sun_intens, sun_intens);
		prog.uniform1f(shader.ambient, ambient);
		prog.uniform3f(shader.cam_pos, cam_pos.x, cam_pos.y, cam_pos.z);
		prog.uniform_matrix4f(shader.base.model, false, transf.as_array());
		prog.uniform_matrix4f(shader.base.proj, false, self.projection_matrix.get().as_array());
	}

	/// Use `lightmap.{vert, frag}`.
	pub fn use_lightmap(&self, transf: &mat4) {
		let prog = &self.lightmap.base.prog;
		self.lazy_switch(prog);
		prog.uniform1i(self.lightmap.texture_unit, 0);
		prog.uniform1i(self.lightmap.lightmap_unit, 1);
		prog.uniform_matrix4f(self.lightmap.base.model, false, transf.as_array());
		prog.uniform_matrix4f(self.lightmap.base.proj, false, self.projection_matrix.get().as_array());
	}

	/// Use `particles.{vert, frag}`.
	pub fn use_particles(&self, color: vec3, alpha: f32, gravity: f32, time: f32, transf: &mat4) {
		let prog = &self.particles.base.prog;
		self.lazy_switch(prog);
		prog.uniform1f(self.particles.gravity, gravity);
		prog.uniform1f(self.particles.time, time);
		prog.uniform4f(self.particles.color, color.x, color.y, color.z, alpha);
		prog.uniform_matrix4f(self.particles.base.model, false, transf.as_array());
		prog.uniform_matrix4f(self.particles.base.model, false, transf.as_array());
		prog.uniform_matrix4f(self.particles.base.proj, false, self.projection_matrix.get().as_array());
	}

	/// Use `font.{vert, frag}`.
	pub fn use_font(&self, tex_offset: vec2, pos_offset: vec2, color: vec3) {
		let prog = &self.font.prog;
		self.lazy_switch(prog);
		prog.uniform2f(self.font.tex_offset, tex_offset.x, tex_offset.y);
		prog.uniform2f(self.font.pos_offset, pos_offset.x, pos_offset.y);
		prog.uniform3f(self.font.color, color.x, color.y, color.z);
		prog.uniform_matrix4f(self.font.proj, false, self.isometric_matrix.get().as_array());
	}

	/// Switch to shader if not yet in use
	/// (limits expensive shader switching).
	fn lazy_switch(self: &Self, program: &Program) {
		let id = program.handle();
		if id != self.curr_shader.get() {
			self.shader_switches.inc();
			self.curr_shader.set(id);
			program.use_program();
		}
	}
}

/// A shader that only has a model transform + projection matrix,
/// no other uniforms.  Can also serve as the base for shaders with more uniforms.
struct SimpleShader {
	prog: Program,
	model: UniformLocation,
	proj: UniformLocation,
}

impl SimpleShader {
	/// Vertex source must have uniforms called "model" and "proj".
	pub fn new(vertex_src: &str, fragment_src: &str) -> Self {
		let prog = compile_program(vertex_src, fragment_src);
		prog.use_program();

		Self {
			model: prog.uniform_location("model"),
			proj: prog.uniform_location("proj"),
			prog,
		}
	}
}

/// Wraps particles.{vert,frag}.
struct Particles {
	base: SimpleShader,
	gravity: UniformLocation,
	time: UniformLocation,
	color: UniformLocation,
}

impl Particles {
	fn new() -> Self {
		let base = SimpleShader::new(include_str!("particles.vert"), include_str!("particles.frag"));
		Self {
			gravity: base.prog.uniform_location("gravity"),
			time: base.prog.uniform_location("time"),
			color: base.prog.uniform_location("color"),
			base,
		}
	}
}

/// Wraps uniform_color.{vert,frag}.
struct UniformColor {
	base: SimpleShader,
	uniform_color: UniformLocation,
}

impl UniformColor {
	fn new() -> Self {
		let base = SimpleShader::new(include_str!("uniform_color.vert"), include_str!("uniform_color.frag"));
		Self {
			uniform_color: base.prog.uniform_location("color"),
			base,
		}
	}
}

/// Wraps matte_texture.{vert,frag}.
struct MatteTexture {
	base: SimpleShader,
	sun_dir: UniformLocation,
	ambient: UniformLocation,
}

impl MatteTexture {
	fn new() -> Self {
		let base = SimpleShader::new(include_str!("matte_texture.vert"), include_str!("matte_texture.frag"));
		Self {
			sun_dir: base.prog.uniform_location("sun_dir"),
			ambient: base.prog.uniform_location("ambient"),
			base,
		}
	}
}

/// Wraps matte_texture.{vert,frag}.
struct Glossy {
	base: SimpleShader,
	sun_dir: UniformLocation,
	sun_intens: UniformLocation,
	ambient: UniformLocation,
	cam_pos: UniformLocation,
}

impl Glossy {
	fn new() -> Self {
		let base = SimpleShader::new(include_str!("glossy.vert"), include_str!("glossy.frag"));
		Self {
			sun_dir: base.prog.uniform_location("sun_dir"),
			sun_intens: base.prog.uniform_location("sun_intens"),
			ambient: base.prog.uniform_location("ambient"),
			cam_pos: base.prog.uniform_location("cam_pos"),
			base,
		}
	}
}

/// Wraps matte_texture.{vert,frag}.
struct Lightmap {
	base: SimpleShader,
	texture_unit: UniformLocation,
	lightmap_unit: UniformLocation,
}

impl Lightmap {
	fn new() -> Self {
		let base = SimpleShader::new(include_str!("lightmap.vert"), include_str!("lightmap.frag"));
		Self {
			texture_unit: base.prog.uniform_location("texture_unit"),
			lightmap_unit: base.prog.uniform_location("lightmap_unit"),
			base,
		}
	}
}

/// Wraps font.{vert,frag}.
struct Font {
	prog: Program,
	proj: UniformLocation,
	tex_offset: UniformLocation,
	pos_offset: UniformLocation,
	color: UniformLocation,
}

impl Font {
	fn new() -> Self {
		let prog = compile_program(include_str!("font.vert"), include_str!("font.frag"));
		Self {
			tex_offset: prog.uniform_location("tex_offset"),
			pos_offset: prog.uniform_location("pos_offset"),
			color: prog.uniform_location("color"),
			proj: prog.uniform_location("proj"),
			prog,
		}
	}
}

fn compile_program(vertex_src: &str, fragment_src: &str) -> Program {
	let vertex_shader = gl_obj::Shader::new_vertex(vertex_src);
	let texture_shader = gl_obj::Shader::new_fragment(fragment_src);
	Program::new(&[&vertex_shader, &texture_shader])
}
