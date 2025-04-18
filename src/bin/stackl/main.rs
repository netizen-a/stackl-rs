use std::process::ExitCode;
use std::str::FromStr;
use std::sync::mpsc::{channel, Sender};
use std::sync::RwLock;
use std::time::Duration;
use std::{fs, io, path, sync, thread, time};

use clap::Parser;
use machine::flag::{IntVec, MachineCheck, Status};
use machine::MachineState;
use request::Request;
use stackl::{StacklFlags, StacklFormatV1, StacklFormatV2};

mod machine;
mod request;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: path::PathBuf,
    #[arg(
        long,
        default_value_t = false,
        help = "Write an instruction trace to stderr"
    )]
    trace: bool,
    #[arg(
        short,
        long,
        default_value_t = 500000,
        help = "Set the memory size for the virtual machine"
    )]
    memory: usize,
    #[arg(long, default_value_t = 0, help = "Instruction delay in milliseconds")]
    mdelay: u64,
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Enable the INP instruction"
    )]
    inp: bool,
    #[arg(
        short = 'G',
        long,
        default_value_t = false,
        help = "Enable the General IO device"
    )]
    gen_io: bool,
}
fn main() -> ExitCode {
    let args = Args::parse();
    let content = match fs::read(&args.file) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Failed to load `{}`:{:?}", args.file.display(), err);
            return ExitCode::FAILURE;
        }
    };
    let mut data = match StacklFormatV2::try_from(content.as_slice()) {
        Ok(data) => data,
        Err(stackl::ErrorKind::InvalidMagic) => {
            let fmt1 = StacklFormatV1::try_from(content.as_slice()).unwrap();
            fmt1.try_into().unwrap()
        }
        Err(kind) => match kind {
            stackl::ErrorKind::InvalidVersion { expected, found } => {
                eprintln!("Error: Expected header version {expected}, but found {found}");
                return ExitCode::FAILURE;
            }
            err => panic!("failed to load: {:?}", err),
        },
    };

    if args.inp {
        // force INP to be enabled regardless of binary
        data.flags.set(StacklFlags::FEATURE_INP, true);
    }

    if args.gen_io {
        // force gen_io to be enabled regardless of binary
        data.flags.set(StacklFlags::FEATURE_GEN_IO, true);
    }

    // copy to local variable to handle threading later.
    let flags = data.flags;

    let mut machine = MachineState::new(args.memory);
    machine.store_program(data, true, -1).unwrap();
    machine.set_trace(args.trace);
    let machine = RwLock::new(machine);

    // prevent move semantics on closures with `move`.
    let machine = &machine;

    let (request_send, request_recv) = channel::<Request>();
    thread::scope(|f| {
        static RUNNING_STATE: sync::Once = sync::Once::new();
        f.spawn(|| {
            RUNNING_STATE.call_once(|| {
                run_machine(machine, request_send, args.mdelay);
            });
        });
        if flags.contains(StacklFlags::FEATURE_INP) {
            f.spawn(|| {
                for request in request_recv {
                    let result = request::process_request(machine, &request);
                    let mut write_lock = machine.write().unwrap();
                    let mut val: u32 = 0x80000000;
                    if result.is_ok() {
                        write_lock.store_i32(val as i32, request.offset).unwrap();
                    } else {
                        val |= 0x40000000;
                        write_lock.store_i32(val as i32, request.offset).unwrap();
                    }
                }
            });
        }
        if flags.contains(StacklFlags::FEATURE_GEN_IO) {
            f.spawn(|| {
                while !RUNNING_STATE.is_completed() {
                    thread::sleep(Duration::from_micros(100));
                    run_gen_io(machine);
                }
            });
        }
        if flags.contains(StacklFlags::FEATURE_PIO_TERM) {
            f.spawn(|| {
                while !RUNNING_STATE.is_completed() {
                    thread::sleep(Duration::from_micros(100));
                }
            });
        }
    });
    ExitCode::SUCCESS
}

pub fn run_machine(
    machine_lock: &RwLock<MachineState>,
    request_send: Sender<Request>,
    delay_step: u64,
) {
    loop {
        if delay_step != 0 {
            thread::sleep(time::Duration::from_millis(delay_step));
        }
        let mut cpu = machine_lock.write().unwrap();
        if cpu.flag.get_status(Status::HALTED) {
            return;
        }
        if let Err(check) = machine::step::next_opcode(&mut cpu, &request_send) {
            if cpu.ivec == 0 && cpu.load_abs_i32(0).unwrap() == -1 {
                // Default machine check
                eprintln!("Machine Check: {check} at {}", cpu.ip);
                return;
            } else {
                cpu.flag.check.set(check, true);
                cpu.flag.intvec.set(IntVec::MACHINE_CHECK, true);
                cpu.interrupt(false).unwrap();
            }
        }
    }
}

pub fn run_gen_io(machine_lock: &RwLock<MachineState>) {
    let mut machine_lock = machine_lock.write().unwrap();
    let mut csr = machine_lock.load_abs_i32(0x0B00_0000).unwrap();
    if csr & machine::memory::gen_io::GEN_IO_CSR_DONE != 0 {
        return;
    }
    let buff = machine_lock.load_abs_i32(0x0B00_0004).unwrap();
    let size = machine_lock.load_abs_i32(0x0B00_0008).unwrap();
    match csr & 0xFF {
        0 => {
            // do nothing.
        }
        // GEN_IO_OP_PRINTS
        1 => {
            let count = machine_lock.print(buff, size as usize).unwrap();
            machine_lock
                .store_abs_i32(count as i32, 0x0B00_000C)
                .unwrap();
            csr |= 0x8000_0000u32 as i32;
        }
        csr => todo!("gen_io: {csr}"),
    }
    machine_lock.store_abs_i32(csr, 0x0B00_0000).unwrap();
}
