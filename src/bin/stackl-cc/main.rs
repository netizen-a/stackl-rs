use std::process::ExitCode;

use cli::PreprocStdout;

mod cli;
mod lex;
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
	ExitCode::SUCCESS
}
