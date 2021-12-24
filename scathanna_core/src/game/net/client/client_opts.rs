use structopt::*;

/// Command-line options for game client.
#[derive(StructOpt)]
pub struct ClientOpts {
	/// Player name
	#[structopt(long, short)]
	pub name: String,

	/// Player avatar (0, 1, 2,... or "frog", "panda", ...)
	#[structopt(long, short, default_value = "frog")]
	pub avatar: String,

	/// Server address to connect to.
	#[structopt()]
	pub addr: String,
}
