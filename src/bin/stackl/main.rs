use std::process::ExitCode;
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::RwLock;
use std::thread::{self, scope};
use std::{fs, io, path, time};

use chk::{CheckKind, MachineCheck};
use clap::Parser;
use flag::Status;
use machine::MachineState;
use stackl::StacklFormat;

mod chk;
mod flag;
mod machine;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: path::PathBuf,
    #[arg(long, default_value_t = false)]
    trace: bool,
    #[arg(short, long, default_value_t = 500000)]
    memory: usize,
    #[arg(long, default_value_t = 0, help = "Instruction delay in milliseconds")]
    mdelay: u64,
}
fn main() -> ExitCode {
    let args = Args::parse();
    let content = fs::read(args.file).unwrap();
    let data = StacklFormat::try_from(content.as_slice()).unwrap();
    let mut machine = MachineState::new(data, args.memory);
    machine.set_trace(args.trace);
    let machine = RwLock::new(machine);

    let (request_send, request_recv) = channel::<i32>();
    let (_, response_recv) = channel::<Result<(), chk::MachineCheck>>();
    scope(|f| {
        f.spawn(|| {
            run_machine(&machine, request_send, response_recv, args.mdelay);
        });
        f.spawn(|| {
            for offset in request_recv {
                let result = process_request(&machine, offset);
                let mut write_lock = machine.write().unwrap();
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

pub fn run_machine(
    mach: &RwLock<MachineState>,
    request_send: Sender<i32>,
    response_recv: Receiver<Result<(), chk::MachineCheck>>,
    delay_step: u64,
) {
    loop {
        if delay_step != 0 {
            thread::sleep(time::Duration::from_millis(delay_step));
        }
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
        if let Err(check) = machine::step::next_opcode(&mut write_lock, &request_send) {
            eprintln!("{check}");
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
            read_lock.print(param1)
        }
        INP_GETS_CALL => {
            drop(read_lock);
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            let mut write_lock = machine.write().unwrap();
            write_lock.store_slice(buf.as_bytes(), param1)
        }
        INP_GETL_CALL => {
            drop(read_lock);
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            buf.truncate(255);
            buf.push('\0');
            let mut write_lock = machine.write().unwrap();
            write_lock.store_slice(buf.as_bytes(), param1)
        }
        INP_GETI_CALL => {
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
            let c_str = read_lock.load_cstr(param1)?;
            let Ok(filepath) = c_str.to_str() else {
                return Err(chk::MachineCheck::from(chk::CheckKind::Other));
            };
            let content = fs::read(filepath).expect("Failed to open file");
            drop(read_lock);

            let program = StacklFormat::try_from(content.as_slice()).unwrap();
            let mut machine_lock = machine.write().unwrap();
            let bp = machine_lock.bp;
            let lp = machine_lock.lp;
            machine_lock.store_slice(&program.text, bp)?;
            let result = machine_lock.store_i32(program.stack_size, lp + 4);
            // eprintln!("{:?}", result);
            result
        }
        _ => {
            Err(chk::MachineCheck::from(CheckKind::IllegalOp))
        }
    }
}
