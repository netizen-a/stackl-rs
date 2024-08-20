// Copyright (c) 2024-2026 Jonathan A. Thomason

use clap::Parser;
use lalrpop_util::ErrorRecovery;
use lalrpop_util::lalrpop_mod;
use stackl::asm::ast::*;
use std::fs;
use std::path;
use std::process::ExitCode;

use crate::grammar::ProgramParser;
use tok::{
	LexicalError,
	Token,
};

mod cli;
mod code_gen;
mod error;
mod lex;
mod sym;
mod tok;

lalrpop_mod! {
	#[allow(clippy::ptr_arg)]
	grammar,
	"/bin/stackl-as/grammar.rs"
}

fn main() -> ExitCode {
	let args = cli::Args::parse();
	let source = match fs::read_to_string(&args.asmfile) {
		Ok(file) => file,
		Err(err) => {
			eprintln!(
				"stackl-as: fatal: can't open '{}' for reading: {}",
				args.asmfile.to_string_lossy(),
				err
			);
			return ExitCode::FAILURE;
		}
	};

	let mut ast = match parse_grammar(&source) {
		Ok(ast) => ast,
		Err(err) => {
			error::print_errors(&args.asmfile, err, &source);
			return ExitCode::FAILURE;
		}
	};

	sym::fixup_labels(&mut ast);
	sym::fixup_start(&mut ast);

	// TODO: add fixup_sections, which will combine section blocks

	let code = code_gen::ast_to_fmt2(ast).unwrap();
	let outfile = match args.outfile {
		Some(o) => o,
		None => {
			let outfile = args.asmfile.with_extension("stackl");
			let outfile = outfile.file_name().unwrap();
			path::PathBuf::from(outfile)
		}
	};
	fs::write(outfile, code.to_vec()).unwrap();

	ExitCode::SUCCESS
}

pub fn parse_grammar(
	input: &str,
) -> Result<Vec<Stmt>, Vec<ErrorRecovery<usize, Token, LexicalError>>> {
	let tokens = lex::Lexer::new(input);
	let mut errors = Vec::new();
	let mut ast = match ProgramParser::new().parse(&mut errors, tokens) {
		Ok(v) => v,
		Err(parse_error) => {
			errors.push(ErrorRecovery {
				error: parse_error,
				dropped_tokens: vec![],
			});
			return Err(errors);
		}
	};
	// prepend .text directive in case fixup rotates vector
	ast.insert(
		0,
		Stmt::new(Inst::Directive(
			Directive::Segment,
			vec![".text".to_string()],
		)),
	);
	if errors.is_empty() {
		Ok(ast)
	} else {
		Err(errors)
	}
}
