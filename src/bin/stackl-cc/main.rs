use std::process::ExitCode;

mod cli;
mod lex;

use crate::lex::preproc::ParseError;

fn main() -> ExitCode {
    let args = cli::Args::parse();
    let mut preproc = lex::preproc::Preprocessor::new(args.in_file, args.pp_stdout);
    let result = preproc.parse();
    if args.pp_stdout > 0 {
        return ExitCode::SUCCESS;
    }
    match result {
        Ok(tokens) => {
            for token in tokens {
                println!("{token} :: {token:?}");
            }
        }
        Err(error) => match error {
            ParseError::IOError(io_error) => eprintln!("io error: {io_error}"),
            ParseError::LexicalErrors(error_list) => {
                for err in error_list {
                    eprintln!("{err:?}");
                }
            }
        },
    }
    ExitCode::SUCCESS
}
