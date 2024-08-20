// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::ffi;
use std::sync::mpsc::Sender;

use flag::{
	MachineCheck,
	MachineFlags,
	MetaFlags,
	Status,
};
use memory::MachineMemory;
use stackl::{
	StacklFlags,
	StacklFormatV2,
	asm::op,
};

pub mod flag;
mod interrupt;
pub mod memory;
pub mod step;
mod timer;
mod trace;

#[derive(Debug)]
pub struct MachineState {
	/// base pointer
	pub bp: i32,
	/// limit pointer
	pub lp: i32,
	/// instruction pointer
	pub ip: i32,
	/// stack pointer
	pub sp: i32,
	/// frame pointer
	pub fp: i32,
	pub flag: MachineFlags,
	pub ivec: i32,
	/// page table base register
	pub ptbr: i32,
	pub mem: MachineMemory,
	pub meta: MetaFlags,
	pub last_trace: u8,
}

impl MachineState {
	pub fn new(mem_size: usize) -> MachineState {
		MachineState {
			bp: 0,
			lp: mem_size as i32,
			ip: 8,
			sp: 0,
			fp: 0,
			flag: MachineFlags::new(),
			ivec: 0,
			ptbr: 0,
			mem: MachineMemory::new(mem_size),
			meta: MetaFlags::empty(),
			last_trace: 0,
		}
	}
	pub fn store_program(
		&mut self,
		program: StacklFormatV2,
		boot: bool,
		bp: i32,
	) -> Result<(), MachineCheck> {
		let text_len = program.text.len();
		if boot {
			let sp_addr = if text_len % 4 != 0 {
				text_len + 4 - (text_len % 4)
			} else {
				text_len
			};
			let mut meta = MetaFlags::empty();
			if program.flags.contains(StacklFlags::FEATURE_GEN_IO) {
				meta.set(MetaFlags::FEATURE_GEN_IO, true);
			}
			if program.flags.contains(StacklFlags::FEATURE_PIO_TERM) {
				meta.set(MetaFlags::FEATURE_PIO_TERM, true);
			}
			if program.flags.contains(StacklFlags::FEATURE_DMA_TERM) {
				meta.set(MetaFlags::FEATURE_DMA_TERM, true);
			}
			if program.flags.contains(StacklFlags::FEATURE_DISK) {
				meta.set(MetaFlags::FEATURE_DISK, true);
			}
			if program.flags.contains(StacklFlags::FEATURE_INP) {
				meta.set(MetaFlags::FEATURE_INP, true);
			}
			self.meta = meta;
			self.sp = (sp_addr + 4) as i32;
			self.fp = self.sp;
		}

		let addr = if bp < 0 { self.bp } else { bp };

		// put the stack size just above the text segment
		self.store_i32(program.stack_size, addr + text_len as i32)?;

		// copy text segment to memory
		let offset = addr as usize;
		self.mem.set(offset..(text_len + offset), &program.text)?;
		Ok(())
	}

	pub fn push_i32(&mut self, val: i32) -> Result<(), MachineCheck> {
		self.store_i32(val, self.sp)?;
		self.sp += 4;
		Ok(())
	}
	pub fn pop_i32(&mut self) -> Result<i32, MachineCheck> {
		self.sp -= 4;
		self.load_i32(self.sp)
	}
	pub fn set_trace(&mut self, value: bool) {
		self.meta.set(MetaFlags::TRACE, value);
		if value {
			eprintln!(
				"\n{:>8} {:>6} {:>6} {:>6} {:>6} {:>6}",
				"Flag", "BP", "LP", "IP", "SP", "FP"
			);
		}
	}

	pub fn is_user(&self) -> bool {
		self.flag.get_status(Status::USR_MODE)
	}

	// This function does not check alignment.
	pub fn store_slice(&mut self, val: &[u8], offset: i32) -> Result<(), MachineCheck> {
		let offset = i32_to_offset(offset)?;
		self.mem.set(offset..offset + val.len(), val)
	}
	pub fn load_cstr(&self, offset: i32) -> Result<&ffi::CStr, MachineCheck> {
		let offset = i32_to_offset(offset)?;
		let bytes = self.mem.get(offset..)?;
		let Ok(c_str) = ffi::CStr::from_bytes_until_nul(bytes) else {
			return Err(MachineCheck::ILLEGAL_ADDR);
		};
		Ok(c_str)
	}
	pub fn load_abs_i32(&self, offset: i32) -> Result<i32, MachineCheck> {
		check_align(offset)?;
		let offset = i32_to_offset(offset)?;
		let mem = self.mem.get(offset..=(offset + 3))?;
		mem.try_into()
			.map(i32::from_le_bytes)
			.or(Err(MachineCheck::ILLEGAL_ADDR))
	}
	pub fn store_abs_i32(&mut self, val: i32, offset: i32) -> Result<(), MachineCheck> {
		check_align(offset)?;
		let bytes = i32::to_le_bytes(val);
		self.store_slice(&bytes, offset)
	}
	pub fn load_i32(&self, offset: i32) -> Result<i32, MachineCheck> {
		let offset = if self.is_user() {
			offset + self.bp
		} else {
			offset
		};
		self.load_abs_i32(offset)
	}
	pub fn store_i32(&mut self, val: i32, offset: i32) -> Result<(), MachineCheck> {
		let offset = if self.is_user() {
			offset + self.bp
		} else {
			offset
		};
		self.store_abs_i32(val, offset)
	}
	// This function does not check alignment
	pub fn load_u8(&self, offset: i32) -> Result<u8, MachineCheck> {
		let offset = if self.is_user() {
			offset + self.bp
		} else {
			offset
		};
		let offset = i32_to_offset(offset)?;
		self.mem.get(offset..=offset).map(|x| x[0])
	}
	// This function does not check alignment
	pub fn store_u8(&mut self, val: u8, offset: i32) -> Result<(), MachineCheck> {
		let offset = if self.is_user() {
			offset + self.bp
		} else {
			offset
		};
		let offset = i32_to_offset(offset)?;
		self.mem.set(offset..=offset, &[val])
	}
}

// Helper function to convert i32 to usize.
// This function will return Err if val is negative
fn i32_to_offset(val: i32) -> Result<usize, MachineCheck> {
	val.try_into().or(Err(MachineCheck::ILLEGAL_ADDR))
}

fn check_align(offset: i32) -> Result<(), MachineCheck> {
	if offset % 4 != 0 {
		return Err(MachineCheck::ILLEGAL_ADDR);
	}
	Ok(())
}
