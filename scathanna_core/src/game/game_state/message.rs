///! Client-Server messaging protocol.
use super::internal::*;

/// Message addressed to a (sub)set of all players.
pub struct Envelope<T> {
	pub to: Addressee,
	pub msg: T,
}

/// Who to send a message to: Just one player, All but one player, All players.
#[derive(Copy, Clone)]
pub enum Addressee {
	Just(ID),
	Not(ID),
	All,
}

pub type ClientMsgs = Vec<ClientMsg>;
pub type ServerMsgs = Vec<Envelope<ServerMsg>>;

/// Initial message sent by client when first joining a server.
#[derive(Serialize, Deserialize, Debug)]
pub struct JoinMsg {
	pub name: String, // Player's nickname
	pub avatar_id: u8,
	pub team: Team,
}

/// Subsequent messages sent by Client after the initial JoinMsg.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMsg {
	/// This is my location, orientation, ... (sent continuously).
	MovePlayer(Frame),

	/// I'm ready to (re-)spawn
	ReadyToSpawn,

	/// Spawn a visual effect.
	AddEffect(Effect),

	/// Start a sound effect.
	PlaySound(SoundEffect),

	/// I have shot player with ID `victim`.
	HitPlayer(ID),

	/// Send a CLI command to the server.
	Command(String),
}

/// Messages sent by Server.
#[derive(Serialize, Deserialize, Clone)]
pub enum ServerMsg {
	AddPlayer(Player),
	DropPlayer(ID),
	SwitchMap { map_name: String, players: Players, player_id: ID, entities: Entities },
	ForceMovePlayer(vec3),
	RequestRespawn(SpawnPoint),
	// Update a player's position, orientation, velocity (source of truth = client).
	MovePlayer(ID, Frame),
	// Update everything not controlled by `MovePlayer` (source of truth = server).
	UpdatePlayer(Player),
	UpdateEntity(Entity),
	RemoveEntity(EID),
	AddEffect(Effect),
	PlaySound(SoundEffect),
	UpdateHUD(HUDUpdate),
}

impl ServerMsg {
	pub fn to_all(self) -> Envelope<Self> {
		self.to(Addressee::All)
	}

	pub fn to_just(self, player_id: ID) -> Envelope<Self> {
		self.to(Addressee::Just(player_id))
	}

	pub fn to_not(self, player_id: ID) -> Envelope<Self> {
		self.to(Addressee::Not(player_id))
	}

	pub fn to(self, to: Addressee) -> Envelope<Self> {
		Envelope { to, msg: self }
	}
}
