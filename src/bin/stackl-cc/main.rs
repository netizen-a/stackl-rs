use std::process::ExitCode;

mod cli;
mod lex;
mod tok;

fn main() -> ExitCode {
	let args = cli::Args::parse();
	let preproc = lex::preproc::Preprocessor::new(args.in_file, args.pp_stdout).unwrap();
	if args.pp_stdout > 0 {
		for _ in preproc {}
		return ExitCode::SUCCESS;
	}
	ExitCode::SUCCESS
}
