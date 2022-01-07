use super::internal::*;

/// Networked game server.
///
/// The server and clients all have a game state in memory
/// which they continuously synchronize.
///
/// The world data is structured so that each part
/// may only be mutated by exactly one party:
///
///   * Only clients can update their own Player.
///   * Only the server can update the rest of the game state.
///
/// This guarantees eventual consistency, as no two parties will
/// every attempt to update the same part of the world.
///
/// In case a party wants to update world which it does not own
/// it needs to send a message requesting the change from the owner.
///
/// E.g.: the server cannot directly update a Player's location,
/// because that is owned by the clients. If the server wishes to
/// respawn a player, it must send a message requesting so.
pub struct NetServer {
	clients: HashMap<ID, ClientConn>,   // Maps player Entity ID to net pipe
	send_events: Sender<ServerEvent>,   // All server events are sent to (clone of) this channel
	recv_events: Receiver<ServerEvent>, // All server events are received here

	state: ServerState,
}

type ClientConn = NetSender<ServerMsg>;

// Events handled by serve_loop.
enum ServerEvent {
	Conn(TcpStream),                // A client has connected
	Drop(ID),                       // A client has dropped
	ClientMessage((ID, ClientMsg)), // Client sent a message
	Tick(f32),                      // Internal clock tick
}

impl NetServer {
	/// Serve incoming connections on `opts.addr`.
	/// Only returns in case of error.
	pub fn listen_and_serve(opts: ServerOpts) -> Result<()> {
		let (clients_send, server_recv) = channel::<ServerEvent>();
		Self::spawn_listen_loop(&opts.addr, clients_send.clone())?;
		Self::spawn_ticker(clients_send.clone());

		let mut server = Self {
			send_events: clients_send,
			recv_events: server_recv,
			clients: HashMap::default(),
			state: ServerState::new(opts)?,
		};

		server.serve_loop()
	}

	//____________________________________________________________ server event handling

	// Run the "manager task", who exclusively controls the shared state
	// (game state + client connections) via message passing.
	//
	// This provides an ordering for the incoming events/requests.
	//
	// TODO: return Result<!> when in stable Rust.
	fn serve_loop(&mut self) -> Result<()> {
		use ServerEvent::*;
		loop {
			match self.recv_events.recv()? {
				Conn(netpipe) => self.handle_conn_client(netpipe),
				Drop(id) => self.handle_drop_client(id),
				ClientMessage((id, msg)) => self.handle_client_msg(id, msg),
				Tick(dt) => self.handle_tick(dt),
			}
		}
	}

	// Handle a new client connection.
	fn handle_conn_client(&mut self, tcp_stream: TcpStream) {
		if let Err(e) = self.handle_conn_with_result(tcp_stream) {
			println!("server: handle_conn: error: {}", e)
		}
	}

	// add new player to the game, send them the full state.
	fn handle_conn_with_result(&mut self, mut tcp_stream: TcpStream) -> Result<()> {
		// receive player attributes (name, etc) from client
		let join_msg: JoinMsg = wireformat::deserialize_from(&mut tcp_stream)?;

		// add player to server game state.
		let player_id = self.state.join_new_player(join_msg);

		// send "accepted" message with map info and player ID
		//serialize_into(&mut BufWriter::new(&mut tcp_stream), &accepted)?;

		// make a new client connection under the player ID.
		// forward client messages to server event loop.
		let send = ClientConn::new(tcp_stream.try_clone().expect("clone TCP stream"));
		assert!(!self.clients.contains_key(&player_id));
		self.clients.insert(player_id, send);
		self.spawn_recv_loop(tcp_stream, player_id);

		// announce the new player to others.
		self.flush_pending_diffs();

		Ok(())
	}

	// Handle a dropped connection event.
	fn handle_drop_client(&mut self, player_id: ID) {
		self.clients.remove(&player_id);
		self.state.handle_drop_player(player_id);
		println!("dropped client #{}, {} left", player_id, self.clients.len());
		self.flush_pending_diffs();
	}

	// Handle an incoming message from a client.
	fn handle_client_msg(&mut self, player_id: ID, msg: ClientMsg) {
		self.state.handle_client_msg(player_id, msg);
		self.flush_pending_diffs();
	}

	fn handle_tick(&mut self, dt: f32) {
		self.state.handle_tick(dt);
		self.flush_pending_diffs();
	}

	//____________________________________________________________ communication protocol

	fn flush_pending_diffs(&mut self) {
		let client_ids = self.clients.keys().copied().collect::<SmallVec<_>>();

		let diffs = mem::take(&mut self.state.pending_diffs); // sending to disconnected client caused drop which might lead to new diffs.
		for msg in diffs {
			for client_id in Self::addressees(&client_ids, msg.to) {
				self.send_to(client_id, msg.msg.clone())
			}
		}
	}

	// expand Addressee (Just/Not/All) into list of matching client IDs.
	fn addressees(clients: &[ID], a: Addressee) -> SmallVec<ID> {
		match a {
			Addressee::Just(id) => smallvec![id],
			Addressee::Not(id) => clients.into_iter().copied().filter(|&i| i != id).collect(),
			Addressee::All => clients.iter().copied().collect(),
		}
	}

	// send a message to just one player
	fn send_to(&mut self, player_id: ID, msg: ServerMsg) {
		if let Some(client) = self.clients.get_mut(&player_id) {
			match client.send(msg) {
				Err(e) => {
					println!("{}", e);
					self.handle_drop_client(player_id)
				}
				Ok(()) => (),
			}
		}
	}

	//____________________________________________________________ async workers

	// Spawn a loop that continuously decodes client messages from the network,
	// send them to the central server event queue `events`.
	fn spawn_recv_loop(&mut self, tcp_stream: TcpStream, player_id: ID) {
		let mut stream = BufReader::new(tcp_stream);
		let send = self.send_events.clone();
		thread::spawn(move || loop {
			let msg = wireformat::deserialize_from(&mut stream);
			match msg {
				Err(e) => {
					eprintln!("server: recv from {}: {}", player_id, e);
					send.send(ServerEvent::Drop(player_id)).unwrap();
					return;
				}
				Ok(msg) => send.send(ServerEvent::ClientMessage((player_id, msg))).unwrap(),
			}
		});
	}

	// Spawn a loop that accepts incoming connections,
	// sends the server a `ServerEvent::Conn` event for each accepted connection.
	fn spawn_listen_loop(address: &str, clients_send: Sender<ServerEvent>) -> Result<()> {
		let listener = TcpListener::bind(address)?;
		println!("listening on {}", listener.local_addr().unwrap());
		thread::spawn(move || {
			for stream in listener.incoming() {
				match stream {
					Err(e) => eprintln!("{}", e), // client failed to connect, server carries on.
					Ok(tcp_stream) => {
						println!("connected to {}", tcp_stream.peer_addr().unwrap());
						if clients_send.send(ServerEvent::Conn(tcp_stream)).is_err() {
							return; // server quit, so stop worker thread.
						}
					}
				}
			}
		});
		Ok(())
	}

	fn spawn_ticker(clients_send: Sender<ServerEvent>) {
		thread::spawn(move || {
			let period = Duration::from_millis(100);
			let dt = period.as_secs_f32();
			loop {
				std::thread::sleep(period);
				if clients_send.send(ServerEvent::Tick(dt)).is_err() {
					return; // server quit, so stop worker thread.
				}
			}
		});
	}
}
