use super::internal::*;

/// The World stores an in-memory representation of everything that can be drawn on the screen
/// (or equivalently, interacted with during gameplay).
///
/// consists of a map (immovable objects like walls, loaded from file)
/// and players (moving entities, dynamically added/removed).
///
/// Both ServerState and ClientState manage their copy of a World.
/// Each is responsible for mutating certain parts of it.
pub struct World {
	pub map: MapData,
	pub players: Players,
	pub entities: HashMap<EID, Entity>, // TODO: struct Entities. fn insert(Entity), etc.
	pub effects: Vec<Effect>,
}

impl World {
	/// Construct a GameState by loading a map from local disk,
	/// and adding the specified players and entities (e.g. received from a Server).
	pub fn from_map(map_name: &str, players: Players, entities: Entities) -> Result<Self> {
		Ok(Self {
			map: MapData::load(map_name)?,
			players,
			entities,
			effects: default(),
		})
	}

	/// Intersect a ray (e.g. a line of sight) with the map and players except `player_id`
	/// (to avoid shooting yourself right where the line of fire exits your hitbox).
	/// Returns intersection distance along the ray
	/// and  the ID of the nearest hit player, if any.
	pub fn intersect_except(&self, player_id: ID, ray: &DRay) -> Option<(f64, Option<ID>)> {
		let intersect_map = self.map.intersect(ray);

		let mut nearest = intersect_map.map(|t| (t, None));

		for (id, player) in self.players.iter() {
			if let Some(t) = player.intersect(ray) {
				if t < nearest.map(|(t, _)| t).unwrap_or(f64::INFINITY) && id != player_id {
					nearest = Some((t, Some(id)));
				}
			}
		}

		nearest
	}

	// Iterate over (a copy of) all player IDs.
	// (Does not hold self borrowed so can be conveniently used to self-modify).
	pub fn entity_ids(&self) -> impl Iterator<Item = EID> {
		self.entities.keys().copied().collect::<SmallVec<_>>().into_iter()
	}

	// Iterate over (a copy of) all player IDs.
	// (Does not hold self borrowed so can be conveniently used to self-modify).
	// pub fn player_ids(&self) -> impl Iterator<Item = ID> {
	// 	self.players.keys().copied().collect::<SmallVec<_>>().into_iter()
	// }
}
