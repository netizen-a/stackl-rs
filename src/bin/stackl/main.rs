use std::process::ExitCode;
use std::sync::mpsc::channel;
use std::thread::scope;
use std::{fs, io, path};

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
    ram::VM_RAM
        .write()
        .map(|mut ram| {
            ram.resize(args.memory, 0x79);
            ram.store_slice(&data.text, 0).unwrap();
        })
        .expect("Failed to initialize VM's RAM");
    ram::VM_ROM
        .write()
        .map(|mut rom| {
            rom.resize(64, 0);
            for slot in 0..15 {
                rom.store_i32(0x0001, 4 * slot).unwrap();
            }
            if data.trap_vec != -1 {
                rom.store_i32(data.trap_vec, 4).unwrap();
            }
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
    let (_, response_recv) = channel::<Result<(), chk::MachineCheck>>();
    scope(|f| {
        f.spawn(|| {
            machine.run(request_send, response_recv);
        });
        f.spawn(|| {
            for offset in request_recv {
                let result = process_request(offset);
                let mut write_lock = ram::VM_RAM.write().unwrap();
                let mut val: u32 = 0x80000000;
                if result.is_ok() {
                    write_lock.store_i32(val as i32, offset).unwrap();
                } else {
                    val |= 0x40000000;
                    write_lock.store_i32(val as i32, offset).unwrap();
                }
            }
        });
    });
    ExitCode::SUCCESS
}

fn process_request(offset: i32) -> Result<(), MachineCheck> {
    let read_lock = ram::VM_RAM.read().unwrap();
    let op = read_lock.load_i32(offset)?;
    let param1 = read_lock.load_i32(offset + 4)?;
    let _param2 = read_lock.load_i32(offset + 8)?;
    match op {
        3 => read_lock.print(param1),
        5 => {
            drop(read_lock);
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            let mut write_lock = ram::VM_RAM.write().unwrap();
            write_lock.store_slice(buf.as_bytes(), param1)
        }
        _ => Err(chk::MachineCheck::from(CheckKind::IllegalOp)),
    }
}
