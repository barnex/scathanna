use super::internal::*;

use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread;

/// BufReader for reading file with more descriptive message on error.
pub fn open(file: &Path) -> Result<impl Read> {
	Ok(BufReader::new(File::open(file).map_err(|err| error(format!("open {:?}: {}", file, err)))?))
}

/// BufWriter for writing file with more descriptive message on error.
pub fn create(file: &Path) -> Result<impl Write> {
	Ok(BufWriter::new(File::create(file).map_err(|err| error(format!("open {:?}: {}", file, err)))?))
}

/// Attempt to prefix `file` with the executable's path.
/// Intended to find asset files relative to the executable. E.g.:
///
///   assets/maps =>  /path/to/binary/assets/maps
///
/// Return file unchanged if this fails (will still work when cd'ed in binary's directory).
/// Return the file unchanged if it is already absolute.
pub fn abs_path(file: &Path) -> PathBuf {
	if file.is_absolute() || file.exists() {
		return file.to_owned();
	}

	if let Ok(wd) = std::env::current_dir() {
		let abs = wd.join(file);
		if abs.exists() {
			return abs;
		}
	}

	match std::env::current_exe() {
		Err(e) => {
			eprintln!("ERROR getting executable path: {}.\nPlease run this command from the directory containing `assets/`", e);
			file.to_owned()
		}
		Ok(exe) => match exe.parent() {
			None => file.to_owned(),
			Some(dir) => dir.join(file),
		},
	}
}

/// Spawn a thread that reads lines from stdin,
/// pipes them through a channel.
/// Used for non-blocking stdin reads.
pub fn pipe_stdin() -> StdinPipe {
	let (send, recv) = std::sync::mpsc::sync_channel(0);
	thread::spawn(move || {
		let stdin = std::io::stdin();
		let mut stdin = std::io::Stdin::lock(&stdin);
		loop {
			let mut input = String::new();
			match stdin.read_line(&mut input) {
				Err(e) => {
					eprintln!("{}", e);
					return;
				}
				Ok(0) => return, // end of steam
				Ok(_) => (),
			};
			match send.send(input) {
				Ok(_) => (),
				Err(_) => return,
			}
		}
	});
	StdinPipe(recv)
}

pub struct StdinPipe(Receiver<String>);

impl StdinPipe {
	/// Try read a line from stdin, non-blocking.
	/// Returns None if no input is currently available.
	pub fn try_read(&mut self) -> Option<String> {
		match self.0.try_recv() {
			Err(TryRecvError::Empty) => None,
			Err(TryRecvError::Disconnected) => None,
			Ok(cmd) => {
				let cmd = cmd.trim().to_owned();
				if cmd.len() != 0 {
					Some(cmd)
				} else {
					None
				}
			}
		}
	}
}
