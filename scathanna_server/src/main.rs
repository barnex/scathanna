use anyhow::*;
use scathanna_core::game::net::*;
use scathanna_core::game::*;
use structopt::*;

fn main() {
	NetServer::listen_and_serve(ServerOpts::from_args()).unwrap_or_else(exit);
}

fn exit(err: Error) {
	eprintln!("{}", err);
	std::process::exit(1);
}
