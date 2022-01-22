use super::errormessage::*;
use super::wireformat::*;

use anyhow::anyhow;
use anyhow::Result;

use serde::de::DeserializeOwned;
use serde::Serialize;

use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread::spawn;

/// A typed, bi-directional, non-blocking network conduit.
/// Sends messages of type `S`, receives messages of type `R`.
pub struct NetPipe<S, R> {
	send: NetSender<S>,
	recv: NetReceiver<R>,
}

impl<S, R> NetPipe<S, R>
where
	S: Serialize + DeserializeOwned + Send + 'static,
	R: Serialize + DeserializeOwned + Send + 'static,
{
	pub fn new(tcp_stream: TcpStream) -> Self {
		Self {
			send: NetSender::new(tcp_stream.try_clone().expect("clone TCP stream")),
			recv: NetReceiver::new(tcp_stream.try_clone().expect("clone TCP stream")),
		}
	}

	pub fn send(&mut self, msg: S) -> Result<()> {
		self.send.send(msg)
	}

	pub fn try_recv(&mut self) -> Option<Result<R>> {
		self.recv.try_recv()
	}
}

/// A typed, non-blocking network conduit.
/// Sends messages of type `T`.
pub struct NetSender<T>(Sender<T>);
/// A typed, non-blocking network conduit.
/// Receives messages of type `T`.
pub struct NetReceiver<T>(Receiver<PipeResult<T>>);

impl<T> NetSender<T>
where
	T: Serialize + DeserializeOwned + Send + 'static,
{
	pub fn new(tcp_stream: TcpStream) -> Self {
		tcp_stream.set_nodelay(true).expect("set TCP no delay");
		let (send, worker_recv) = channel();
		start_upload(tcp_stream, worker_recv);
		Self(send)
	}

	/// Attempts to send a Message.
	pub fn send(&mut self, msg: T) -> Result<()> {
		self.0.send(msg).map_err(|err| anyhow!("netpipe: {}", err))
	}
}

impl<T> NetReceiver<T>
where
	T: Serialize + DeserializeOwned + Send + 'static,
{
	pub fn new(tcp_stream: TcpStream) -> Self {
		tcp_stream.set_nodelay(true).expect("set TCP no delay");
		let (worker_send, recv) = channel();
		start_download(tcp_stream, worker_send);
		Self(recv)
	}

	pub fn try_recv(&mut self) -> Option<Result<T>> {
		match self.0.try_recv() {
			Ok(result) => match result {
				Ok(msg) => Some(Ok(msg)),
				Err(e) => Some(Err(Box::new(e).into())),
			},
			Err(TryRecvError::Empty) => None,
			Err(e) => Some(Err(Box::new(e).into())),
		}
	}
}

// Spawn a loop taking messages from `worker_recv` and serializing them to `tcp_stream`.
//
// The loop aborts on error, causing the next `NetPipe::recv` call to error out.
// I.e.: errors are to be handled on receive, not send.
// (Because even a successful send would not guarantee that the server successfully
// processed the message, it may have crashed right after receiving.
// Only a server response guarantees success.)
fn start_upload<T>(tcp_stream: TcpStream, worker_recv: Receiver<T>)
where
	T: Serialize + Send + 'static,
{
	let mut buf = BufWriter::new(tcp_stream.try_clone().expect("clone TCP stream"));

	spawn(move || {
		if let Err(_e) = || -> Result<()> {
			// wrapper so we can use ? operator
			loop {
				// wait for the first message,
				// then consume all further pending messages, if any,
				// flush them all together.
				let msg = worker_recv.recv()?;
				serialize_into(&mut buf, &msg)?;

				for msg in worker_recv.try_iter() {
					serialize_into(&mut buf, &msg)?;
				}
				buf.flush()?;
			}
		}() {
			let _ = tcp_stream.shutdown(Shutdown::Both);
			return;
		}
	});
}

// Spawn a loop deserializing messages from `tcp_stream` and sending them to `worker_send`.
// Drops `worker_send` on error, causing future `recv` calls to error out.
fn start_download<T>(tcp_stream: TcpStream, worker_send: Sender<PipeResult<T>>)
where
	T: DeserializeOwned + Send + 'static,
{
	let mut buf = BufReader::new(tcp_stream.try_clone().expect("clone TCP steam"));
	spawn(move || loop {
		//match bincode::deserialize_from(&mut buf) {
		match deserialize_from(&mut buf) {
			Ok(msg) => {
				if let Err(_e) = worker_send.send(Ok(msg)) {
					let _ = tcp_stream.shutdown(Shutdown::Both);
					return;
				}
			}
			Err(_e) => {
				let _ = tcp_stream.shutdown(Shutdown::Both);
				return;
			}
		}
	});
}

// Sendable result.
type PipeResult<T> = std::result::Result<T, ErrorMessage>;
