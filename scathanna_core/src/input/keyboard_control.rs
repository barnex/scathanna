use super::internal::*;

/// Direction an entity wants to move in,
/// based on the currently pressed keys and look direction.
pub fn walk_dir(yaw: f32, input: &InputState) -> vec3 {
	let mut dir = vec3::ZERO;
	if input.is_down(Key::Left) {
		dir.x -= 1.0;
	}
	if input.is_down(Key::Right) {
		dir.x += 1.0;
	}
	if input.is_down(Key::Forward) {
		dir.z -= 1.0;
	}
	if input.is_down(Key::Backward) {
		dir.z += 1.0;
	}
	if dir == vec3::ZERO {
		return vec3::ZERO;
	}
	let dir = yaw_matrix(-yaw).transform_point_ignore_w(dir);
	dir.safe_normalized()
}

/// Direction an entity wants to fly in,
/// based on the currently pressed keys and look direction.
pub fn fly_dir(yaw: f32, input: &InputState) -> vec3 {
	let mut fly_dir = walk_dir(yaw, input);
	if input.is_down(Key::Jump) {
		fly_dir.y += 1.0;
	}
	if input.is_down(Key::Crouch) {
		fly_dir.y -= 1.0;
	}
	fly_dir.safe_normalized()
}
