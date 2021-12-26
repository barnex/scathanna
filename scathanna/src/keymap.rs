use any_result::*;
use glutin::event::{ModifiersState, VirtualKeyCode};
use scathanna_core::*;

pub struct KeyMap {
	up: VirtualKeyCode,
	left: VirtualKeyCode,
	down: VirtualKeyCode,
	right: VirtualKeyCode,
}

impl KeyMap {
	pub fn new(config: &Config) -> Result<Self> {
		let wasd = &config.movement_keys;
		if wasd.len() != 4 {
			return err(format!("configuration error: `movement_keys` requires 4 characters (e.g. `wasd`), got: `{}`", wasd));
		}

		let wasd = wasd.as_bytes();
		Ok(Self {
			up: keycode_for(wasd[0])?,
			left: keycode_for(wasd[1])?,
			down: keycode_for(wasd[2])?,
			right: keycode_for(wasd[3])?,
		})
	}

	/// Map physical keys (e.g. 'A') to game-specific keys (e.g.: "Jump").
	pub fn map(&self, code: VirtualKeyCode, modifiers: ModifiersState) -> Option<Key> {
		use glutin::event::VirtualKeyCode::*;
		if modifiers.is_empty() {
			if code == self.up {
				return Some(Key::Forward);
			}
			if code == self.left {
				return Some(Key::Left);
			}
			if code == self.down {
				return Some(Key::Backward);
			}
			if code == self.right {
				return Some(Key::Right);
			}
			return match code {
				S | Left => Some(Key::Left),
				F | Right => Some(Key::Right),
				E | Up => Some(Key::Forward),
				D | Down => Some(Key::Backward),
				Space => Some(Key::Jump),
				Z | B => Some(Key::Crouch),
				A => Some(Key::Sprint),
				Key1 => Some(Key::Key1),
				Key2 => Some(Key::Key2),
				Key3 => Some(Key::Key3),
				Key4 => Some(Key::Key4),
				Key5 => Some(Key::Key5),
				Key6 => Some(Key::Key6),
				Key7 => Some(Key::Key7),
				Key8 => Some(Key::Key8),
				Key9 => Some(Key::Key9),
				Key0 => Some(Key::Key0),
				T => Some(Key::Grab),
				Minus => Some(Key::Minus),
				Equals | Plus => Some(Key::Plus),
				_ => None,
			};
		}

		if modifiers.ctrl() {
			return match code {
				S => Some(Key::Save),
				X => Some(Key::CtrlX),
				Y => Some(Key::CtrlY),
				Z => Some(Key::CtrlZ),
				B => Some(Key::StartBake),
				_ => None,
			};
		}

		if modifiers.alt() {
			return match code {
				S => Some(Key::Save),
				X => Some(Key::AltX),
				Y => Some(Key::AltY),
				Z => Some(Key::AltZ),
				_ => None,
			};
		}

		None
	}
}

fn keycode_for(c: u8) -> Result<VirtualKeyCode> {
	let c = c as char;
	let c = c.to_ascii_lowercase();
	use VirtualKeyCode::*;
	Ok(match c {
		'a' => A,
		'b' => B,
		'c' => C,
		'd' => D,
		'e' => E,
		'f' => F,
		'g' => G,
		'h' => H,
		'i' => I,
		'j' => J,
		'k' => K,
		'l' => L,
		'm' => M,
		'n' => N,
		'o' => O,
		'p' => P,
		'q' => Q,
		'r' => R,
		's' => S,
		't' => T,
		'u' => U,
		'v' => V,
		'w' => W,
		'x' => X,
		'y' => Y,
		'z' => Z,
		_ => return err(format!("Sorry, key `{}` cannot be used for movement, use a-z", c)),
	})
}
