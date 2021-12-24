use super::internal::*;

/// Stores which `Key`s (including mouse buttons) are currently pressed,
/// or have been briefly pressed since the last call to clear().
///
/// Stores the current mouse position expressed as global (yaw, pitch) angles.
///
/// Used for:
///  * de-bouncing: very brief key presses in between game ticks are not lost.
///  * removing periodic key repeats, sent by the OS when a key is held.
///  * providing absolute mouse orientation (so that we don't have to deal with every individual delta).
///  
#[derive(Default)]
pub struct InputState {
	pressed: [bool; NUM_KEYS],
	released: [bool; NUM_KEYS],
	down: [bool; NUM_KEYS],
	mouse_yaw: f32,
	mouse_pitch: f32,
}

impl InputState {
	pub fn new() -> Self {
		Self::default()
	}

	/// Record that `key` was pressed or released.
	pub fn record_key(&mut self, key: Key, pressed: bool) {
		if pressed {
			self.pressed[key as usize] = true;
			self.down[key as usize] = true;
		} else {
			self.released[key as usize] = true;
		}
	}

	/// Record that the mouse was moved by `(delta_x, delta_y)`.
	pub fn record_mouse(&mut self, delta: (f64, f64)) {
		let (dx, dy) = (delta.0 as f32, delta.1 as f32);
		self.mouse_yaw = wrap_angle(self.mouse_yaw - dx);
		self.mouse_pitch = clamp(self.mouse_pitch + dy, -PI / 2.0, PI / 2.0);
	}

	/// Must be called at the end of each game tick
	/// to clear pressed/released states.
	pub fn clear(&mut self) {
		for (i, &r) in self.released.iter().enumerate() {
			if r {
				self.down[i] = false;
			}
		}
		self.pressed = [false; NUM_KEYS];
		self.released = [false; NUM_KEYS];
	}

	/// True if `key` was down at least some time during the last tick.
	/// I.e., very short key presses are debounced and still recorded as having been down.
	pub fn is_down(&self, key: Key) -> bool {
		self.down[key as usize]
	}

	/// True if `key` transitioned (at least once) from up to down during the last tick.
	pub fn is_pressed(&self, key: Key) -> bool {
		self.pressed[key as usize]
	}

	/// True if `key` transitioned (at least once) from down to up during the last tick.
	pub fn is_released(&self, key: Key) -> bool {
		self.released[key as usize]
	}

	/// Absolute viewing angle wrt. the -Z axis (positive = CCW),
	/// accumulated by all mouse movements ever (i.e.: not reset on clear()).
	pub fn mouse_yaw(&self) -> f32 {
		self.mouse_yaw
	}

	/// Absolute viewing angle wrt. the -Z axis,
	/// accumulated by all mouse movements ever.
	/// (is not reset on clear()).
	pub fn mouse_pitch(&self) -> f32 {
		self.mouse_pitch
	}
}
