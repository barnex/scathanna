use super::internal::*;

/// Heads-up display (score, log messages, ...).
#[derive(Default)]
pub struct HUD {
	topleft: String,

	message: String,
	message_ttl: f32,

	log_msg: Vec<String>,
	log_ttl: f32,
}

// Time-to-live for "you killed..." message shown above crosshair.
const MSG_TTL: f32 = 4.0;
const LOG_TTL: f32 = 4.0;

// Max log queue length.
const MAX_LOG_MSG: usize = 8;

impl HUD {
	pub fn update(&mut self, upd: HUDUpdate) {
		use HUDUpdate::*;
		match upd {
			Message(message) => self.show(message),
			Log(message) => self.log(message),
			Score(message) => self.topleft = message,
		}
	}

	pub fn show(&mut self, message: String) {
		self.message = message;
		self.message_ttl = MSG_TTL;
	}

	pub fn log(&mut self, message: String) {
		self.log_msg.push(message);
		if self.log_msg.len() > MAX_LOG_MSG {
			self.log_msg.remove(0);
		}
		self.log_ttl = LOG_TTL;
	}

	pub fn tick(&mut self, dt: f32) {
		self.message_ttl -= dt;
		self.log_ttl -= dt;

		if self.message_ttl < 0.0 {
			self.message.clear();
		}

		if self.log_ttl < 0.0 && self.log_msg.len() != 0 {
			self.log_msg.remove(0);
			self.log_ttl = LOG_TTL;
		}
	}

	pub fn draw(&self, engine: &Engine, player: &Player) {
		engine.print_bottom_left(GREY, &self.log_msg.join("\n"));

		engine.print_top_left(WHITE, &self.topleft);

		match self.message.as_str() {
			"" => (),
			msg => engine.print_center(WHITE, msg),
		}

		if !player.spawned {
			engine.print_top_center(WHITE, "Click to respawn")
		}

		engine.draw_perf_stats();
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HUDUpdate {
	Message(String),
	Log(String),
	Score(String),
}
