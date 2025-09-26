// warnings are unhelpful when debugging hard errors
#![allow(warnings)]

mod analysis;
mod data_types;
mod diagnostics;
mod symtab;
mod synthesis;

use clap::Parser;
use std::{path::PathBuf, process::ExitCode};
use std::io::IsTerminal;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum EnableColor {
	Auto,
	Always,
	Never,
}

impl ToString for EnableColor {
	fn to_string(&self) -> String {
		match self {
			Self::Auto => String::from("auto"),
			Self::Always => String::from("always"),
			Self::Never => String::from("never"),
		}
	}
}

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
	#[arg(long, default_value_t = EnableColor::Auto)]
	pub enable_color: EnableColor,
}

fn main() -> ExitCode {
	let args = Args::parse();
	let enable_color = match args.enable_color {
		EnableColor::Auto => std::io::stdout().is_terminal(),
		EnableColor::Always => true,
		EnableColor::Never => false,
	};

	let mut diag_engine = diagnostics::DiagnosticEngine::new(enable_color);
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
