use std::process::ExitCode;
use std::sync::mpsc::channel;
use std::thread::scope;
use std::{fs, path};

use chk::{CheckKind, MachineCheck};
use clap::Parser;
use stackl::StacklFormat;

mod chk;
mod flag;
mod mach;
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
    let mut machine = mach::MachineState::new(data.int_vec, args.memory);
    let _ = ram::VM_RAM
        .write()
        .and_then(|mut ram| {
            ram.resize(args.memory, 0x79);
            ram.store_slice(&data.text, 0).unwrap();
            Ok(())
        })
        .expect("Failed to initialize VM's RAM");
    let _ = ram::VM_ROM
        .write()
        .and_then(|mut rom| {
            rom.resize(64, 0);
            for slot in 0..15 {
                rom.store_i32(0x0001, 4 * slot).unwrap();
            }
            if data.trap_vec != -1 {
                rom.store_i32(data.trap_vec, 4).unwrap();
            }
            Ok(())
        })
        .expect("Failed to initialize VM's ROM");
    let sp_addr = if data.text.len() % 2 != 0 {
        data.text.len() + 2 - (data.text.len() % 2)
    } else {
        data.text.len()
    };
    machine.sp = sp_addr.try_into().unwrap();
    machine.set_trace(args.trace);
    let (request_send, request_recv) = channel::<i32>();
    let (response_send, response_recv) = channel::<Result<(), chk::MachineCheck>>();
    scope(|f| {
        f.spawn(|| {
            machine.run(request_send, response_recv);
        });
        f.spawn(|| {
            for offset in request_recv {
                let response = process_request(offset);
                if let Err(_) = response_send.send(response) {
                    return;
                }
            }
        });
    });
    ExitCode::SUCCESS
}

fn process_request(offset: i32) -> Result<(), MachineCheck> {
    let memory = ram::VM_RAM.read().unwrap();
    let op = memory.load_i32(offset)?;
    let param1 = memory.load_i32(offset + 4)?;
    let _param2 = memory.load_i32(offset + 8)?;
    match op {
        3 => memory.print(param1),
        _ => Err(MachineCheck::from(CheckKind::IllegalOp)),
    }
}
