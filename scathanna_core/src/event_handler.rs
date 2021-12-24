use super::internal::*;

pub trait EventHandler {
	/// Handle keyboard input.
	fn on_key(&mut self, k: Key, pressed: bool);

	/// Handle mouse input.
	fn on_mouse_move(&mut self, x: f64, y: f64);

	fn tick(&mut self);

	fn draw(&mut self, width: u32, height: u32);
}
