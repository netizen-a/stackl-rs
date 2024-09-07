use std::fs;
use std::path;
use std::process::ExitCode;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    asmfile: path::PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();
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

    let ast = match stackl::parse_grammar(&source) {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("{:?}", err);
            return ExitCode::FAILURE;
        }
    };

    let _symtab = stackl::sym::build_symtab(&ast);

    ExitCode::SUCCESS
}
