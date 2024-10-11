use std::process::ExitCode;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::RwLock;
use std::thread::scope;
use std::{ffi, fs, io, path};
use std::str::FromStr;

use chk::{CheckKind, MachineCheck};
use clap::Parser;
use flag::Status;
use mach::MachineState;
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
    let machine = RwLock::new(mach::MachineState::new(data.int_vec, args.memory));
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
    {
        let mut write_lock = machine.write().unwrap();
        write_lock.ram.resize(args.memory, 0x79);
        write_lock.ram.store_slice(&data.text, 0).unwrap();
        write_lock.sp = sp_addr.try_into().unwrap();
        write_lock.set_trace(args.trace);
    }
    let (request_send, request_recv) = channel::<i32>();
    let (_, response_recv) = channel::<Result<(), chk::MachineCheck>>();
    scope(|f| {
        f.spawn(|| {
            run_machine(&machine, request_send, response_recv);
        });
        f.spawn(|| {
            for offset in request_recv {
                eprintln!("\nrequest: offset: {offset}\n");
                let result = process_request(&machine, offset);
                let mut write_lock = machine.write().unwrap();
                let mut val: u32 = 0x80000000;
                if result.is_ok() {
                    eprintln!("\nrequest: ok\n");
                    write_lock.store_i32(val as i32, offset).unwrap();
                } else {
                    eprintln!("\nrequest: err\n");
                    eprintln!("\nrequest: err offset: {offset}\n");
                    val |= 0x40000000;
                    write_lock.store_i32(val as i32, offset).unwrap();
                }
            }
        });
    });
    ExitCode::SUCCESS
}

pub fn run_machine(
    mach: &RwLock<mach::MachineState>,
    request_send: Sender<i32>,
    response_recv: Receiver<Result<(), chk::MachineCheck>>,
) {
    loop {
        let mut _mach_check = None;
        for recv in response_recv.try_iter() {
            if let Err(check) = recv {
                _mach_check = Some(check);
                return;
            }
        }
        let mut write_lock = mach.write().unwrap();
        if write_lock.flag.get_status(Status::HALTED) {
            return;
        }
        if let Err(check) = mach::execute_op(&mut *write_lock, &request_send) {
            eprintln!("{check}");
            eprintln!(
                "{:08x} {:6} {:6} {:6} {:6} {:6}",
                write_lock.flag.as_u32(),
                write_lock.bp,
                write_lock.lp,
                write_lock.ip,
                write_lock.sp,
                write_lock.fp
            );
            return;
        }
    }
}

fn process_request(machine: &RwLock<MachineState>, offset: i32) -> Result<(), MachineCheck> {
    const INP_PRINTS_CALL: i32 = 3;
    const INP_GETS_CALL: i32 = 5;
    const INP_GETL_CALL: i32 = 6;
    const INP_GETI_CALL: i32 = 7;
    const INP_EXEC_CALL: i32 = 8;
    let read_lock = machine.read().unwrap();
    let op = read_lock.load_i32(offset)?;
    let param1 = read_lock.load_i32(offset + 4)?;
    let _param2 = read_lock.load_i32(offset + 8)?;
    match op {
        INP_PRINTS_CALL => {
            eprintln!("DEBUG: INP_PRINTS_CALL");
            read_lock.ram.print(param1)
        }
        INP_GETS_CALL => {
            eprintln!("DEBUG: INP_GETS_CALL");
            drop(read_lock);
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            let mut write_lock = machine.write().unwrap();
            write_lock.ram.store_slice(buf.as_bytes(), param1)
        }
        INP_GETL_CALL => {
            eprintln!("DEBUG: INP_GETL_CALL");
            drop(read_lock);
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            buf.truncate(255);
            buf.push('\0');
            let mut write_lock = machine.write().unwrap();
            write_lock.ram.store_slice(buf.as_bytes(), param1)
        }
        INP_GETI_CALL => {
            eprintln!("DEBUG: INP_GETI_CALL");
            drop(read_lock);
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            let Ok(deci) = i32::from_str(buf.trim()) else {
                return Err(chk::MachineCheck::from(chk::CheckKind::Other));
            };
            let mut write_lock = machine.write().unwrap();
            write_lock.store_i32(deci, param1)
        }
        INP_EXEC_CALL => {
            eprintln!("DEBUG: INP_EXEC_CALL");
            let bytes = read_lock.ram.load_slice(param1)?;
            let Ok(c_str) = ffi::CStr::from_bytes_until_nul(bytes) else {
                return Err(chk::MachineCheck::from(chk::CheckKind::Other))
            };
            let Ok(filepath) = c_str.to_str() else {
                return Err(chk::MachineCheck::from(chk::CheckKind::Other))
            };
            let content = fs::read(filepath).expect("Failed to open file");
            drop(read_lock);

            let program = StacklFormat::try_from(content.as_slice()).unwrap();
            let mut machine_lock = machine.write().unwrap();
            let bp = machine_lock.bp;
            let lp = machine_lock.lp;
            machine_lock.ram.store_slice(&program.text, bp)?;
            let result = machine_lock.store_i32(program.stack_size, lp + 4);
            eprintln!("{:?}", result);
            result
        }
        _ => {
            eprintln!("DEBUG: Unknown option");
            Err(chk::MachineCheck::from(CheckKind::IllegalOp))
        }
    }
}
