use super::internal::*;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum Team {
	Red = 0,
	Blue = 1,
	Green = 2,
}

pub const NUM_TEAMS: usize = 3;

use Team::*;

impl Team {
	pub fn random() -> Self {
		match rand::thread_rng().gen_range((Red as u8)..(NUM_TEAMS as u8)) {
			0 => Red,
			1 => Blue,
			2 => Green,
			_ => unreachable!(),
		}
	}

	/// To be multiplied by colors to make them team-color like.
	pub fn color_filter(self) -> vec3 {
		match self {
			Red => vec3(1.0, 0.5, 0.5),
			Blue => vec3(0.5, 0.5, 1.0),
			Green => vec3(0.5, 1.0, 0.3),
		}
	}
}

impl FromStr for Team {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		match s {
			"red" => Ok(Red),
			"blu" | "blue" => Ok(Blue),
			"green" => Ok(Green),
			bad => Err(anyhow!("unknown team `{}`, options: `red`, `blue`, `green`", bad)),
		}
	}
}

impl fmt::Display for Team {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Red => f.write_str("Red"),
			Blue => f.write_str("Blue"),
			Green => f.write_str("Green"),
		}
	}
}
