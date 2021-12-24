use super::internal::*;

/// Networked game client.
///
/// The NetClient controls a GLClient, which renders a world locally.
///
/// The NetClient handles local keyboard/mouse input,
/// which it passes through to the GLClient (e.g. moving the player)
/// while also messaging the Server about the corresponding changes to the local Player.
/// (e.g.: "the player has moved")
///
/// The NetClient also receives messages from the Server
/// to update the local world (e.g. the positions of other players, etc).
pub struct NetClient {
	server_conn: NetPipe<ClientMsg, ServerMsg>,
	gl_client: GLClient,
}


impl NetClient {
	/// Create a client connected to a game server, as specified by command-line options.
	pub fn connect(opts: ClientOpts) -> Result<Self> {
		println!("connecting to {}...", &opts.addr);
		let mut tcp_stream = TcpStream::connect(&opts.addr)?;
		println!("connection accepted, joining...");

		let avatar_id = parse_avatar_id(&opts.avatar)?;

		serialize_into(&mut tcp_stream, &JoinMsg { name: opts.name, avatar_id })?;
		tcp_stream.flush()?;

		let accepted_msg: ServerMsg = deserialize_from(&mut tcp_stream)?;
		let (map_name, player_id, players) = match accepted_msg {
			ServerMsg::SwitchMap { map_name, player_id, players } => (map_name, player_id, players),
			_ => return err("expected initial server message to be SwitchMap"),
		};
		println!("accepted as {}", player_id);

		let world = World::with_players(&map_name, players)?;

		Ok(Self {
			gl_client: GLClient::new(world, player_id)?,
			server_conn: NetPipe::new(tcp_stream),
		})
	}

	fn tick(&mut self) -> Result<()> {
		self.apply_messages()?;
		let diff = self.gl_client.tick_and_diff();
		self.send_updates(diff)
	}

	/// Apply updates received from server.
	fn apply_messages(&mut self) -> Result<()> {
		while let Some(result) = self.server_conn.try_recv() {
			let msg = result?;
			self.gl_client.state_mut().apply_server_msg(msg);
		}
		Ok(())
	}

	/// Apply updates received from server.
	fn send_updates(&mut self, diff: Vec<ClientMsg>) -> Result<()> {
		for msg in diff {
			self.server_conn.send(msg)?;
		}
		Ok(())
	}
}

impl EventHandler for NetClient {
	/// Handle keyboard input.
	fn on_key(&mut self, k: Key, pressed: bool) {
		self.gl_client.on_key(k, pressed)
	}

	/// Handle mouse input.
	fn on_mouse_move(&mut self, x: f64, y: f64) {
		self.gl_client.on_mouse_move(x, y)
	}

	fn draw(&mut self, width: u32, height: u32) {
		self.gl_client.draw(width, height)
	}

	fn tick(&mut self) {
		self.tick().expect("Client::tick error"); // TODO: how to handle?
	}
}
