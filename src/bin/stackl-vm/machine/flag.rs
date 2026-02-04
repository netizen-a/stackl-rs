// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::fmt;

use bitflags::bitflags;

bitflags! {
	#[derive(Debug, Clone, Copy, Default)]
	pub struct Status: u8 {
		const HALTED           = 1;
		const USR_MODE         = 1 << 1;
		const INT_MODE         = 1 << 2;
		const INT_DIS          = 1 << 3;
		const VMEM_MODE        = 1 << 4;
		const FPU_ENABLE       = 1 << 5;
		const _                = !0;
	}
}

bitflags! {
	#[derive(Debug, Clone, Copy, Default, PartialEq)]
	pub struct MachineCheck: u8 {
		/// Illegal Instruction
		const ILLEGAL_INST = 1;
		/// Illegal Address
		const ILLEGAL_ADDR = 1 << 1;
		/// Hardware Failure
		const HW_FAILURE   = 1 << 2;
		/// Hardware Warning
		const HW_WARNING   = 1 << 3;
		/// Protected Instruction
		const PROT_INST    = 1 << 4;
		/// Divide by zero
		const DIVIDE_ZERO  = 1 << 5;
		// TODO: Overflow exception
		const OVF          = 1 << 6;
		// TODO: Floating point arithmetic exception
		const FPE          = 1 << 7;
	}
}

impl fmt::Display for MachineCheck {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let kind = match *self {
			MachineCheck::ILLEGAL_INST => "Illegal Instruction".to_string(),
			MachineCheck::ILLEGAL_ADDR => "Illegal Address".to_string(),
			MachineCheck::HW_FAILURE => "Hardware Failure".to_string(),
			MachineCheck::HW_WARNING => "Hardware Warning".to_string(),
			MachineCheck::PROT_INST => "Protected Instruction".to_string(),
			_ => "Illegal Operation".to_string(),
		};
		write!(f, "{kind}")
	}
}

bitflags! {
	#[derive(Debug, Clone, Copy, Default)]
	pub struct IntVec: u16 {
		const MACHINE_CHECK = 1;
		const TRAP          = 1 << 1;
		const DISK          = 1 << 2;
		const TIMER         = 1 << 3;
		const DMA_T         = 1 << 4;
		const PIO_T         = 1 << 5;
		const GEN_IO        = 1 << 8;
		// TODO: Breakpoint caused by `BREAK` instruction
		const BKPT          = 1 << 9;
		const _ = !0;
	}
}

bitflags! {
	#[derive(Debug, Clone, Copy, Default)]
	pub struct MetaFlags: u32 {
		const TRACE            = 1;
		const FEATURE_GEN_IO   = 1 << 1;
		const FEATURE_PIO_TERM = 1 << 2;
		const FEATURE_DMA_TERM = 1 << 3;
		const FEATURE_DISK     = 1 << 4;
		const FEATURE_INP      = 1 << 5;
		const _                = !0;
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MachineFlags {
	pub status: Status,
	pub check: MachineCheck,
	pub intvec: IntVec,
}

impl MachineFlags {
	pub const fn new() -> Self {
		Self {
			status: Status::empty(),
			check: MachineCheck::empty(),
			intvec: IntVec::empty(),
		}
	}
	pub fn set_status(&mut self, flag: Status, value: bool) {
		self.status.set(flag, value);
	}
	pub fn get_status(&self, flag: Status) -> bool {
		self.status.contains(flag)
	}
	pub fn set_intvec(&mut self, flag: IntVec, value: bool) {
		self.intvec.set(flag, value);
	}
	pub fn get_intvec(&self, flag: IntVec) -> bool {
		self.intvec.contains(flag)
	}
	pub fn set_check(&mut self, flag: MachineCheck, value: bool) {
		self.check.set(flag, value);
	}
	pub fn get_check(&self, flag: MachineCheck) -> bool {
		self.check.contains(flag)
	}
	pub const fn as_u32(&self) -> u32 {
		let mut result: u32 = self.status.bits() as u32;
		result |= (self.check.bits() as u32) << 8;
		result |= (self.intvec.bits() as u32) << 16;
		result
	}
}

impl From<MachineFlags> for u32 {
	fn from(value: MachineFlags) -> Self {
		value.as_u32()
	}
}

impl From<u32> for MachineFlags {
	fn from(value: u32) -> Self {
		Self {
			status: Status::from_bits_retain(value as u8),
			check: MachineCheck::from_bits_retain((value >> 8) as u8),
			intvec: IntVec::from_bits_retain((value >> 16) as u16),
		}
	}
}
