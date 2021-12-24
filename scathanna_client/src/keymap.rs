use glutin::event::{ModifiersState, VirtualKeyCode};
use scathanna_core::*;

/// Map physical keys (e.g. 'A') to game-specific keys (e.g.: "Jump").
pub fn keymap(code: VirtualKeyCode, modifiers: ModifiersState) -> Option<Key> {
	use glutin::event::VirtualKeyCode::*;
	if modifiers.is_empty() {
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
