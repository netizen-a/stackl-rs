use std::process::ExitCode;

mod cli;
mod lex;

fn main() -> ExitCode {
    let args = cli::Args::parse();
    let mut preproc = lex::preproc::Preprocessor::new(args.in_file, args.pp_stdout);
    let _tokens = preproc.parse().unwrap();
    if args.pp_stdout > 0 {
        return ExitCode::SUCCESS;
    }
    for token in _tokens {
        println!("{token} :: {token:?}");
    }
    ExitCode::SUCCESS
}
