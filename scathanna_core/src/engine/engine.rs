use super::internal::*;
use gl_safe::*;

const MIPMAP_LEVELS: u32 = 6;

pub struct Engine {
	viewport: (u32, u32),
	camera_pos: vec3,
	camera_dir: vec3,
	shaders: Shaders,

	//view_distance: f32,
	sun_direction: vec3,
	ambient: f32,

	tex_cache: RefCell<HashMap<String, Rc<Texture>>>,
	obj_cache: RefCell<HashMap<String, Rc<VertexArray>>>,

	font_char_vao: VertexArray,

	// performance counters
	texture_binds: Counter,
	draw_calls: Counter,
	vertices_drawn: Counter,
	last_perf: Cell<Instant>,
}

impl Engine {
	pub fn new() -> Self {
		gl_safe::glEnable(gl::DEPTH_TEST);
		gl_safe::glEnable(gl::CULL_FACE);

		Self {
			shaders: Shaders::new(),
			viewport: (0, 0),
			tex_cache: default(),
			obj_cache: default(),
			sun_direction: vec3(1.0, 2.0, 0.5).normalized(),
			ambient: 0.2,
			font_char_vao: Self::build_vao_(&Self::char_rect()),
			texture_binds: Counter::new(),
			draw_calls: Counter::new(),
			vertices_drawn: Counter::new(),
			camera_dir: default(),
			camera_pos: default(),
			last_perf: Cell::new(Instant::now()),
		}
	}

	// for access to counters
	pub fn shaders(&self) -> &Shaders {
		&self.shaders
	}
}

// Font rendering
impl Engine {
	pub const FONT_SIZE: f32 = 1.0 / 96.0;
	pub const FONT_HSIZE: f32 = 1.0 * Self::FONT_SIZE;
	pub const FONT_VSIZE: f32 = 2.0 * Self::FONT_SIZE;

	/// Draw text in the top-left corner.
	pub fn print_top_left(&self, color: vec3, text: &str) {
		let pos = vec2(-0.5, 0.5 * self.viewport_aspect());
		self.draw_text(pos, color, text)
	}

	/// Draw text in the top-right corner.
	pub fn print_top_right(&self, color: vec3, text: &str) {
		let w = Self::text_width(text) as f32 * Self::FONT_HSIZE;
		let pos = vec2(0.5 - w, 0.5 * self.viewport_aspect());
		self.draw_text(pos, color, text)
	}

	/// Draw text in the bottom-left corner.
	pub fn print_bottom_left(&self, color: vec3, text: &str) {
		let pos = vec2(-0.5, -0.5 * self.viewport_aspect() + Self::FONT_VSIZE);
		self.draw_text(pos, color, text)
	}

	/// Draw text in the bottom-right corner.
	pub fn print_bottom_right(&self, color: vec3, text: &str) {
		let w = Self::text_width(text) as f32 * Self::FONT_HSIZE;
		let h = Self::text_height(text) as f32 * Self::FONT_VSIZE;
		let pos = vec2(0.5 - w, -0.5 * self.viewport_aspect() + h);
		self.draw_text(pos, color, text)
	}

	/// Draw text in the center of the screen,
	/// a bit above the crosshair.
	pub fn print_center(&self, color: vec3, text: &str) {
		let w = Self::text_width(text) as f32 * Self::FONT_HSIZE / 2.0;
		let pos = vec2(-w, 0.125 * self.viewport_aspect() + Self::FONT_VSIZE);
		self.draw_text(pos, color, text)
	}

	/// Draw text at position (in range -0.5..0.5).
	fn draw_text(&self, pos: vec2, color: vec3, text: &str) {
		self.set_depth_test(false);

		self.set_cull_face(false);
		let font_tex = self.texture("font", WHITE);
		self.bind_texture(font_tex.as_ref(), 0 /*unit*/);

		let mut char_pos = pos - vec2(0.0, Self::FONT_VSIZE);
		for &byte in text.as_bytes() {
			if byte == b'\n' {
				char_pos.x = pos.x;
				char_pos.y -= Self::FONT_VSIZE;
				continue;
			}

			let tex_offset = self.char_tex_offset(byte);
			self.shaders.use_font(tex_offset, char_pos, color);

			self.draw_triangles(&self.font_char_vao);

			char_pos.x += Self::FONT_HSIZE;
		}

		self.set_depth_test(true);
		self.set_cull_face(true);
	}

	// viewport height/width ratio.
	fn viewport_aspect(&self) -> f32 {
		self.viewport.1 as f32 / self.viewport.0 as f32
	}

	// find a character position in the ascii font texture.
	fn char_tex_offset(&self, char: u8) -> vec2 {
		let x = (char & 0xf) as f32 / 16.0;
		let y = (char >> 4) as f32 / 8.0;
		vec2(x, y)
	}

	// columns needed to layout text.
	fn text_width(text: &str) -> usize {
		text.lines().map(|l| l.len()).max().unwrap_or_default()
	}

	// lines needed to layout text.
	fn text_height(text: &str) -> usize {
		text.lines().count()
	}

	// A rectangular VAO for blitting a single character.
	fn char_rect() -> MeshBuffer {
		let c = Self::FONT_SIZE;
		let z = 0.5;
		let positions = vec![
			c * vec3(0.0, 0.0, z), //
			c * vec3(1.0, 0.0, z),
			c * vec3(1.0, 2.0, z),
			c * vec3(1.0, 2.0, z),
			c * vec3(0.0, 2.0, z),
			c * vec3(0.0, 0.0, z),
		];
		let texcoords = vec![
			vec2(0.0, 2.0) / 16.0, //
			vec2(1.0, 2.0) / 16.0,
			vec2(1.0, 0.0) / 16.0,
			vec2(1.0, 0.0) / 16.0,
			vec2(0.0, 0.0) / 16.0,
			vec2(0.0, 2.0) / 16.0,
		];
		MeshBuffer::from_positions(positions).with_texcoords(texcoords)
	}

	/// Draw a crosshair in the center of the screen.
	pub fn draw_crosshair(&self) {
		let pos = vec2(-0.5 * Self::FONT_HSIZE, 0.5 * Self::FONT_VSIZE);
		self.draw_text(pos, YELLOW, "+");
	}
}

// Rendering
impl Engine {
	pub fn set_camera(&mut self, viewport: (u32, u32), camera: &Camera) {
		glViewport(0, 0, viewport.0 as i32, viewport.1 as i32);
		self.viewport = viewport;
		self.camera_pos = camera.position;
		self.camera_dir = camera.orientation.look_dir();
		self.shaders.set_camera(viewport, camera)
	}

	pub fn clear(&self, r: f32, g: f32, b: f32) {
		glClearColor(r, g, b, 1.0);
		glClear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
	}

	pub fn set_sun_direction(&mut self, sun_direction: vec3) {
		self.sun_direction = sun_direction
	}

	pub fn set_ambient(&mut self, ambient: f32) {
		self.ambient = ambient
	}

	/// Draw model without any transforms.
	pub fn draw_model(&mut self, model: &Model) {
		self.draw_model_with(model, &mat4::UNIT)
	}

	/// Draw `model` with translation by `position`.
	/// I.e. if `model` is centered at (0,0,0) then it will be drawn at `position`.
	pub fn draw_model_at(&self, model: &Model, position: vec3) {
		self.draw_model_with(model, &translation_matrix(position))
	}

	/// Draw `model` with an arbitrary transformation.
	pub fn draw_model_with(&self, model: &Model, transform: &mat4) {
		self.set_cull_face(!model.double_sided);
		self.use_shader(&model.material, transform);
		if model.lines {
			self.draw_lines(&model.vao)
		} else {
			self.draw_triangles(&model.vao)
		}
	}

	pub fn draw_boundingbox(&self, bounds: BoundingBox<f32>) {
		self.enable_line_offset();

		let BoundingBox { min, max } = bounds;
		let (x, y, z) = (max - min).into();
		let o = 0.0;
		let pos = [
			(o, o, o),
			(x, o, o),
			(x, o, o),
			(x, y, o),
			(x, y, o),
			(o, y, o),
			(o, y, o),
			(o, o, o),
			(o, o, o),
			(o, o, z),
			(x, o, o),
			(x, o, z),
			(x, y, o),
			(x, y, z),
			(o, y, o),
			(o, y, z),
			(o, o, z),
			(x, o, z),
			(x, o, z),
			(x, y, z),
			(x, y, z),
			(o, y, z),
			(o, y, z),
			(o, o, z),
		]
		.into_iter()
		.map(|v| vec3::from(v))
		.collect();

		let buf = MeshBuffer::from_positions(pos);
		let model = Model::new(self.build_vao(&buf), Material::UniformColor(WHITE)).with_lines();
		self.draw_model_at(&model, min);
	}

	pub fn use_shader(&self, material: &Material, transf: &mat4) {
		use Material::*;
		match material {
			UniformColor(col) => self.shaders.use_uniform_color(*col, transf),
			VertexColor => self.shaders.use_vertex_color(transf),
			FlatTexture(tex) => {
				self.bind_texture(&tex, 0 /*unit*/);
				self.shaders.use_flat_texture(transf)
			}
			MatteTexture(tex) => {
				self.bind_texture(&tex, 0 /*unit*/);
				self.shaders.use_matte_texture(self.sun_direction, self.ambient, transf)
			}
			Lightmap { texture, lightmap } => {
				self.bind_texture(&texture, 0 /*unit*/);
				self.bind_texture(&lightmap, 1 /*unit*/);
				self.shaders.use_lightmap(transf)
			}
		}
	}

	pub fn draw_triangles(&self, vao: &VertexArray) {
		self.draw_with_mode(vao, gl::TRIANGLES)
	}

	pub fn draw_lines(&self, vao: &VertexArray) {
		self.draw_with_mode(vao, gl::LINES)
	}

	/// Draws a line between points `from` and `to`.
	/// (Intended for debugging. Not very efficient.)
	pub fn draw_line(&self, color: vec3, width: f32, start: vec3, end: vec3) {
		self.use_shader(&Material::UniformColor(color), &mat4::UNIT);
		self.set_line_width(width);
		let vao = VertexArray::create().with_vec3_attrib(Shaders::ATTRIB_POSITIONS, &[start, end]);
		self.draw_with_mode(&vao, gl::LINES)
	}

	fn draw_with_mode(&self, vao: &VertexArray, mode: GLuint) {
		self.draw_with_mode_n(vao, mode, vao.len())
	}

	pub fn draw_with_mode_n(&self, vao: &VertexArray, mode: GLuint, n: usize) {
		self.draw_calls.inc();
		self.vertices_drawn.add(vao.len() as u64);
		vao.bind();
		glDrawArrays(mode, 0 /*first*/, n as i32);
	}

	pub fn enable_line_offset(&self) {
		glEnable(gl::POLYGON_OFFSET_LINE);
		glPolygonOffset(0.0, -8.0)
	}

	pub fn set_line_width(&self, width: f32) {
		glLineWidth(width)
	}

	pub fn set_cull_face(&self, enable: bool) {
		if enable {
			glEnable(gl::CULL_FACE)
		} else {
			glDisable(gl::CULL_FACE)
		}
	}

	pub fn set_depth_test(&self, enable: bool) {
		if enable {
			glEnable(gl::DEPTH_TEST)
		} else {
			glDisable(gl::DEPTH_TEST)
		}
	}
}

// Texture management
impl Engine {
	/// Get a texture by base name, fetching it if not yet in cache.
	///
	/// `base` does not need to have an extension: it defaults
	/// to `.jpg`, but can be explicitly present if needed
	/// (e.g. `texture.png` for semi-transparent textures)
	///
	/// Textures are loaded from directory `assets/textures/lo/{base}[.jpg]`.
	///
	/// Returns a uniform color as fallback if loading fails.
	///
	/// TODO: ideally, we should search both the `hi` and `lo` directories,
	/// and also automatically search for an existing file extension.
	/// However, this is not straightforward in a WASM context.
	///
	/// TODO: use interior mutability.
	pub fn texture(&self, base: &str, fallback_color: vec3) -> Rc<Texture> {
		if !self.tex_cache.borrow().contains_key(base) {
			self.tex_cache.borrow_mut().insert(
				base.to_owned(),
				Rc::new(load_mipmapped(MIPMAP_LEVELS, &Self::find_texture(base).unwrap_or(PathBuf::from(base)), fallback_color)),
			);
		}

		Rc::clone(self.tex_cache.borrow().get(base).unwrap())
	}

	fn find_texture(base: &str) -> Option<PathBuf> {
		// debug assertions makes loading textures terribly slow,
		// so us low-resolution in that case.
		#[cfg(debug_assertions)]
		const RES: &str = "lo/";
		#[cfg(not(debug_assertions))]
		const RES: &str = "hi/";

		for ext in ["png", "jpg", "jpeg"] {
			let file = abs_path(&Path::new(TEXTURES_PATH).join(RES).join(base).with_extension(ext));
			if file.exists() {
				return Some(file);
			}
		}
		None
	}

	pub fn lightmap_from_mem(&self, img: &Image<Color>) -> Rc<Texture> {
		const NO_MIPMAP_LEVELS: u32 = 1;
		let size = img.dimensions().into();
		Rc::new(
			Texture::new2d(gl::RGBA8, NO_MIPMAP_LEVELS, size) //
				.sub_image2d(0, 0, 0, size.x, size.y, gl::RGBA, gl::UNSIGNED_BYTE, &img.raw_rgba())
				.filter_linear(),
		)
	}

	fn bind_texture(&self, tex: &Texture, unit: u32) {
		// TODO: lazy bind
		self.texture_binds.inc();
		tex.bind_texture_unit(unit)
	}
}

// VAO management
impl Engine {
	/// Upload MeshBuffer to the GPU for rendering.
	/// Note: takes a self argument to ensure that the caller has access to the GL main thread.
	pub fn build_vao(&self, buf: &MeshBuffer) -> Rc<VertexArray> {
		Rc::new(Self::build_vao_(buf))
	}

	fn build_vao_(buf: &MeshBuffer) -> VertexArray {
		let mut vao = VertexArray::create();
		vao = vao.with_vec3_attrib(Shaders::ATTRIB_POSITIONS, buf.positions());
		if let Some(normals) = &buf.normals {
			vao = vao.with_vec3_attrib(Shaders::ATTRIB_NORMALS, normals);
		}
		if let Some(texcoords) = &buf.texcoords {
			vao = vao.with_vec2_attrib(Shaders::ATTRIB_TEXCOORDS, texcoords);
		}
		if let Some(lightcoords) = &buf.lightcoords {
			vao = vao.with_vec2_attrib(Shaders::ATTRIB_LIGHTCOORDS, lightcoords);
		}
		vao
	}

	/// Fetch a Wavefront OBJ file from `{assets_path}/obj/{base}.obj`.
	/// Return faces as a flattened vertex array.
	/// Cached, so okay to get the same object several times
	/// (because, e.g. several avatar models share the same gun, feet, etc,
	/// and it would be cumbersome to deduplicate those manually).
	pub fn wavefront_obj(&self, base: &str) -> Result<Rc<VertexArray>> {
		if !self.obj_cache.borrow().contains_key(base) {
			self.obj_cache.borrow_mut().insert(base.to_owned(), Rc::new(self.wavefront_obj_uncached(base)?));
		}
		Ok(Rc::clone(self.obj_cache.borrow().get(base).unwrap()))
	}

	fn wavefront_obj_uncached(&self, base: &str) -> Result<VertexArray> {
		let obj_file = abs_path(&Path::new(OBJ_PATH).join(base).with_extension("obj"));
		println!("loading {}", obj_file.to_string_lossy());
		let objs = wavefrontobj::parse(open(&obj_file)?)?;

		Ok(VertexArray::create() //
			.with_vec3_attrib(Shaders::ATTRIB_POSITIONS, &objs.vertex_positions())
			.with_vec2_attrib(Shaders::ATTRIB_TEXCOORDS, &Self::flip_texcoords_y(objs.texture_cordinates()))
			.with_vec3_attrib(Shaders::ATTRIB_NORMALS, &objs.vertex_normals()))
	}

	// Fix texture orientation to be blender-compatible.
	fn flip_texcoords_y(mut texcoords: Vec<vec2>) -> Vec<vec2> {
		texcoords.iter_mut().for_each(|t| t.y = 1.0 - t.y);
		texcoords
	}
}
// Performance stats

impl Engine {
	pub fn draw_perf_stats(&self) {
		let last_perf = self.last_perf.replace(Instant::now());
		self.print_top_right(YELLOW, &self.prev_frame_stats(last_perf.elapsed()))
	}

	fn prev_frame_stats(&self, since_last_frame: Duration) -> String {
		let fps = (1.0 / (since_last_frame.as_secs_f64())).round();

		format!(
			r"{} FPS ({} ms)
shader : {}
texture: {}
draw   : {}
vertex : {}k",
			fps,
			since_last_frame.as_millis(),
			self.shaders().shader_switches.take(),
			self.texture_binds.take(),
			self.draw_calls.take(),
			self.vertices_drawn.take() / 1000,
		)
	}
}
