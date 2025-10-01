// warnings are unhelpful when debugging hard errors
#![allow(warnings)]

mod analysis;
mod data_types;
mod diagnostics;
mod symtab;
mod synthesis;

use clap::Parser;
use std::cell;
use std::collections::HashMap;
use std::io::IsTerminal;
use std::io::Read;
use std::{fs, rc};
use std::{path::PathBuf, process::ExitCode};

use analysis::{lex, sem, syn, tok};

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
	#[arg(short = 'E', long, default_value_t = false)]
	pub pp_stdout_tokens: bool,
	#[arg(long = "trace", default_value_t = false)]
	pub is_traced: bool,
	#[arg(long, default_value_t = EnableColor::Auto)]
	pub enable_color: EnableColor,
	#[arg(short = 'W', default_value_t = WarnLevel::Minimal)]
	pub warn_lvl: WarnLevel,
}

fn main() -> ExitCode {
	let args = Args::parse();
	let enable_color = match args.enable_color {
		EnableColor::Auto => std::io::stdout().is_terminal(),
		EnableColor::Always => true,
		EnableColor::Never => false,
	};

	let mut diag_engine = diagnostics::DiagnosticEngine::new(enable_color);

	diag_engine.insert_file_info(0, &args.in_file);
	let mut file = fs::File::open(&args.in_file).unwrap();
	let mut text = String::new();
	file.read_to_string(&mut text).unwrap();
	let lexer = lex::lexer::Lexer::new(text, 0);
	let pp_iter = lex::PPTokenIter::new(lexer, diag_engine.get_file_map());
	let tokens: Vec<tok::TokenTriple> = lex::TokensParser::new(&mut diag_engine, pp_iter).parse();

	diag_engine.print_diagnostics();
	if args.pp_stdout_tokens {
		println!("{:#?}", tokens);
		return ExitCode::SUCCESS;
	}

	if diag_engine.contains_error() {
		return ExitCode::FAILURE;
	}

	let tk_iter = syn::TokenIter::from(tokens.into_boxed_slice());
	let unit = match syn::SyntaxParser::new().parse(&mut diag_engine, tk_iter) {
		Ok(unit) => unit,
		Err(error) => {
			diag_engine.push_syntax_error(error);
			vec![]
		}
	};

	if args.is_traced {
		println!("{:#?}", unit);
	}

	let _analysis_result = sem::SemanticParser::new(&mut diag_engine, &args).parse(unit);

	diag_engine.print_diagnostics();
	if diag_engine.contains_error() {
		return ExitCode::FAILURE;
	}

	//synthesis::parse(&analysis_result.unwrap());

	ExitCode::SUCCESS
}
