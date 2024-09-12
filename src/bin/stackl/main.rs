use std::process::ExitCode;
use std::{fs, path};

use clap::Parser;
use stackl::StacklFormat;

mod mach;
mod op;

#[derive(Parser, Debug)]
struct Args {
    file: path::PathBuf,
}
fn main() -> ExitCode {
    let args = Args::parse();
    let content = fs::read(args.file).unwrap();
    let data = StacklFormat::try_from(content.as_slice()).unwrap();
    let machine = mach::MachineState::new(500000);
    machine.store(&data.text, 0);
    machine.execute();
    ExitCode::SUCCESS
}
