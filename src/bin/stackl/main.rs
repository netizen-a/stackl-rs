use std::process::ExitCode;
use std::{fs, path};

use clap::Parser;
use stackl::StacklFormat;

mod mach;
mod ram;

#[derive(Parser, Debug)]
struct Args {
    file: path::PathBuf,
}
fn main() -> ExitCode {
    let args = Args::parse();
    let content = fs::read(args.file).unwrap();
    let data = StacklFormat::try_from(content.as_slice()).unwrap();
    // println!("{:?}", data.text);
    // println!("text.len() = {}", data.text.len());
    let mut machine = mach::MachineState::new(1000);
    machine.ram.store_slice(&data.text, 0);
    let sp_addr = if data.text.len() % 2 != 0 {
        data.text.len() + 2 - (data.text.len() % 2)
    } else {
        data.text.len()
    };
    machine.set_sp(sp_addr.try_into().unwrap());
    machine.run();
    ExitCode::SUCCESS
}
