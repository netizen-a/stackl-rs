use std::process::ExitCode;
use std::sync::mpsc::{channel, Sender};
use std::sync::RwLock;
use std::{fs, path, sync, thread};

use clap::Parser;
use machine::flag::{IntVec, Status};
use machine::MachineState;
use stackl::{StacklFlags, StacklFormatV1, StacklFormatV2};

mod device;
mod io;
mod machine;

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
    let machine_lock = &RwLock::new(machine);

    let (request_send, request_recv) = channel::<device::inp::Request>();
    thread::scope(|f| {
        static RUNNING_STATE: sync::Once = sync::Once::new();
        f.spawn(|| {
            RUNNING_STATE.call_once(|| {
                run_machine(machine_lock, request_send);
            });
        });
        if flags.contains(StacklFlags::FEATURE_INP) {
            f.spawn(|| {
                for request in request_recv {
                    let result = device::inp::process_request(machine_lock, &request);
                    let mut write_lock = machine_lock.write().unwrap();
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
                device::gen_io::run_device(machine_lock, &RUNNING_STATE);
            });
        }
        if flags.contains(StacklFlags::FEATURE_PIO_TERM) {
            f.spawn(|| {
                device::pio_term::run_device(machine_lock, &RUNNING_STATE);
            });
        }
        if flags.contains(StacklFlags::FEATURE_DISK) {
            f.spawn(|| {
                device::disk::run_device(machine_lock, &RUNNING_STATE);
            });
        }
        if flags.contains(StacklFlags::FEATURE_DMA_TERM) {
            f.spawn(|| {
                device::dma_term::run_device(machine_lock, &RUNNING_STATE);
            });
        }
    });
    ExitCode::SUCCESS
}

pub fn run_machine(
    machine_lock: &RwLock<MachineState>,
    request_send: Sender<device::inp::Request>,
) {
    loop {
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
