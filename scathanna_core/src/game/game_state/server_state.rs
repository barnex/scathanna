use super::internal::*;
use ServerMsg::*;

/// A game server's mutable state and business logic.
///
/// Owned and controlled by a NetServer, who adds an RPC layer on top.
pub struct ServerState {
	next_player_id: ID,
	rng: RefCell<StdRng>,
	maplist: Vec<String>,
	curr_map: usize,

	world: World,

	score: HashMap<ID, i32>,
	gametype: GameType,

	// Tilt the odds of good powerups in favor of the worst player.
	enable_levelling: bool,

	pub pending_diffs: Vec<Envelope<ServerMsg>>,
}

// invulnerability duration (e.g. after respawn, party hat protection).
const DEFAULT_INVULN_TTL: f32 = 1.5;

impl ServerState {
	pub fn new(opts: ServerOpts) -> Result<Self> {
		let ServerOpts { maplist, game_type, .. } = opts;
		let enable_levelling = true; // TODO

		println!("maplist: {}", maplist.join(", "));
		println!("game type: {}", &game_type);

		if maplist.len() == 0 {
			return Err(anyhow!("server: maplist: need at least one map"));
		}
		// TODO: verify all maps on startup.

		let curr_map = 0;

		let world = World::from_map(&maplist[curr_map], default(), default())?;

		let mut slf = Self {
			enable_levelling,
			gametype: game_type.parse()?,
			maplist,
			curr_map,
			pending_diffs: default(),
			world,
			next_player_id: 1,
			rng: RefCell::new(StdRng::seed_from_u64(123)),
			score: default(),
		};
		slf.populate_all_pickups();
		Ok(slf)
	}

	// ============================================================================ event handling

	// ____________________________________________________________________________ join

	/// Add a new player to the game and return their unique ID.
	pub fn join_new_player(&mut self, join_msg: JoinMsg) -> ID {
		let player_id = self.new_player_id();
		let spawn_point = self.pick_spawn_point();
		let player = Player::new(player_id, spawn_point.position(), spawn_point.orientation(), join_msg.name, join_msg.avatar_id, join_msg.team);

		// insert score 0 or keep previous score if same player joined earlier.
		self.score.entry(player_id).or_default();
		self.world.players.insert(player_id, player);
		let player = self.world.players[player_id].clone();

		self.pending_diffs.push(
			// send the client the full current game state and their Player ID.
			// Newly connected client expects SwitchMap message to be first.
			SwitchMap {
				map_name: self.map_name().into(),
				players: self.world.players.clone(),
				entities: self.world.entities.clone(),
				player_id,
			}
			.to_just(player_id),
		);

		self.log(format!("{} joined", &player.name)); //
		self.hud_message(player_id, format!("Welcome to {}.", self.map_name()));
		self.pending_diffs.push(PlaySound(SoundEffect::raw("begin")).to_just(player_id));
		self.pending_diffs.push(AddPlayer(player).to_all());
		self.broadcast_scores_mini();

		player_id
	}

	// ____________________________________________________________________________ msg

	/// Respond to message sent by a player.
	pub fn handle_client_msg(&mut self, player_id: ID, msg: ClientMsg) {
		// check that the player has not been disconnected in a network race.
		// after this check, all downstream methods may safely use `self.player(id)`,
		// as we will never remove a player while handling client messages.
		if !self.world.players.contains(player_id) {
			return;
		}

		use ClientMsg::*;
		match msg {
			MovePlayer(frame) => self.handle_move_player(player_id, frame),
			ReadyToSpawn => self.handle_ready_to_respawn(player_id),
			AddEffect(effect) => self.handle_add_effect(player_id, effect),
			PlaySound(sound) => self.handle_play_sound(player_id, sound),
			HitPlayer(victim_id) => self.handle_hit_player(player_id, victim_id),
			Command(cmd) => self.handle_command(player_id, cmd),
		};
	}

	// ____________________________________________________________________________ tick

	pub fn handle_tick(&mut self, dt: f32) {
		self.tick_pickups(dt);
		self.tick_players(dt);
	}

	// ____________________________________________________________________________ move

	// Handle a client's MovePlayer message:
	// update the server's world and broadcast the move to all other clients.
	fn handle_move_player(&mut self, player_id: ID, frame: Frame) {
		self.record_move_player(player_id, frame);

		self.handle_lava(player_id);
		self.handle_pickups(player_id);
		self.handle_off_world(player_id);
	}

	// After a player has moved: check if they get lava damage.
	fn handle_lava(&mut self, player_id: ID) {
		// don't kill dead player again
		if !self.player(player_id).spawned {
			return;
		}

		// party hat grants lava immunity
		if self.player(player_id).powerup == Some(EKind::PartyHat) {
			return;
		}

		if self.player(player_id).is_on_lava(&self.world) {
			if self.try_kill_player(player_id, None) {
				self.increment_score(player_id, -1);
				self.record_add_effect(Effect::particle_explosion(self.player(player_id).center(), RED)); // TODO: duplicate with confetti
				self.broadcast_sound_at("death_lava", self.player(player_id).center(), 3.0);
				self.log(format!("{} went swimming in hot lava", &self.player(player_id).name));
				self.pending_diffs.push(UpdateHUD(HUDUpdate::Message("You fell in lava".to_owned())).to_just(player_id));
			}
		}
	}

	fn handle_off_world(&mut self, player_id: ID) {
		if !self.player(player_id).spawned {
			return;
		}

		const WORLD_BOTTOM: f32 = -1024.0;
		if self.player(player_id).position().y() < WORLD_BOTTOM {
			self.kill_player(player_id);
			self.log(format!("{} fell off the world", &self.player(player_id).name));
			self.hud_message(player_id, "You fell off the world".to_owned());
		}
	}

	// place at gift box at every pick-up point
	// (called when a map first loads).
	fn populate_all_pickups(&mut self) {
		for i in 0..self.world.map.metadata.pickup_points.len() {
			self.populate_pickup(i);
		}
	}

	// re-populate taken pick-up points from time to time.
	fn tick_pickups(&mut self, _dt: f32) {
		let mut rng = rand::thread_rng();
		for (i, pp) in self.world.map.metadata.pickup_points.iter().copied().collect::<SmallVec<_>>().into_iter().enumerate() {
			if pp.taken && rng.gen::<f32>() < 0.001 {
				self.populate_pickup(i);
			}
		}
	}

	// re-populate pick-up point `i`.
	pub fn populate_pickup(&mut self, i: usize) {
		let pickup_point = &mut self.world.map.metadata.pickup_points[i];
		pickup_point.taken = false;
		let entity = Entity::new(pickup_point.position(), EKind::GiftBox { pickup_point_id: Some(i) });
		self.record_add_entity(entity);
	}

	// after a player has moved: check if they're on a pick-up.
	pub fn handle_pickups(&mut self, player_id: ID) {
		if !self.player(player_id).spawned {
			return;
		}

		let player_bounds = self.player(player_id).skeleton.bounds();
		for eid in self.entity_ids() {
			if touches(&player_bounds, self.entity(eid).unwrap().bounds()) {
				self.pick_up(player_id, eid)
			}
		}
	}

	// pick-up a powerup.
	// Gift boxes are replaced by a random other powerup.
	fn pick_up(&mut self, player_id: ID, pickup_id: EID) {
		let powerup = match self.entity(pickup_id).unwrap().kind {
			EKind::GiftBox { pickup_point_id: pickup_point } => {
				if let Some(pickup_point) = pickup_point {
					self.world.map.metadata.pickup_points.get_mut(pickup_point).map(|pp| pp.taken = true);
				}
				self.randomish_powerup_for(player_id)
			}
			other => other,
		};
		let powerup_name = powerup.as_str().replace("_", " "); // "xmas_hat" => "xmas hat"

		self.record_remove_entity(pickup_id);
		self.record_apply_to_player(player_id, |p| p.powerup = Some(powerup));
		self.broadcast_sound_at(powerup.as_str(), self.player(player_id).position(), 1.0);

		self.hud_message(player_id, format!("You got the {}\n[{}]", powerup_name, powerup.description()));
		self.log(format!("{} has the {}", &self.player(player_id).name, powerup_name));
	}

	// Pick a random-ish powerup for a player.
	// Levelling gives better powerups to worse players.
	fn randomish_powerup_for(&self, player_id: ID) -> EKind {
		if self.enable_levelling {
			if self.has_best_score(player_id) && by_chance(0.3) {
				return EKind::GiftBox { pickup_point_id: None };
			}
			if self.has_worst_score(player_id) && by_chance(0.3) {
				return EKind::BerserkerHelmet;
			}
		}
		EKind::random_powerup_except(self.player(player_id).powerup)
	}

	fn tick_players(&mut self, dt: f32) {
		for player_id in self.world.players.copied_ids() {
			self.tick_player(player_id, dt)
		}
	}

	fn tick_player(&mut self, player_id: ID, dt: f32) {
		// invulnerability wears off after some time
		if let Some(ttl) = self.player_mut(player_id).invulnerability_ttl {
			let ttl = ttl - dt;
			self.player_mut(player_id).invulnerability_ttl = if ttl > 0.0 { Some(ttl) } else { None }
		}
	}

	// ____________________________________________________________________________ shoot

	// Handle a client saying they just shot a player.
	// We trust clients not to lie about this.
	//
	// Hitting players is computed client-side for latency reasons:
	// a client always sees other players at a location that lags slightly behind.
	// If a client hits a player where they see them on their screen, then it should
	// count as a hit regardless of latency.
	// Otherwise players with more than about 30ms latency would be at a noticeable disadvantage.
	pub fn handle_hit_player(&mut self, player_id: ID, victim_id: ID) {
		if !self.world.players.contains(victim_id) {
			// victim has disconnected in a network race.
			return;
		}

		if !self.player(victim_id).spawned {
			// Player is hit again before respawn: ignore.
			// (this can happen in a network race, because hitting is determined client-side)
			return;
		}

		if self.try_kill_player(victim_id, Some(player_id)) {
			self.increment_score(player_id, 1);
			self.broadcast_sound_at("kill", self.player(victim_id).position(), 1.0);
			self.hud_message(victim_id, format!("You got confettied by {}", self.player(player_id).name));
			self.hud_message(player_id, format!("You confettied {}", self.player(victim_id).name));
			self.log(format!(
				"{} ({}) confetti'd {} ({})",
				self.player(player_id).name,
				self.score(player_id),
				self.player(victim_id).name,
				self.score(victim_id)
			));
		}
	}

	fn increment_score(&mut self, player_id: ID, delta: i32) {
		*self.score.entry(player_id).or_default() += delta;

		let team = self.player(player_id).team;

		match &mut self.gametype {
			GameType::DeadMatch(_) => (),
			GameType::TeamMatch(tm) => tm.team_score[team as usize] += delta,
		}

		self.broadcast_scores_mini();
	}

	// broadcast scores to the top-left corner of player HUDs.
	fn broadcast_scores_mini(&mut self) {
		for player_id in self.world.players.ids() {
			self.pending_diffs.push(UpdateHUD(HUDUpdate::Score(self.format_scores_for(player_id))).to_just(player_id));
		}
	}

	// broadcast scores to the top-left corner of player HUDs.
	fn broadcast_scoreboard(&mut self) {
		self.pending_diffs.push(UpdateHUD(HUDUpdate::Message(self.format_scoreboard())).to_all());
	}

	fn format_scores_for(&self, player_id: ID) -> String {
		use Team::*;
		match &self.gametype {
			GameType::DeadMatch(_) => format!("score: {}", self.score(player_id)),
			GameType::TeamMatch(tm) => format!(
				"Red: {} | Blu: {} | Green: {}\nYou: {}",
				tm.team_score[Red as usize],
				tm.team_score[Blue as usize],
				tm.team_score[Green as usize],
				self.score(player_id)
			),
		}
	}

	fn format_scoreboard(&self) -> String {
		let mut s = String::new();

		use Team::*;
		s.push_str(&match &self.gametype {
			GameType::DeadMatch(_) => String::new(),
			GameType::TeamMatch(tm) => format!(
				"Red: {} | Blu: {} | Green: {}\n",
				tm.team_score[Red as usize], tm.team_score[Blue as usize], tm.team_score[Green as usize],
			),
		});

		let mut scores = self.score.iter().map(|(p, s)| (p.to_owned(), *s)).collect::<SmallVec<(ID, i32)>>();
		scores.sort_by_key(|(_, score)| *score);

		for (player_id, score) in scores {
			// player may have disconnected by the time we show scores, so ID not necessarily valid.
			if let Some(player) = self.world.players.get(player_id) {
				s.push_str(&format!("{:20} {}\n", &player.name, score))
			}
		}
		s
	}

	// ____________________________________________________________________________ respawn

	pub fn handle_ready_to_respawn(&mut self, player_id: ID) {
		// levelling: best player gets no spawn protection
		let spawn_protect = match (self.enable_levelling, self.has_best_score(player_id)) {
			(true, true) => None,
			(_, _) => Some(DEFAULT_INVULN_TTL),
		};
		self.record_apply_to_player(player_id, |p| {
			p.spawned = true;
			p.invulnerability_ttl = spawn_protect // spawn kill protection
		});
		self.broadcast_sound_at("respawn", self.player(player_id).center(), 1.0)
	}

	// ____________________________________________________________________________ effects

	// Handle a client's AddEffect message: just broadcast to other clients.
	// There is little point in adding visual effects to the server's world.
	pub fn handle_add_effect(&mut self, player_id: ID, effect: Effect) {
		self.pending_diffs.push(AddEffect(effect).to_not(player_id))
	}

	// Handle a client's PlaySound message: just broadcast to other clients.
	pub fn handle_play_sound(&mut self, player_id: ID, sound: SoundEffect) {
		self.pending_diffs.push(PlaySound(sound).to_not(player_id))
	}

	fn broadcast_sound_at(&mut self, clip_name: &'static str, location: vec3, volume: f32) {
		self.broadcast_sound(SoundEffect::spatial(clip_name, location, volume))
	}

	fn broadcast_sound(&mut self, sound: SoundEffect) {
		self.pending_diffs.push(PlaySound(sound).to_all())
	}

	// ____________________________________________________________________________ drop

	pub fn handle_drop_player(&mut self, player_id: ID) {
		self.log(format!("{} left", &self.player(player_id).name));
		self.world.players.remove(player_id);
		self.score.remove(&player_id);
		self.pending_diffs.push(DropPlayer(player_id).to_not(player_id));
	}

	// ____________________________________________________________________________ commands

	fn handle_command(&mut self, player_id: ID, cmd: String) {
		println!("command by #{} ({}): `{}`", player_id, &self.player(player_id).name, &cmd);
		match self.handle_command_with_result(player_id, cmd) {
			Ok(()) => println!("ok"),
			Err(e) => {
				println!("{}", e);
				self.pending_diffs.push(UpdateHUD(HUDUpdate::Log(format!("error: {}", e))).to_just(player_id));
			}
		}
	}

	fn handle_command_with_result(&mut self, player_id: ID, cmd: String) -> Result<()> {
		let split = cmd.split_ascii_whitespace().collect::<Vec<_>>();
		if split.len() == 0 {
			return Ok(()); // empty command
		}
		let cmd = split[0];
		let args = &split[1..];
		match cmd {
			"summon" => self.summon(player_id, one_arg(args)?),
			"switch" => self.switch_map(one_arg(args)?),
			unknown => Err(anyhow!("unknown command: `{}`", unknown)),
		}
	}

	fn summon(&mut self, player_id: ID, arg: &str) -> Result<()> {
		const SUMMON_DIST: f32 = 5.0;
		let player = self.player(player_id);
		let position = player.position() + SUMMON_DIST * player.orientation().look_dir_h();
		Ok(self.record_add_entity(Entity::new(position, EKind::from_str(arg)?)))
	}

	fn switch_map(&mut self, arg: &str) -> Result<()> {
		let i = self.maplist.iter().position(|name| name == arg).ok_or(anyhow!("`{}` not in map list", arg))?;

		let mut world2 = World::from_map(&self.maplist[i], default(), default())?;

		self.curr_map = i;
		mem::swap(&mut world2.players, &mut self.world.players);
		mem::swap(&mut self.world, &mut world2);

		for player_id in self.player_ids() {
			self.record_apply_to_player(player_id, |p| p.spawned = false);

			self.pending_diffs.push(
				ServerMsg::SwitchMap {
					player_id,
					map_name: self.map_name().to_owned(),
					players: self.world.players.clone(),
					entities: self.world.entities.clone(),
				}
				.to_just(player_id),
			);

			// request respawn but also force player to move to the respawn point immediately.
			// (normally when we request a respawn the player stays at their death location
			// so they can see who killed them. But when switching maps, that location could be out of the world).
			let spawn_point = self.pick_spawn_point();
			self.pending_diffs.push(ForceMovePlayer(spawn_point.position()).to_just(player_id));
			self.pending_diffs.push(RequestRespawn(spawn_point).to_just(player_id));
		}

		self.broadcast_scoreboard();
		self.score = default();
		// TODO: reset GameType state.

		self.populate_all_pickups();

		Ok(())
	}

	// ________________________________________________________________________ HUD

	// Send a message to be shown in the center of one player's screen.
	// E.g.: "you got killed by ...".
	fn hud_message(&mut self, player_id: ID, message: String) {
		self.pending_diffs.push(UpdateHUD(HUDUpdate::Message(message)).to_just(player_id));
	}

	// Send a message to shown in the logs of all players.
	// E.g. "A killed B".
	pub fn log(&mut self, message: String) {
		println!("{}", &message);
		self.pending_diffs.push(UpdateHUD(HUDUpdate::Log(message)).to_all());
	}

	// ________________________________________________________________________ mutators

	// add an entity to the world and record as pending diff
	fn record_add_entity(&mut self, entity: Entity) {
		self.world.entities.insert(entity.id(), entity.clone());
		self.pending_diffs.push(UpdateEntity(entity).to_all());
	}

	// remove entity and record as pending diff
	fn record_remove_entity(&mut self, eid: EID) {
		self.world.entities.remove(&eid);
		self.pending_diffs.push(RemoveEntity(eid).to_all());
	}

	// move a player and record as pending diff
	fn record_move_player(&mut self, player_id: ID, frame: Frame) {
		self.player_mut(player_id).skeleton.set_frame(frame);
		self.pending_diffs.push(MovePlayer(player_id, self.player(player_id).skeleton.frame()).to_not(player_id));
	}

	// apply `f` to player and record as pending diff
	fn record_apply_to_player<F: Fn(&mut Player)>(&mut self, player_id: ID, f: F) {
		f(self.player_mut(player_id));
		self.pending_diffs.push(UpdatePlayer(self.player(player_id).clone()).to_all());
	}

	// Try to kill a player, return true if successful.
	#[must_use]
	fn try_kill_player(&mut self, victim_id: ID, aggressor_id: Option<ID>) -> bool {
		// temporarily invulnerable.
		if self.player(victim_id).invulnerability_ttl.is_some() {
			println!("{} invulnerable", self.player(victim_id).name);
			if aggressor_id.is_some() {
				self.record_add_effect(Effect::ricochet(self.player(victim_id).center(), self.player(victim_id).team.color_filter()));
			}
			return false;
		}

		// same team.
		// TODO: team should only be accessibly in team match.
		if let Some(aggressor_id) = aggressor_id {
			if self.gametype.is_team() && self.player(aggressor_id).team == self.player(victim_id).team {
				return false;
			}
		}

		// Party hat powerup grants immunity against one shot.
		if self.player(victim_id).powerup == Some(EKind::PartyHat) {
			self.record_add_effect(Effect::particle_explosion(self.player(victim_id).camera().position, WHITE));
			self.record_apply_to_player(victim_id, |p| {
				p.powerup = None;
				p.invulnerability_ttl = Some(DEFAULT_INVULN_TTL)
			});
			self.log(format!("{} was saved by their party hat", self.player(victim_id).name));
			self.broadcast_sound_at("protect", self.player(victim_id).center(), 1.0);
			return false;
		}

		self.kill_player(victim_id);
		true
	}

	// unconditionally kill a player
	fn kill_player(&mut self, player_id: ID) {
		// drop powerup:
		//  - always drop gift boxes
		//  - hats are dropped by chance (to avoid hats staying around forever).
		if let Some(powerup) = self.player(player_id).powerup {
			if powerup.is_kind(&EKind::GiftBox { pickup_point_id: None }) || by_chance(self.hat_drop_chance(player_id)) {
				self.record_add_entity(Entity::new(self.player(player_id).position(), powerup));
			}
		}

		self.record_apply_to_player(player_id, |p| {
			p.powerup = None;
			p.spawned = false;
		});

		self.record_add_effect(Effect::particle_explosion(self.player(player_id).center(), WHITE));

		self.pending_diffs.push(RequestRespawn(self.pick_spawn_point()).to_just(player_id));
	}

	// levelling: best player is much more likely to drop their hat on death.
	fn hat_drop_chance(&self, player_id: ID) -> f32 {
		if self.enable_levelling {
			if self.has_best_score(player_id) {
				println!("best player should drop their hat");
				return 0.9;
			}
			if self.has_worst_score(player_id) {
				println!("worst player should keep their hat");
				return 0.1;
			}
		}
		// chosen so that hats occasionally disappear, else they stay around forever.
		0.75
	}

	// record a new effect in the pending diffs
	fn record_add_effect(&mut self, effect: Effect) {
		// effects are not used by server
		self.pending_diffs.push(AddEffect(effect).to_all())
	}

	// ________________________________________________________________________ accessors

	fn score(&self, player_id: ID) -> i32 {
		self.score.get(&player_id).copied().unwrap_or(0)
	}

	// Has this player the best score so far?
	// (used for levelling: give them some disadvantage).
	fn has_best_score(&self, player_id: ID) -> bool {
		let score = self.score(player_id);
		score == self.best_score() && score != self.worst_score()
	}

	// Has this player the worst score so far?
	// (used for levelling: give them some advantage).
	fn has_worst_score(&self, player_id: ID) -> bool {
		let score = self.score(player_id);
		score == self.worst_score() && score != self.best_score()
	}

	fn best_score(&self) -> i32 {
		self.score.values().copied().max().unwrap_or(0)
	}

	fn worst_score(&self) -> i32 {
		self.score.values().copied().min().unwrap_or(0)
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

	fn entity(&self, eid: EID) -> Option<&Entity> {
		self.world.entities.get(&eid)
	}

	// A copy of the current entity IDs.
	// (Copy allows mutation while iterating).
	fn entity_ids(&self) -> impl Iterator<Item = EID> {
		self.world.entities.keys().copied().collect::<SmallVec<_>>().into_iter()
	}

	fn player(&self, player_id: ID) -> &Player {
		&self.world.players[player_id]
	}

	fn player_ids(&self) -> impl Iterator<Item = ID> {
		self.world.players.copied_ids()
	}

	fn player_mut(&mut self, player_id: ID) -> &mut Player {
		&mut self.world.players[player_id]
	}
}

fn by_chance(probabilty: f32) -> bool {
	rand::thread_rng().gen::<f32>() < probabilty
}
