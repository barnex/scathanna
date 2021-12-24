use super::internal::*;

/// Heads-up display (score, log messages, ...).
#[derive(Default)]
pub struct HUD {
	message: String,
	message_ttl: f32,
	log_msg: Vec<String>,
	log_ttl: f32,
}

// Time-to-live for "you killed..." message shown above crosshair.
const MSG_TTL: f32 = 4.0;
const LOG_TTL: f32 = 2.0;

// Max log queue length.
const MAX_LOG_MSG: usize = 8;

// Print FPS/Draw calls/...
const DBG_STATS: bool = true;

impl HUD {
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
}

pub fn draw_hud(engine: &Engine, player: &Player, hud: &HUD) {
	engine.print_top_left(WHITE, &hud.log_msg.join("\n"));

	match hud.message.as_str() {
		"" => (),
		msg => engine.print_center(WHITE, msg),
	}

	if !player.spawned {
		engine.print_center(WHITE, "\nClick to respawn")
	}

	if DBG_STATS {
		engine.draw_perf_stats();
	}
}
