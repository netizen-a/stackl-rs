use crate::machine::MachineState;
use std::sync::RwLock;

pub const GEN_IO_CSR: i32 = 0x0B000000; // Control/Status register
pub const GEN_IO_BUFF: i32 = 0x0B000004; // Buffer address register
pub const GEN_IO_SIZE: i32 = 0x0B000008; // Buffer size register
pub const GEN_IO_COUNT: i32 = 0x0B00000C; // Number of characters sent

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

pub fn run_gen_io(machine_lock: &RwLock<MachineState>) {
    let mut machine_lock = machine_lock.write().unwrap();
    let mut csr = machine_lock.load_abs_i32(0x0B00_0000).unwrap();
    if csr & GEN_IO_CSR_DONE != 0 {
        return;
    }
    let buff = machine_lock.load_abs_i32(0x0B00_0004).unwrap();
    let size = machine_lock.load_abs_i32(0x0B00_0008).unwrap();
    match csr & 0xFF {
        0 => {
            // do nothing.
        }
        GEN_IO_OP_PRINTS => {
            let count = machine_lock.print(buff, size as usize).unwrap();
            machine_lock
                .store_abs_i32(count as i32, 0x0B00_000C)
                .unwrap();
            csr |= 0x8000_0000u32 as i32;
        }
        GEN_IO_OP_PRINTC => {}
        GEN_IO_OP_GETL => {}
        GEN_IO_OP_GETI => {}
        GEN_IO_OP_EXEC => {}
        csr => todo!("gen_io: {csr}"),
    }
    machine_lock.store_abs_i32(csr, 0x0B00_0000).unwrap();
}
