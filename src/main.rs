//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate hex_literal;

mod chain_spec;
mod service;
mod cli;

pub use substrate_cli::{VersionInfo, IntoExit, error};

fn run() -> cli::error::Result<()> {
	let version = VersionInfo {
		name: "Substrate Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "erc20-substrate-bridge",
		author: "Akropolis",
		description: "erc20-substrate-bridge",
		support_url: "support@akropolis.io",
	};
	cli::run(::std::env::args(), cli::Exit, version)
}

error_chain::quick_main!(run);
