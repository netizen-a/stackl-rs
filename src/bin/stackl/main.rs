use std::process::ExitCode;
use std::{fs, path};

use clap::Parser;
use stackl::StacklFormat;

mod chk;
mod mach;
mod ram;
mod flag;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: path::PathBuf,
    #[arg(long, default_value_t = false)]
    trace: bool,
    #[arg(short, long, default_value_t = 500000)]
    memory: usize,
}
fn main() -> ExitCode {
    let args = Args::parse();
    let content = fs::read(args.file).unwrap();
    let data = StacklFormat::try_from(content.as_slice()).unwrap();
    let mut machine = mach::MachineState::new(args.memory);
    machine.ram.store_slice(&data.text, 0).unwrap();
    let sp_addr = if data.text.len() % 2 != 0 {
        data.text.len() + 2 - (data.text.len() % 2)
    } else {
        data.text.len()
    };
    machine.sp = sp_addr.try_into().unwrap();
    machine.set_trace(args.trace);
    machine.run();
    ExitCode::SUCCESS
}
