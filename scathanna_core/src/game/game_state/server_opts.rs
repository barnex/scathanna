use structopt::*;

/// Command-line options for game server.
#[derive(StructOpt)]
pub struct ServerOpts {
	#[structopt(short, long, default_value = "dm")]
	pub game_type: String,

	//#[structopt(long)]
	//pub enable_levelling: bool,
	/// TCP listen address
	#[structopt()]
	pub addr: String,

	/// Map files to cycle through
	#[structopt()]
	pub maplist: Vec<String>,
}
