// warnings are unhelpful when debugging hard errors
#![allow(warnings)]

mod analysis;
mod data_types;
mod diagnostics;
mod symtab;
mod synthesis;

use clap::Parser;
use diagnostics::*;
use std::cell;
use std::collections::HashMap;
use std::io::IsTerminal;
use std::io::Read;
use std::thread::sleep;
use std::time;
use std::time::Duration;
use std::{fs, rc};
use std::{path::PathBuf, process::ExitCode};

use analysis::{lex, sem, syn, tok};

use crate::diagnostics::ToSpan;

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

#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq, Eq)]
enum WarnLevel {
	All,
	Minimal,
	None,
}

impl ToString for WarnLevel {
	fn to_string(&self) -> String {
		match self {
			Self::All => String::from("all"),
			Self::Minimal => String::from("minimal"),
			Self::None => String::from("none"),
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
	#[arg(
		short = 'E',
		group = "early-exit",
		group = "stdout",
		help = "Preprocess only; do not compile, assemble or link"
	)]
	pub stdout_preproc: bool,
	#[arg(long = "trace", group = "stdout")]
	pub is_traced: bool,
	#[arg(long, default_value_t = EnableColor::Auto)]
	pub enable_color: EnableColor,
	#[arg(short = 'W', default_value_t = WarnLevel::Minimal)]
	pub warn_lvl: WarnLevel,
	#[arg(
		short = 'S',
		group = "early-exit",
		help = "Compile only; do not assemble or link"
	)]
	pub check: bool,
	#[arg(
		long = "time",
		group = "stdout",
		help = "Time the execution of each subprocess"
	)]
	pub is_timed: bool,
}

fn main() -> ExitCode {
	let args = Args::parse();
	let enable_color = match args.enable_color {
		EnableColor::Auto => std::io::stdout().is_terminal(),
		EnableColor::Always => true,
		EnableColor::Never => false,
	};

	let mut diag_engine = DiagnosticEngine::new(enable_color);

	let Ok(text) = diag_engine.insert_file_info(0, &args.in_file) else {
		let diag = Diagnostic::fatal(DiagKind::FileNotFound(args.in_file), None);
		diag_engine.push(diag);
		diag_engine.print_once();
		return ExitCode::FAILURE;
	};

	let mut since_array = vec![];
	// start preprocessor timer
	let timer = time::Instant::now();
	let lexer = lex::lexer::Lexer::new(text.to_string(), 0);
	let pp_iter = lex::PPTokenIter::new(lexer, diag_engine.get_file_map());
	let tokens: Vec<tok::TokenTriple> =
		lex::TokensParser::new(&mut diag_engine, pp_iter, args.stdout_preproc).parse();

	let duration = time::Instant::now().duration_since(timer);
	since_array.push((duration, "preprocessor time"));

	if let Some(last_token) = tokens.last().map(|t| &t.1) {
		diag_engine.set_eof_span(last_token);
	}

	let has_error = diag_engine.contains_error();
	diag_engine.print_once();
	if has_error || args.stdout_preproc {
		if args.is_timed {
			print_time(since_array);
		}
		return match has_error {
			true => ExitCode::FAILURE,
			false => ExitCode::SUCCESS,
		};
	}

	let timer = time::Instant::now();
	let tk_iter = syn::TokenIter::from(tokens.into_boxed_slice());
	let unit = match syn::SyntaxParser::new().parse(&mut diag_engine, tk_iter) {
		Ok(unit) => unit,
		Err(error) => {
			diag_engine.push_syntax_error(error);
			vec![]
		}
	};

	let duration = time::Instant::now().duration_since(timer);
	since_array.push((duration, "syntax parser time"));

	let timer = time::Instant::now();
	let _analysis_result = sem::SemanticParser::new(&mut diag_engine, &args).parse(unit);

	let duration = time::Instant::now().duration_since(timer);
	since_array.push((duration, "semantic parser time"));

	let has_error = diag_engine.contains_error();
	diag_engine.print_once();
	if has_error || args.check {
		if args.is_timed {
			print_time(since_array);
		}
		return match has_error {
			true => ExitCode::FAILURE,
			false => ExitCode::SUCCESS,
		};
	}

	//synthesis::parse(&analysis_result.unwrap());

	if args.is_timed {
		print_time(since_array);
	}

	ExitCode::SUCCESS
}

fn print_time(since_array: Vec<(Duration, &str)>) {
	for (duration, name) in since_array {
		let secs = duration.as_secs();
		let millis = duration.as_millis();
		let micros = duration.as_micros();
		if secs > 0 {
			println!("{name}: {secs}.{}s", millis % 1000);
		} else if millis > 0 {
			println!("{name}: {millis}.{}ms", micros % 1000);
		} else {
			println!("{name}: {}μs", micros);
		}
	}
}
