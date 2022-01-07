use super::internal::*;

type NetPipe = super::super::netpipe::NetPipe<ClientMsg, ServerMsg>;

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
	engine: Rc<Engine>,
	server_conn: NetPipe,
	gl_client: GLClient,
	error: Option<String>,
	stdin: StdinPipe,
}

impl NetClient {
	pub fn connect(engine: Rc<Engine>, config: &Config) -> Result<Self> {
		let (server_conn, gl_client) = Self::connect_with_result(engine.clone(), config)?;
		Ok(Self {
			engine,
			server_conn,
			gl_client,
			error: None,
			stdin: pipe_stdin(),
		})
	}

	/// Create a client connected to a game server, as specified by command-line options.
	pub fn connect_with_result(engine: Rc<Engine>, opts: &Config) -> Result<(NetPipe, GLClient)> {
		println!("connecting to {}...", &opts.server);
		let mut tcp_stream = TcpStream::connect(&opts.server)?;
		println!("connection accepted, joining...");

		let avatar_id = parse_avatar_id(&opts.avatar)?;
		let team = if opts.team.is_empty() { Team::random() } else { Team::from_str(&opts.team)? };

		serialize_into(
			&mut tcp_stream,
			&JoinMsg {
				name: opts.name.clone(),
				avatar_id,
				team,
			},
		)?;
		tcp_stream.flush()?;

		let accepted_msg: ServerMsg = deserialize_from(&mut tcp_stream)?;
		let (map_name, player_id, players, entities) = match accepted_msg {
			ServerMsg::SwitchMap {
				map_name,
				player_id,
				players,
				entities,
			} => (map_name, player_id, players, entities),
			_ => return err("expected initial server message to be SwitchMap"),
		};
		println!("accepted as #{}", player_id);

		Ok((NetPipe::new(tcp_stream), Self::join_map(engine.clone(), map_name, player_id, players, entities)?))
	}

	fn join_map(engine: Rc<Engine>, map_name: String, player_id: ID, players: Players, entities: Entities) -> Result<GLClient> {
		let world = World::from_map(&map_name, players, entities)?;
		GLClient::new(engine, world, player_id)
	}

	/// Handle keyboard input.
	fn on_key(&mut self, k: Key, pressed: bool) {
		self.gl_client.on_key(k, pressed)
	}

	/// Handle mouse input.
	fn on_mouse_move(&mut self, x: f64, y: f64) {
		self.gl_client.on_mouse_move(x, y)
	}

	fn draw_(&self, width: u32, height: u32) {
		if let Some(err) = &self.error {
			self.draw_message(width, height, err);
		} else {
			self.gl_client.draw(width, height);
		}
	}

	// Draw an on-screen message.
	// TODO: should be handled by HUD.
	fn draw_message(&self, width: u32, height: u32, msg: &str) {
		let camera = Camera::new(vec3::ZERO); // TODO: should work without this.
		self.engine.set_camera((width, height), &camera);
		self.engine.clear(0.1, 0.1, 0.3);
		self.engine.print_center(RED, msg);
	}

	fn tick(&mut self) {
		if self.error.is_some() {
			return;
		}
		let result = self.tick_with_result();
		if let Err(e) = result {
			self.error = Some(e.to_string());
		}
	}

	fn tick_with_result(&mut self) -> Result<()> {
		self.handle_stdin()?;
		self.apply_messages()?;
		let diff = self.gl_client.tick_and_diff();
		self.send_updates(diff)?;
		Ok(())
	}

	fn handle_stdin(&mut self) -> Result<()> {
		if let Some(cmd) = self.stdin.try_read() {
			self.send_updates(vec![ClientMsg::Command(cmd)])?;
		}
		Ok(())
	}

	/// Apply updates received from server.
	fn apply_messages(&mut self) -> Result<()> {
		while let Some(result) = self.server_conn.try_recv() {
			let msg = result?;

			match msg {
				ServerMsg::SwitchMap {
					map_name,
					player_id,
					players,
					entities,
				} => self.gl_client = Self::join_map(self.engine.clone(), map_name, player_id, players, entities)?,
				msg => self.gl_client.state_mut().apply_server_msg(msg),
			}
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
		self.on_key(k, pressed)
	}

	/// Handle mouse input.
	fn on_mouse_move(&mut self, x: f64, y: f64) {
		self.on_mouse_move(x, y)
	}

	fn draw(&mut self, width: u32, height: u32) {
		self.draw_(width, height)
	}

	fn tick(&mut self) {
		self.tick()
	}
}
