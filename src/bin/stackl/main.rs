use std::process::ExitCode;
use std::{fs, path};

use clap::Parser;
use stackl::StacklFormat;

mod mach;
mod ram;
mod chk;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: path::PathBuf,
    #[arg(long, default_value_t = false)]
    trace: bool,
}
fn main() -> ExitCode {
    let args = Args::parse();
    let content = fs::read(args.file).unwrap();
    let data = StacklFormat::try_from(content.as_slice()).unwrap();
    let mut machine = mach::MachineState::new(1000);
    machine.ram.store_slice(&data.text, 0).unwrap();
    let sp_addr = if data.text.len() % 2 != 0 {
        data.text.len() + 2 - (data.text.len() % 2)
    } else {
        data.text.len()
    };
    machine.set_sp(sp_addr.try_into().unwrap());
    machine.set_trace(args.trace);
    machine.run();
    ExitCode::SUCCESS
}
