use std::{process::ExitCode, sync::mpsc, thread};

use ast::ExternalDeclaration;
use cli::PreprocStdout;
use tok::Token;

mod ast;
mod cli;
mod diag;
mod lex;
mod sem;
mod syn;
mod tok;

fn main() -> ExitCode {
	let args = cli::Args::parse();
	let diagnostics = diag::DiagnosticEngine::new();
	let mut preproc = lex::preproc::Preprocessor::new(args.in_file, &diagnostics).unwrap();
	if args.pp_stdout != PreprocStdout::Disabled {
		let preproc_string = preproc.to_string(args.pp_stdout);
		print!("{}", preproc_string);
		return ExitCode::SUCCESS;
	}

	let (snd_tok, rcv_tok) = mpsc::channel::<Token>();
	let (snd_syn, _rcv_syn) = mpsc::channel::<ExternalDeclaration>();
	let syntax = syn::SyntaxParser::new(rcv_tok);
	thread::scope(|s| {
		s.spawn(|| {
			for token in preproc {
				snd_tok.send(token).expect("failed to send token");
			}
		});
		s.spawn(|| {
			for result in syntax {
				if let Ok(external_declaration) = result {
					snd_syn
						.send(external_declaration)
						.expect("failed to send external-declaration");
				} else if let Err(error) = result {
					diagnostics.push_syn(error)
				}
			}
		});
	});

	ExitCode::SUCCESS
}
