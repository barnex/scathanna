use super::internal::*;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Players(HashMap<ID, Player>);

impl Players {
	pub fn iter(&self) -> impl Iterator<Item = (ID, &Player)> {
		self.0.iter().map(|(id, p)| (*id, p))
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = (ID, &mut Player)> {
		self.0.iter_mut().map(|(id, p)| (*id, p))
	}

	pub fn insert(&mut self, player_id: ID, player: Player) {
		self.0.insert(player_id, player);
	}

	pub fn get(&self, player_id: ID) -> Option<&Player> {
		self.0.get(&player_id)
	}

	pub fn get_mut(&mut self, player_id: ID) -> Option<&mut Player> {
		self.0.get_mut(&player_id)
	}

	pub fn contains(&self, player_id: ID) -> bool {
		self.0.contains_key(&player_id)
	}

	pub fn remove(&mut self, player_id: ID) {
		self.0.remove(&player_id);
	}

	// A copy of the current entity IDs.
	// (Copy allows mutation while iterating).
	pub fn ids(&self) -> impl Iterator<Item = ID> + '_ {
		self.0.keys().copied()
	}

	// A copy of the current entity IDs.
	// (Copy allows mutation while iterating).
	pub fn copied_ids(&self) -> impl Iterator<Item = ID> {
		self.ids().collect::<SmallVec<_>>().into_iter()
	}
}

impl Index<ID> for Players {
	type Output = Player;

	// Get and unwrap player by ID.
	// Safe to be called downstream from handle_client_msg,
	// which checks that the player exists.
	fn index(&self, index: ID) -> &Self::Output {
		self.0.get(&index).expect("BUG: player ID not found")
	}
}

impl IndexMut<ID> for Players {
	// Get and unwrap player by ID.
	// Safe to be called downstream from handle_client_msg,
	// which checks that the player exists.
	fn index_mut(&mut self, index: ID) -> &mut Self::Output {
		self.0.get_mut(&index).expect("BUG: player ID not found")
	}
}
