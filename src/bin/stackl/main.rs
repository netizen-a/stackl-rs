use std::process::ExitCode;
use std::{fs, path};

use clap::Parser;
use stackl::StacklFormat;

#[derive(Parser, Debug)]
struct Args {
    file: path::PathBuf,
}
fn main() -> ExitCode {
    let args = Args::parse();
    let content = fs::read(args.file).unwrap();
    let _stackl = StacklFormat::try_from(content.as_slice()).unwrap();
    ExitCode::SUCCESS
}
