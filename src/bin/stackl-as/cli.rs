// Copyright (c) 2024-2026 Jonathan Thomason

use std::path;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
	pub asmfile: path::PathBuf,
	#[arg(short)]
	pub outfile: Option<path::PathBuf>,
}
