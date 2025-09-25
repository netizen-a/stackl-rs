// warnings are unhelpful when debugging hard errors
#![allow(warnings)]

mod analysis;
mod data_types;
mod diagnostics;
mod symtab;
mod synthesis;

use clap::Parser;
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser, Debug)]
#[command(version, about = "Stackl C99 compiler", long_about = None)]
pub struct Args {
	#[arg(name = "FILE", required = true)]
	pub in_file: PathBuf,
	#[arg(long = "output", short = 'o')]
	pub out_file: Option<PathBuf>,
	#[arg(long, default_value_t = false)]
	pub pp_stdout_comments: bool,
	#[arg(long, default_value_t = false)]
	pub pp_stdout_tokens: bool,
	#[arg(long = "trace", default_value_t = false)]
	pub is_traced: bool,
}

fn main() -> ExitCode {
	// let args = cli::Args::parse();
	let args = Args::parse();
	let mut diag_engine = diagnostics::DiagnosticEngine::new();
	let _analysis_result = analysis::parse(args.in_file, &mut diag_engine, args.is_traced);

	if diag_engine.contains_error() {
		diag_engine.print_diagnostics();
		return ExitCode::FAILURE;
	}

	//synthesis::parse(&analysis_result.unwrap());
	if args.is_traced {
		println!("{:#?}", _analysis_result);
	}

	ExitCode::SUCCESS
}
