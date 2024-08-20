// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::io;
use crate::machine::MachineState;
use crate::machine::flag;
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

const INTERRUPT_MASK: i32 = GEN_IO_CSR_IE | GEN_IO_CSR_DONE;

pub fn run_device(machine_lock: &RwLock<MachineState>, state: &sync::Once) {
	while !state.is_completed() {
		thread::sleep(Duration::from_micros(100));
		if execute_operation(machine_lock).is_err() {
			let mut cpu = machine_lock.write().unwrap();
			let mut csr = cpu.load_abs_i32(GEN_IO_CSR).unwrap();
			csr |= GEN_IO_CSR_ERR;
			cpu.store_i32(csr, GEN_IO_CSR).unwrap();
		}
	}
}

fn execute_operation(machine_lock: &RwLock<MachineState>) -> Result<(), flag::MachineCheck> {
	let cpu = machine_lock.read().unwrap();
	let mut csr = cpu.load_abs_i32(GEN_IO_CSR).unwrap();
	// If done don't check anything else.
	if csr & GEN_IO_CSR_DONE != 0 {
		return Ok(());
	}
	let addr = cpu.load_abs_i32(GEN_IO_BUFF).unwrap();
	let size = cpu.load_abs_i32(GEN_IO_SIZE).unwrap();
	drop(cpu);
	match csr & 0xFF {
		GEN_IO_OP_PRINTS => {
			let cpu = machine_lock.read().unwrap();
			let max_addr = (addr + size) as usize;
			let buf = cpu.mem.get((addr as usize)..max_addr)?;
			let count = io::try_print(buf);
			drop(cpu);
			let mut cpu = machine_lock.write().unwrap();
			cpu.store_abs_i32(count as i32, GEN_IO_COUNT)?;
			csr |= GEN_IO_CSR_DONE;
			cpu.store_abs_i32(csr, 0x0B00_0000).unwrap();
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
	if (csr & INTERRUPT_MASK) == INTERRUPT_MASK {
		let mut cpu = machine_lock.write().unwrap();
		csr |= GEN_IO_CSR_INT;
		cpu.store_abs_i32(csr, GEN_IO_CSR)?;
		cpu.flag.set_intvec(flag::IntVec::GEN_IO, true);
	}
	Ok(())
}
