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
}

/// Subsequent messages sent by Client after the initial JoinMsg.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMsg {
	/// This is my location, orientation, ... (sent continuously).
	MovePlayer(Frame),

	/// I'm ready to (re-)spawn
	ReadyToSpawn,

	/// Spawn a visual effect.
	AddEffect { effect: Effect },

	/// I have shot player with ID `victim`.
	HitPlayer { victim: ID },
}

/// Subsequent messages sent by Server after the initial AcceptedMsg.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMsg {
	AddPlayer(Player),
	DropPlayer { player_id: ID },
	SwitchMap { map_name: String, players: Players, player_id: ID },
	RequestRespawn(SpawnPoint),
	// Update a player's position, orientation, velocity (source of truth = client).
	MovePlayer { player_id: ID, frame: Frame },
	// Update everything not controlled by `MovePlayer` (source of truth = server).
	UpdatePlayer(Player),
	AddEffect(Effect),
	LogMessage(String),
	HUDMessage(String),
}
