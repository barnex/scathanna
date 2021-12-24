use super::internal::*;

// #[derive(Default)]
// pub struct ScoreBoard {
// 	player_status: HashMap<ID, PlayerStatus>,
// }

#[derive(Default)]
pub struct PlayerScore {
	pub score: i32,
	pub liveness: Liveness,
}

pub enum Liveness {
	Live,
	Dead,
}

impl Default for Liveness {
	fn default() -> Self {
		Liveness::Live
	}
}
