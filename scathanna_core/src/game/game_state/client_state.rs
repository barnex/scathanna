use super::internal::*;

/// ClientState stores a client's local copy of the World,
/// and provides the world-mutating methods that are allowed on the client side.
///
/// World mutations that are not allowed on the client side need to be requested from the server
/// (see ServerState).
pub struct ClientState {
	player_id: ID,
	world: World,
	hud: HUD,
}

impl ClientState {
	pub fn new(player_id: ID, world: World) -> Self {
		Self { player_id, world, hud: default() }
	}

	// __________________________________________________________ remote control

	/// Apply a diff to the game state.
	pub fn apply_server_msg(&mut self, msg: ServerMsg) {
		use ServerMsg::*;
		match msg {
			AddPlayer(player) => self.handle_add_player(player),
			MovePlayer(player_id, frame) => self.handle_move_player(player_id, frame),
			UpdatePlayer(player) => self.handle_update_player(player),
			ForceMovePlayer(position) => self.handle_force_move_player(position),
			UpdateEntity(entity) => self.handle_update_entity(entity),
			RemoveEntity(entity_id) => self.handle_remove_entity(entity_id),
			DropPlayer(player_id) => self.handle_drop_player(player_id),
			AddEffect(effect) => self.handle_add_effect(effect),
			RequestRespawn(spawn_point) => self.handle_request_respawn(spawn_point),
			UpdateHUD(update) => self.handle_update_hud(update),
			SwitchMap { .. } => panic!("BUG: SwitchMap must be handled by NetClient"),
			//SwitchMap {
			//	map_name,
			//	players,
			//	player_id,
			//	entities,
			//} => self.handle_switch_map(&map_name, players, player_id, entities),
		}
	}

	fn handle_add_player(&mut self, player: Player) {
		self.world.players.insert(player.id, player);
	}

	fn handle_move_player(&mut self, player_id: ID, frame: Frame) {
		if let Some(p) = self.world.players.get_mut(player_id) {
			p.skeleton.set_frame(frame)
		} else {
			eprintln!("client_state: handle_move_player: player #{} does not exist", player_id);
		}
	}

	// Update part of player state controlled by server: everything except frame.
	fn handle_update_player(&mut self, new: Player) {
		if !self.world.players.contains(new.id) {
			return;
		}
		let old = self.world.players.get_mut(new.id).unwrap();
		let mut new = new;
		new.local = old.local.clone();
		new.skeleton.set_frame(old.skeleton.frame());
		*old = new;
	}

	fn handle_force_move_player(&mut self, position: vec3) {
		self.local_player_mut().skeleton.position = position;
	}

	fn handle_update_entity(&mut self, entity: Entity) {
		self.world.entities.insert(entity.id(), entity);
	}

	fn handle_remove_entity(&mut self, entity_id: EID) {
		self.world.entities.remove(&entity_id);
	}

	fn handle_drop_player(&mut self, player_id: ID) {
		self.world.players.remove(player_id);
	}

	fn handle_request_respawn(&mut self, spawn_point: SpawnPoint) {
		self.local_player_mut().next_spawn_point = spawn_point.position();
		self.local_player_mut().skeleton.velocity = vec3::ZERO;
		self.local_player_mut().skeleton.orientation.pitch = 0.0;
	}

	fn handle_add_effect(&mut self, effect: Effect) {
		self.world.effects.push(effect)
	}

	//fn handle_switch_map(&mut self, map_name: &str, players: Players, player_id: ID, entities: Entities) {
	//	self.player_id = player_id;
	//	// TODO: print error about missing files, exit(1)
	//	self.world = World::from_map(map_name, players, entities).expect("load map");
	//}

	fn handle_update_hud(&mut self, upd: HUDUpdate) {
		self.hud.update(upd)
	}

	// __________________________________________________________ local control

	pub fn tick(&mut self, input_state: &InputState, dt: f32) -> ClientMsgs {
		let upd = self.control_player(input_state, dt);
		self.extrapolate_other_players(dt);
		self.animate_players(dt);
		self.tick_effects(dt);
		self.hud.tick(dt);
		upd
	}

	/// Apply a message by the local client, without round-tripping to the server.
	/// This only applies:
	///
	///   * updates to the local player, so that position/orientation don't lag by one round-trip-time.
	///   * visual effects, because these don't otherwise interact with the game state.
	///
	/// Other messages are not applied locally, but go to the server
	/// and eventually mutate the local GameState via `apply_server_msg`.
	fn apply_self_msgs(&mut self, msgs: &ClientMsgs) {
		use ClientMsg::*;
		for msg in msgs {
			match msg {
				MovePlayer { .. } => (/*already applied locally by control*/),
				AddEffect(effect) => self.world.effects.push(effect.clone()),
				HitPlayer { .. } => (/* handled by server*/),
				ReadyToSpawn => (/*handled by server*/),
				Command(_) => (/*handled by server*/),
			}
		}
	}

	/// Control a player via keyboard/mouse
	#[must_use]
	fn control_player(&mut self, input_state: &InputState, dt: f32) -> ClientMsgs {
		let mut upd = ClientMsgs::new();
		let mut clone = self.local_player().clone();

		clone.control(&mut upd, input_state, &self.world, dt);

		*self.local_player_mut() = clone;

		self.apply_self_msgs(&upd);
		upd
	}

	/// Extrapolate other player's positions based on their last know velocity.
	/// This greatly reduces positional stutter in the face of network latency.
	fn extrapolate_other_players(&mut self, dt: f32) {
		for (id, player) in self.world.players.iter_mut() {
			if id != self.player_id {
				player.skeleton.position += dt * player.skeleton.velocity;
			}
		}
	}

	/// Animate the players feet if they are moving.
	/// This is done locally by each client (do not send
	/// feet position over the network all the time).
	fn animate_players(&mut self, dt: f32) {
		for (_, player) in self.world.players.iter_mut() {
			player.animate_feet(dt)
		}
	}

	//___________________________________________________________________________ effects

	/// Advance visual effects in time.
	/// This is done locally (after creation,
	/// visual effects do not need to synchronize over the network).
	fn tick_effects(&mut self, dt: f32) {
		Self::update_effects_ttl(&mut self.world.effects, dt);
	}

	// decrease effect's TTL by `dt` and remove effects past their TTL.
	fn update_effects_ttl(effects: &mut Vec<Effect>, dt: f32) {
		let mut i = 0;
		while i < effects.len() {
			effects[i].ttl -= dt;
			if effects[i].ttl <= 0.0 {
				effects.swap_remove(i);
			} else {
				i += 1;
			}
		}
	}

	// __________________________________________________________ accessors

	pub fn world(&self) -> &World {
		&self.world
	}

	/// The player controlled by this client.
	pub fn local_player(&self) -> &Player {
		&self.world.players[self.player_id]
	}

	pub fn local_player_mut(&mut self) -> &mut Player {
		&mut self.world.players[self.player_id]
	}

	pub fn player_id(&self) -> ID {
		self.player_id
	}

	pub fn hud(&self) -> &HUD {
		&self.hud
	}
}

impl ClientState {}
