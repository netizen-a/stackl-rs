// Copyright (c) 2024-2026 Jonathan A. Thomason
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
	pub file: PathBuf,
	#[arg(
		long,
		default_value_t = false,
		help = "Write an instruction trace to stderr"
	)]
	pub trace: bool,
	#[arg(
		short,
		long,
		default_value_t = 500000,
		help = "Set the memory size for the virtual machine"
	)]
	pub memory: usize,
	#[arg(
		short,
		long,
		default_value_t = false,
		help = "Enable the INP instruction"
	)]
	pub inp: bool,
	#[arg(
		short = 'G',
		long,
		default_value_t = false,
		help = "Enable the General IO device"
	)]
	pub gen_io: bool,
	// TODO: implement processor delay
	#[arg(
		long,
		default_value_t = 33.0,
		help = "Set the processor speed in megahertz"
	)]
	pub mhz: f32,
	#[arg(short = 'g', long, default_value_t = false, help = "Run in debug mode")]
	pub debug: bool,
}
