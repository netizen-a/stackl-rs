use crate::io;
use crate::machine::flag;
use crate::machine::MachineState;
use std::sync;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

/// Control/Status register
pub const GEN_IO_CSR: i32 = 0x0B000000;
/// Buffer address register
pub const GEN_IO_BUFF: i32 = 0x0B000004;
/// Buffer size register
pub const GEN_IO_SIZE: i32 = 0x0B000008;
/// Bunber of characters sent
pub const GEN_IO_COUNT: i32 = 0x0B00000C;

pub const GEN_IO_CSR_IE: i32 = 0x00010000; // Interrupt enable
pub const GEN_IO_CSR_INT: i32 = 0x00020000; // Interrupt occurred
pub const GEN_IO_CSR_DONE: i32 = 0x80000000u32 as i32; // The operation is complete
pub const GEN_IO_CSR_ERR: i32 = 0x40000000; // The operation resulted in an error

// Operations: occupy the lower 8 bits of the CSR register
pub const GEN_IO_OP_PRINTS: i32 = 1;
pub const GEN_IO_OP_PRINTC: i32 = 2;
pub const GEN_IO_OP_GETL: i32 = 3;
pub const GEN_IO_OP_GETI: i32 = 4;
pub const GEN_IO_OP_EXEC: i32 = 5;

pub fn run_device(machine_lock: &RwLock<MachineState>, state: &sync::Once) {
    while !state.is_completed() {
        thread::sleep(Duration::from_micros(100));
        let mut guard = machine_lock.write().unwrap();
        let _result = execute_operation(&mut guard);
        // do something with result
    }
}

fn execute_operation(cpu: &mut MachineState) -> Result<(), flag::MachineCheck> {
    let mut csr = cpu.load_abs_i32(GEN_IO_CSR).unwrap();
    // If done don't check anything else.
    if csr & GEN_IO_CSR_DONE != 0 {
        return Ok(());
    }
    let addr = cpu.load_abs_i32(GEN_IO_BUFF).unwrap();
    let size = cpu.load_abs_i32(GEN_IO_SIZE).unwrap();
    match csr & 0xFF {
        GEN_IO_OP_PRINTS => {
            let max_addr = (addr + size) as usize;
            let buf = cpu.mem.get((addr as usize)..max_addr)?;
            let count = io::try_print(buf);
            cpu.store_abs_i32(count as i32, GEN_IO_COUNT)?;
            csr |= GEN_IO_CSR_DONE;
        }
        GEN_IO_OP_PRINTC => todo!(),
        GEN_IO_OP_GETL => todo!(),
        GEN_IO_OP_GETI => todo!(),
        GEN_IO_OP_EXEC => todo!(),
        _ => {
            // report error
            return Err(flag::MachineCheck::ILLEGAL_INST);
        }
    }
    cpu.store_abs_i32(csr, 0x0B00_0000).unwrap();
    Ok(())
}
