use serde::{Deserialize, Serialize};

use super::internal::*;

/// An orientation with a yaw + pitch angle (but no roll).
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Orientation {
	/// Yaw angle with respect to negative Z axis (radians, positive = CCW seen from above).
	pub yaw: f32,
	/// Pitch angle with respect to the horizon (radians, positive = pitch up).
	pub pitch: f32,
}

impl Orientation {
	/// Unit vector in the looking direction.
	pub fn look_dir(&self) -> vec3 {
		let yaw = self.yaw;
		let pitch = self.pitch;
		let x = -f32::sin(yaw) * f32::cos(-pitch);
		let z = -f32::cos(yaw) * f32::cos(-pitch);
		let y = f32::sin(-pitch);
		vec3(x, y, z)
	}

	/// Looking direction, projected on the horizontal plane.
	pub fn look_dir_h(&self) -> vec3 {
		let yaw = self.yaw;
		let x = -f32::sin(yaw);
		let z = -f32::cos(yaw);
		let y = 0.0;
		vec3(x, y, z).normalized()
	}

	/// Direction 90 degrees right of look_dir
	pub fn look_right(&self) -> vec3 {
		let look = self.look_dir();
		vec3(-look.z, 0.0, look.x)
	}

	/// Rotate a vector by this orientation's yaw angle,
	/// around the vertical (Y) axis.
	pub fn apply_yaw(&self, rhs: vec3) -> vec3 {
		let s = self.yaw.sin();
		let c = self.yaw.cos();
		let (x, y, z) = rhs.into();

		vec3(
			x * c + z * s, //
			y,
			-x * s + z * c,
		)
	}

	#[must_use]
	pub fn add(&self, rhs: &Self) -> Self {
		Self {
			yaw: wrap_angle(self.yaw + rhs.yaw),
			pitch: wrap_angle(self.pitch + rhs.pitch),
		}
	}
}
