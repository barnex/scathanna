use super::internal::*;

pub enum GameType {
	DeadMatch(DeadMatch),
	TeamMatch(TeamMatch),
}

impl GameType {
	pub fn is_team(&self) -> bool {
		use GameType::*;
		match self {
			DeadMatch(_) => false,
			TeamMatch(_) => true,
		}
	}
}

impl FromStr for GameType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		Ok(match s {
			"dm" | "deathmatch" => GameType::DeadMatch(default()),
			"tm" | "team" | "teammatch" => GameType::TeamMatch(default()),
			bad => return Err(anyhow!("unknown game type `{}`, options: `deathmatch`, `teammatch`", bad)),
		})
	}
}

#[derive(Default)]
pub struct DeadMatch;

#[derive(Default)]
pub struct TeamMatch {
	pub team_score: [i32; NUM_TEAMS],
	// TODO: store player teams here
}
