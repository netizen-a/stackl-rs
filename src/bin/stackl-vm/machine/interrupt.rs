// Copyright (c) 2024-2026 Jonathan A. Thomason

use super::MachineState;
use super::flag::*;

impl MachineState {
	pub fn interrupt(&mut self, is_trap: bool) -> Result<(), MachineCheck> {
		const TRAP_VECTOR: usize = 1;
		let was_user = self.is_user();

		let vector = if is_trap {
			TRAP_VECTOR
		} else {
			// Find highest priority pending interrupt
			let pending_interrupt = self.flag.intvec.iter().enumerate().next();

			let Some((vector, int_flag)) = pending_interrupt else {
				// no pending interrupts
				return Ok(());
			};

			// turn off pending bit for HW interrupts
			self.flag.intvec.set(int_flag, false);
			vector
		};

		self.push_i32(self.sp)?;
		self.push_i32(self.flag.as_u32() as i32)?;
		self.push_i32(self.bp)?;
		self.push_i32(self.lp)?;
		self.push_i32(self.ip)?;
		self.push_i32(self.fp)?;

		if !is_trap {
			self.fp = self.sp;
		}

		// go to system mode and interrupt mode
		self.flag.set_status(Status::USR_MODE, false);
		self.flag.set_status(Status::INT_MODE, true);

		if was_user {
			// switch fp and sp to absolute addresses
			self.fp += self.bp;
			self.sp += self.bp;
		}

		// ISR is at vector
		self.ip = self.load_abs_i32(self.ivec + (vector as i32 * 4))?;
		Ok(())
	}
	pub fn machine_check(&mut self, value: MachineCheck) {
		self.flag.intvec.set(IntVec::MACHINE_CHECK, true);
		self.flag.check.set(value, true);
	}
	pub fn rti(&mut self) -> Result<(), MachineCheck> {
		if self.is_user() {
			return Err(MachineCheck::PROT_INST);
		}
		let flag = self.flag;
		self.fp = self.pop_i32()?;
		self.ip = self.pop_i32()?;
		self.lp = self.pop_i32()?;
		self.bp = self.pop_i32()?;
		let new_flag = self.pop_i32()?;
		self.sp = self.pop_i32()?;

		self.flag = MachineFlags::from(new_flag as u32);
		self.flag.intvec = flag.intvec;
		Ok(())
	}
}
