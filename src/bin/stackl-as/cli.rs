// Copyright (c) 2024-2026 Jonathan A. Thomason

use clap::Parser;
use std::path;

#[derive(Parser, Debug)]
pub struct Args {
	pub asmfile: path::PathBuf,
	#[arg(short)]
	pub outfile: Option<path::PathBuf>,
}
