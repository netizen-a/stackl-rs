use std::{fs, process::ExitCode};

mod cli;
mod lex;

fn main() -> ExitCode {
    let args = cli::Args::parse();
    let mut preproc =
        lex::preproc::Preprocessor::new(args.in_file, args.stdout_pp, args.include_comments);
    let _tokens = preproc.parse().unwrap();
    if args.stdout_pp {
        return ExitCode::SUCCESS;
    }
    ExitCode::SUCCESS
}
