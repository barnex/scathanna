use super::*;
use vector::*;

/// Matrix for rotation around an arbitrary axis.
/// https://en.wikipedia.org/wiki/Rotation_matrix#Rotation_matrix_from_axis_and_angle
pub fn rotation_matrix(axis: vec3, radians: f32) -> mat4 {
	let axis = axis.normalized();
	let (ux, uy, uz) = (axis.x(), axis.y(), axis.z());
	let c = f32::cos(radians);
	let s = f32::sin(radians);
	let c1 = 1.0 - c;
	mat4::from([
		[c + ux * ux * c1, uy * ux * c1 + uz * s, uz * ux * c1 - uy * s, 0.0],
		[ux * uy * c1 - uz * s, c + uy * uy * c1, uz * uy * c1 + ux * s, 0.0],
		[ux * uz * c1 + uy * s, uy * uz * c1 - ux * s, c + uz * uz * c1, 0.0],
		[0.0, 0.0, 0.0, 1.0],
	])
}

pub fn yaw_matrix(yaw: f32) -> mat4 {
	let sin = f32::sin(yaw);
	let cos = f32::cos(yaw);
	mat4::transpose([
		[cos, 0.0, -sin, 0.0], //
		[0.0, 1.0, 0.0, 0.0],
		[sin, 0.0, cos, 0.0],
		[0.0, 0.0, 0.0, 1.0],
	])
}

pub fn pitch_matrix(pitch: f32) -> mat4 {
	let sin = f32::sin(pitch);
	let cos = f32::cos(pitch);
	mat4::transpose([
		[1.0, 0.0, 0.0, 0.0], //
		[0.0, cos, -sin, 0.0],
		[0.0, sin, cos, 0.0],
		[0.0, 0.0, 0.0, 1.0],
	])
}

// A rotation matrix that yaws (rotate around Y), then pitches (rotate around X).
pub fn yaw_pitch_matrix(yaw: f32, pitch: f32) -> mat4 {
	&pitch_matrix(pitch) * &yaw_matrix(yaw)
}

pub fn translation_matrix(delta: vec3) -> mat4 {
	mat4::from([
		[1.0, 0.0, 0.0, 0.0], //
		[0.0, 1.0, 0.0, 0.0],
		[0.0, 0.0, 1.0, 0.0],
		[delta.x(), delta.y(), delta.z(), 1.0],
	])
}

pub fn scale_matrix(scl: f32) -> mat4 {
	mat4::from([
		[scl, 0.0, 0.0, 0.0], //
		[0.0, scl, 0.0, 0.0],
		[0.0, 0.0, scl, 0.0],
		[0.0, 0.0, 0.0, 1.0],
	])
}
