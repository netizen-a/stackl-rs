use std::{process::ExitCode, sync::mpsc, thread};

use cli::PreprocStdout;
use tok::Token;

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

	let (snd, rcv) = mpsc::channel::<tok::Result<Token>>();
	thread::scope(|s| {
		s.spawn(|| {
			for result in preproc {
				snd.send(result).expect("failed to send token");
			}
		});
		s.spawn(|| {
			// syntax/semantics
		});
	});

	ExitCode::SUCCESS
}
