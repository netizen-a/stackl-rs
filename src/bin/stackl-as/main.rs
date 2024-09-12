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

    let mut ast = match stackl::ast::parse_grammar(&source) {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("{:?}", err);
            return ExitCode::FAILURE;
        }
    };

    stackl::ast::fixup_labels(&mut ast);
    stackl::ast::fixup_start(&mut ast);

    let code = stackl::StacklFormat::from(ast);
    let outfile = args.asmfile.with_extension("stackl");
    let outfile = outfile.file_name().unwrap();
    fs::write(outfile, code.to_vec()).unwrap();

    ExitCode::SUCCESS
}
