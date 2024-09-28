use std::process::ExitCode;
use std::sync::mpsc::channel;
use std::thread::scope;
use std::{fs, path};

use clap::Parser;
use stackl::StacklFormat;

mod chk;
mod flag;
mod mach;
mod msg;
mod ram;

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
    let (request_send, request_recv) = channel::<msg::MachineRequest>();
    let (response_send, response_recv) = channel::<msg::MachineResponse>();
    scope(|f| {
        f.spawn(|| {
            machine.run(request_send, response_recv);
        });
        f.spawn(|| {
            for recv in request_recv {
                println!("recv runtime: {recv:?}");
                if let Err(_) = response_send.send(msg::MachineResponse::Test) {
                    return;
                }
            }
        });
    });
    ExitCode::SUCCESS
}
