use super::internal::*;
use ServerMsg::*;

/// A game server's mutable state and business logic.
///
/// Owned and controlled by a NetServer, who adds an RPC layer on top.
pub struct ServerState {
	next_player_id: ID, // TODO: this belongs in server.
	rng: RefCell<StdRng>,

	maplist: Vec<String>,
	curr_map: usize,

	world: World,
	// score is kept by name, not player_id so that:
	//  * a player can drop out and rejoin without losing their score
	//  * scores can displayed even after a user dropped out
	score: HashMap<String, i32>,
}

impl ServerState {
	pub fn new(maplist: Vec<String>) -> Result<Self> {
		println!("maplist: {}", maplist.join(", "));
		// TODO: verify all maps on startup.

		let curr_map = 0;
		let world = World::with_players(&maplist[curr_map], default())?;
		Ok(Self {
			maplist,
			curr_map,
			world,
			next_player_id: 1,
			rng: RefCell::new(StdRng::seed_from_u64(123)),
			score: default(),
		})
	}

	pub fn join_new_player(&mut self, join_msg: JoinMsg) -> (ID, ServerMsgs) {
		let player_id = self.new_player_id();
		let spawn_point = self.pick_spawn_point();
		let player = Player::new(player_id, spawn_point.position(), spawn_point.orientation(), join_msg.name, join_msg.avatar_id);

		// insert score 0 or keep previous score if same player joined earlier.
		self.score.entry(player.name.clone()).or_default();
		self.world.players.insert(player_id, player);
		let player = self.world.players.get(&player_id).unwrap().clone();

		let resp = vec![
			// Newly connected client expects SwitchMap message to be first.
			Envelope {
				to: Addressee::Just(player_id),
				msg: SwitchMap {
					map_name: self.map_name().into(),
					players: self.world.players.clone(),
					player_id,
				},
			},
			log(format!("{} joined", &player.name)), //
			// send the client the full current game state and their Player ID.
			Envelope {
				to: Addressee::All,
				msg: AddPlayer(player),
			},
		];

		(player_id, resp)
	}

	// ____________________________________________________________________________ event handling

	/// Respond to message sent by a player.
	pub fn handle_client_msg(&mut self, player_id: ID, msg: ClientMsg) -> ServerMsgs {
		// check that the player has not been disconnected in a network race.
		// after this check, all downstream methods may safely use `self.player(id)`,
		// as we will never remove a player while handling client messages.
		if !self.world.players.contains_key(&player_id) {
			return ServerMsgs::new();
		}

		let mut resp = Vec::new();
		match msg {
			ClientMsg::MovePlayer(frame) => self.handle_move_player(&mut resp, player_id, frame),
			ClientMsg::ReadyToSpawn => self.handle_ready_to_respawn(&mut resp, player_id),
			ClientMsg::AddEffect { effect } => self.handle_add_effect(&mut resp, player_id, effect),
			ClientMsg::HitPlayer { victim } => self.handle_hit_player(&mut resp, player_id, victim),
		};
		resp
	}

	pub fn handle_ready_to_respawn(&mut self, resp: &mut ServerMsgs, player_id: ID) {
		self.set_spawned(resp, player_id, true);
	}

	fn set_spawned(&mut self, resp: &mut ServerMsgs, player_id: ID, spawned: bool) {
		self.update_player(resp, self.player(player_id).clone().with(|p| p.spawned = spawned))
	}

	fn update_player(&mut self, resp: &mut ServerMsgs, new: Player) {
		if self.world.players.contains_key(&new.id) {
			self.world.players.insert(new.id, new.clone());
			resp.push(Envelope {
				to: Addressee::All,
				msg: UpdatePlayer(new),
			})
		}
	}

	// Handle a client's MovePlayer message:
	// update the server's world and broadcast the move to all other clients.
	pub fn handle_move_player(&mut self, resp: &mut ServerMsgs, player_id: ID, frame: Frame) {
		self.player_mut(player_id).skeleton.set_frame(frame);

		resp.push(Envelope {
			to: Addressee::Not(player_id),
			msg: ServerMsg::MovePlayer {
				player_id,
				frame: self.player(player_id).skeleton.frame(),
			},
		});

		self.do_lava_damage(resp, player_id);
	}

	fn do_lava_damage(&mut self, resp: &mut ServerMsgs, player_id: ID) {
		let player = self.player(player_id);
		if !player.spawned {
			return;
		}

		let probe = player.position() - 0.2 * vec3::EY;
		if self.world.map.voxels.at(probe.to_ivec()) == VoxelType::LAVA {
			push_effect(resp, Effect::particle_explosion(player.position(), RED));
			resp.push(log(format!("{} went swimming in hot lava", &player.name)));
			resp.push(Envelope {
				to: Addressee::Just(player_id),
				msg: HUDMessage("You fell in lava".to_owned()),
			});
			self.kill_player(resp, player_id);
		}
	}

	fn kill_player(&mut self, resp: &mut ServerMsgs, player_id: ID) {
		self.set_spawned(resp, player_id, false);
		resp.push(Envelope {
			to: Addressee::Just(player_id),
			msg: RequestRespawn(self.pick_spawn_point()),
		});
	}

	// Handle a client's AddEffect message: just broadcast to other clients.
	// There is little point in adding visual effects to the server's world.
	pub fn handle_add_effect(&mut self, resp: &mut ServerMsgs, player_id: ID, effect: Effect) {
		resp.push(Envelope {
			to: Addressee::Not(player_id),
			msg: ServerMsg::AddEffect(effect),
		})
	}

	// Handle a client saying they just shot a player.
	// We trust clients not to lie about this.
	//
	// Hitting players is computed client-side for latency reasons:
	// a client always sees other players at a location that lags slightly behind.
	// If a client hits a player where they see them on their screen, then it should
	// count as a hit regardless of latency.
	// Otherwise players with more than about 30ms latency would be at a noticeable disadvantage.
	pub fn handle_hit_player(&mut self, resp: &mut ServerMsgs, player_id: ID, victim_id: ID) {
		if !self.world.players.contains_key(&victim_id) {
			// victim has disconnected in a network race.
			return;
		}

		if !self.player(victim_id).spawned {
			// Player is hit again before respawn: ignore.
			// (this can happen in a network race, because hitting is determined client-side)
			return;
		}

		// TODO: does not belong here
		*self.score.entry(self.player(player_id).name.clone()).or_default() += 1;

		self.kill_player(resp, victim_id);

		let victim = self.player(victim_id);

		resp.push(Envelope {
			to: Addressee::All,
			msg: AddEffect(Effect::particle_explosion(victim.camera().position, WHITE)),
		});

		resp.push(Envelope {
			to: Addressee::Just(victim_id),
			msg: HUDMessage(format!("You got confettied by {}", self.player(player_id).name)),
		});
		resp.push(Envelope {
			to: Addressee::Just(player_id),
			msg: HUDMessage(format!("You confettied {}", self.player(victim_id).name)),
		});

		resp.push(log(format!(
			"{} ({}) confetti'd {} ({})",
			self.player(player_id).name,
			self.score(player_id),
			victim.name,
			self.score(victim.id)
		)));
	}

	pub fn handle_drop_player(&mut self, resp: &mut ServerMsgs, player_id: ID) {
		self.world.players.remove(&player_id);
		resp.push(Envelope {
			to: Addressee::All,
			msg: DropPlayer { player_id },
		})
	}

	// ________________________________________________________________________ accessors

	fn score(&self, player_id: ID) -> i32 {
		self.score.get(&self.player(player_id).name).copied().unwrap_or_default()
	}

	// A fresh, unique player number.
	fn new_player_id(&mut self) -> ID {
		let id = self.next_player_id;
		self.next_player_id += 1;
		id
	}

	fn pick_spawn_point(&self) -> SpawnPoint {
		let n = self.world.map.metadata.spawn_points.len();
		if n == 0 {
			panic!("map has no spawn points")
		}
		let i = self.rng.borrow_mut().gen_range(0..n);
		self.world.map.metadata.spawn_points[i]
	}

	pub fn map_name(&self) -> &str {
		&self.world.map.name
	}

	// Get and unwrap player by ID.
	// Safe to be called downstream from handle_client_msg,
	// which checks that the player exists.
	fn player(&self, player_id: ID) -> &Player {
		self.world.players.get(&player_id).expect("BUG: player ID not found")
	}

	// Get and unwrap player by ID.
	// Safe to be called downstream from handle_client_msg,
	// which checks that the player exists.
	fn player_mut(&mut self, player_id: ID) -> &mut Player {
		self.world.players.get_mut(&player_id).expect("BUG: player ID not found")
	}
}

// construct a LogMessage to be sent over the network,
// but log the message to local stdout first.
fn log(message: String) -> Envelope<ServerMsg> {
	println!("{}", &message);
	Envelope {
		to: Addressee::All,
		msg: LogMessage(message),
	}
}

fn push_effect(resp: &mut ServerMsgs, effect: Effect) {
	resp.push(Envelope {
		to: Addressee::All,
		msg: AddEffect(effect),
	})
}
