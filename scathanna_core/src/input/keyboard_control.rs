use super::internal::*;

/// Direction an entity wants to move in,
/// based on the currently pressed keys and look direction.
pub fn walk_dir(yaw: f32, input: &InputState) -> vec3 {
	let mut dir = vec3::ZERO;
	if input.is_down(Key::Left) {
		dir[X] -= 1.0;
	}
	if input.is_down(Key::Right) {
		dir[X] += 1.0;
	}
	if input.is_down(Key::Forward) {
		dir[Z] -= 1.0;
	}
	if input.is_down(Key::Backward) {
		dir[Z] += 1.0;
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
		fly_dir[Y] += 1.0;
	}
	if input.is_down(Key::Crouch) {
		fly_dir[Y] -= 1.0;
	}
	fly_dir.safe_normalized()
}
