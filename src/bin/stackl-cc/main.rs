use std::{process::ExitCode, sync::mpsc, thread};

use cli::PreprocStdout;
use tok::Token;

mod ast;
mod cli;
mod lex;
mod sem;
mod syn;
mod tok;

fn main() -> ExitCode {
	let args = cli::Args::parse();
	let preproc = lex::preproc::Preprocessor::new(args.in_file, args.pp_stdout).unwrap();
	if args.pp_stdout != PreprocStdout::Disabled {
		for result in preproc {
			if let Err(error) = result {
				eprintln!("DEBUG: {:?}", error);
			}
		}
		return ExitCode::SUCCESS;
	}

	let (snd, rcv) = mpsc::channel::<Token>();
	let mut syntax_parser = syn::SyntaxParser::new(&rcv);
	let mut lex_errors = vec![];
	let mut syntax = Ok(ast::TranslationUnit::default());
	thread::scope(|s| {
		s.spawn(|| {
			for result in preproc {
				if let Ok(token) = result {
					snd.send(token).expect("failed to send token");
				} else if let Err(error) = result {
					lex_errors.push(error);
				}
			}
		});

		syntax = syntax_parser.parse();
	});

	for error in lex_errors.iter() {
		eprintln!("{:?}", error);
	}

	if !lex_errors.is_empty() {
		return ExitCode::FAILURE;
	}
	if syntax.is_ok() {
		println!("yay");
	}

	ExitCode::SUCCESS
}
