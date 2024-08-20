// Copyright (c) 2024-2026 Jonathan A. Thomason

// warnings are unhelpful when debugging hard errors
#![allow(warnings)]

mod analysis;
mod cli;
mod data_type;
mod diagnostics;
mod symtab;
mod synthesis;

use clap::Parser;
use diagnostics as diag;
use std::io::IsTerminal;
use std::process::ExitCode;
use std::time;
use std::time::Duration;

use analysis::{
	lex,
	sema,
	syn,
	tok,
};

use synthesis::icg;

fn main() -> ExitCode {
	let args = cli::Args::parse();
	let enable_color = match args.enable_color {
		cli::EnableColor::Auto => std::io::stdout().is_terminal(),
		cli::EnableColor::Always => true,
		cli::EnableColor::Never => false,
	};

	let mut diag_engine = diag::DiagnosticEngine::new(enable_color);

	let Ok(text) = diag_engine.insert_file_info(0, &args.in_file) else {
		let diag = diag::Diagnostic::fatal(diag::DiagKind::FileNotFound(args.in_file), None);
		diag_engine.push(diag);
		diag_engine.print_once();
		return ExitCode::FAILURE;
	};

	let mut since_array = vec![];
	// start preprocessor timer
	let timer = time::Instant::now();
	let lexer = lex::lexer::Lexer::new(text.to_string(), 0);
	let pp_iter = lex::PPTokenIter::new(lexer, diag_engine.get_file_map());
	let tokens: Vec<tok::TokenTriple> = lex::TokensParser::new(
		&mut diag_engine,
		pp_iter,
		args.stdout_preproc,
		args.warn_lvl.clone(),
	)
	.parse();

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
	let mut unit = match syn::SyntaxParser::new().parse(&mut diag_engine, &args.opt_lvl, tk_iter) {
		Ok(unit) => unit,
		Err(error) => {
			diag_engine.push_syntax_error(error);
			vec![]
		}
	};

	let duration = time::Instant::now().duration_since(timer);
	since_array.push((duration, "syntax parser time"));

	let timer = time::Instant::now();
	let mut semantic_parser = sema::SemanticParser::new(&mut diag_engine, &args);
	let maybe_unit = semantic_parser.parse(unit);

	let duration = time::Instant::now().duration_since(timer);
	since_array.push((duration, "semantic parser time"));

	let has_error = semantic_parser.contains_error();
	semantic_parser.print_errors();
	let tree = semantic_parser.build_tree();
	let layouts = semantic_parser.data_layouts.take().unwrap();
	drop(semantic_parser);

	if args.check || has_error {
		if args.ast {
			ptree::print_tree(&tree);
		}
		if args.is_timed {
			print_time(since_array);
		}
		return if has_error {
			ExitCode::FAILURE
		} else {
			ExitCode::SUCCESS
		};
	}

	let Some(unit) = maybe_unit else {
		return ExitCode::FAILURE;
	};
	let codegen_context = icg::IrContext { layouts, unit };
	let _ssa_module =
		match icg::SSACodeGen::new(&mut diag_engine, args.is_traced).build(codegen_context) {
			Ok(inner) => inner,
			Err(fatal) => diag_engine.push_and_exit(fatal),
		};
	diag_engine.print_once();
	if args.ast {
		ptree::print_tree(&tree);
	}
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
