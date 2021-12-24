use structopt::*;

/// Command-line options for game server.
#[derive(StructOpt)]
pub struct ServerOpts {
	/// TCP listen address
	#[structopt()]
	pub addr: String,

	/// Map files to cycle through
	#[structopt()]
	pub maplist: Vec<String>,
}
