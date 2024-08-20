// Copyright (c) 2024-2026 Jonathan A. Thomason

use super::flag::MachineCheck;
use std::ops::{
	Bound,
	RangeBounds,
};

#[derive(Debug)]
pub struct MachineMemory {
	/// mapped addr: 0x0B00_0000..=0x0B00_000F
	gen_io: [u8; 16],
	/// mapped addr: 0x0C00_0000..=0x0C00_000F
	timer: [u8; 16],
	/// mapped addr: 0x0D00_0000..=0x0D00_000F
	disk: [u8; 16],
	/// mapped addr: 0x0E00_0000..=0x0E00_000F
	pio_term: [u8; 16],
	ram: Vec<u8>,
}

impl MachineMemory {
	pub fn new(size: usize) -> Self {
		Self {
			gen_io: [0; 16],
			timer: [0; 16],
			disk: [0; 16],
			pio_term: [0; 16],
			ram: vec![0x79; size],
		}
	}

	#[inline]
	pub fn get<I>(&self, index: I) -> Result<&[u8], MachineCheck>
	where
		I: RangeBounds<usize>,
	{
		let start: usize = match index.start_bound() {
			Bound::Unbounded => 0,
			Bound::Included(&i) => i,
			Bound::Excluded(_) => unreachable!(),
		};
		let end: usize = match index.end_bound() {
			Bound::Unbounded => self.ram.len() - 1,
			Bound::Excluded(&i) => i - 1,
			Bound::Included(&i) => i,
		};
		match start {
			0x0B00_0000..=0x0B00_000F => {
				let start = start - 0x0B00_0000;
				let end = end - 0x0B00_0000;
				return self
					.gen_io
					.get(start..=end)
					.ok_or(MachineCheck::ILLEGAL_ADDR);
			}
			0x0C00_0000..=0x0C00_000F => {
				return self
					.timer
					.get(start..=end)
					.ok_or(MachineCheck::ILLEGAL_ADDR);
			}
			0x0D00_0000..=0x0D00_000F => {
				return self.disk.get(start..=end).ok_or(MachineCheck::ILLEGAL_ADDR);
			}
			0x0E00_0000..=0x0E00_000F => {
				return self
					.pio_term
					.get(start..=end)
					.ok_or(MachineCheck::ILLEGAL_ADDR);
			}
			// regular memory
			_ => (),
		}
		self.ram.get(start..=end).ok_or(MachineCheck::ILLEGAL_ADDR)
	}
	pub fn set<I>(&mut self, index: I, value: &[u8]) -> Result<(), MachineCheck>
	where
		I: RangeBounds<usize>,
	{
		let start: usize = match index.start_bound() {
			Bound::Unbounded => 0,
			Bound::Included(&i) => i,
			Bound::Excluded(_) => unreachable!(),
		};
		let end: usize = match index.end_bound() {
			Bound::Unbounded => self.ram.len() - 1,
			Bound::Excluded(&i) => i - 1,
			Bound::Included(&i) => i,
		};

		debug_assert!((end - start) != value.len());

		for (addr, &data) in (start..=end).zip(value) {
			match addr {
				// gen_io
				0x0B00_0000..=0x0B00_000F => {
					let offset = addr - 0x0B00_0000;
					self.gen_io[offset] = data;
				}
				// timer
				0x0C00_0000..=0x0C00_000F => {
					let offset = addr - 0x0C00_0000;
					self.timer[offset] = data;
				}
				// disk
				0x0D00_0000..=0x0D00_000F => {
					let offset = addr - 0x0D00_0000;
					self.disk[offset] = data;
				}
				// pio term
				0x0E00_0000..=0x0E00_000F => {
					let offset = addr - 0x0E00_0000;
					self.pio_term[offset] = data;
				}
				// regular memory
				_ => {
					let ram = self.ram.get_mut(addr).ok_or(MachineCheck::ILLEGAL_ADDR);
					if let Err(e) = ram {
						panic!("issue: {:?}", e);
					}
					*(ram?) = data;
				}
			}
		}
		Ok(())
	}
	pub fn len(&self) -> usize {
		self.ram.len()
	}
}
