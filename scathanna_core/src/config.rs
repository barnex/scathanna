use super::internal::*;

#[derive(Serialize, Deserialize)]
pub struct Config {
	/// Server address to connect to (e.g. "127.0.0.1:3344").
	/// (String rather than Address for JSON interop.)
	pub server: String,

	/// Player avatar: "frog", "chicken", "hamster", etc.
	pub avatar: String,

	/// Player nickname, e.g. "Bob".
	pub name: String,

	/// Preferred team: red|blu
	#[serde(default)]
	pub team: String,

	/// Resolution: width (pixels).
	pub window_width: u32,

	/// Resolution: height (pixels).
	pub window_height: u32,

	/// Run in borderless fullscreen mode
	pub fullscreen: bool,

	/// Disable window resizing.
	pub window_resizable: bool,

	/// Disable vsync.
	pub vsync: bool,

	/// Framerate cap.
	pub max_fps: f32,

	/// Enable alpha blending.
	pub alpha_blending: bool,

	/// Multi-sampling anti aliasing number of samples (must be a power of 2).
	pub msaa: u16,

	/// Mouse sensitivity.
	pub mouse_sensitivity: f64,

	/// Up, Left, Down, Right keys, e.g.: "wasd".
	pub movement_keys: String,
}

impl Config {
	pub fn parse(path: &Path) -> Result<Self> {
		serde_json::from_reader(open(path)?).map_err(|err| error(format!("Error in {}: {}", path.to_string_lossy(), err)))
	}
}
