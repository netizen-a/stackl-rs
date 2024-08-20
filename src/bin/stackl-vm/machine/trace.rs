// Copyright (c) 2024-2026 Jonathan A. Thomason

use super::*;

impl MachineState {
	pub fn print_trace(&mut self) -> Result<(), MachineCheck> {
		if self.last_trace > 29 {
			eprintln!(
				"\n{:>8} {:>6} {:>6} {:>6} {:>6} {:>6}",
				"Flag", "BP", "LP", "IP", "SP", "FP"
			);
			self.last_trace = 0;
		} else {
			self.last_trace += 1;
		}
		eprintln!(
			"{:08x} {:6} {:6} {:6} {:6} {:6} {}",
			self.flag.as_u32(),
			self.bp,
			self.lp,
			self.ip,
			self.sp,
			self.fp,
			self.trace_inst(self.ip)?
		);
		Ok(())
	}
	pub fn trace_inst(&self, offset: i32) -> Result<String, MachineCheck> {
		let op = self.load_i32(offset)?;
		let name = match op {
			op::NOP => "NOP",
			op::ADD => "ADD",
			op::SUB => "SUB",
			op::MUL => "MUL",
			op::DIV => "DIV",
			op::MOD => "MOD",
			op::EQ => "EQ",
			op::NE => "NE",
			op::GT => "GT",
			op::LT => "LT",
			op::GE => "GE",
			op::LE => "LE",
			op::AND => "AND",
			op::OR => "OR",
			op::NOT => "NOT",
			op::SWAP => "SWAP",
			op::DUP => "DUP",
			op::HALT => "HALT",
			op::POP => "POP",
			op::RET => "RET",
			op::RETV => "RETV",
			op::NEG => "NEG",
			op::PUSHCVARIND => "PUSHCVARIND ",
			op::OUTS => "OUTS",
			op::INP => "INP",
			op::PUSHFP => &format!("PUSHFP {}", self.fp),
			op::JMPUSER => "JMPUSER ",
			op::TRAP => "TRAP",
			op::RTI => "RTI",
			op::CALLI => "CALLI",
			op::PUSHREG => "PUSHREG ",
			op::POPREG => "POPREG ",
			op::BAND => "BAND",
			op::BOR => "BOR",
			op::BXOR => "BXOR",
			op::SHIFT_LEFT => "SHIFT_LEFT",
			op::SHIFT_RIGHT => "SHIFT_RIGHT",
			op::PUSHVARIND => {
				let offset = self.load_i32(self.sp - 4)?;
				let temp1 = self.load_i32(offset)?;
				let temp2 = self.load_i32(self.sp - 4)?;
				&format!("PUSHVARIND {temp2} {temp1}")
			}
			op::POPCVARIND => "POPCVARIND ",
			op::POPVARIND => {
				// DEBUG("POPVARIND %d %d", GET_INTVAL(SP, -2), GET_INTVAL(SP, -1));
				let temp1 = self.load_i32(self.sp - 8)?;
				let temp2 = self.load_i32(self.sp - 4)?;
				&format!("POPVARIND {temp1} {temp2}")
			}
			op::COMP => "COMP",
			op::PUSH => "PUSH ",
			op::JMP => "JMP ",
			op::JZ => "JZ ",
			op::PUSHVAR => "PUSHVAR ",
			op::POPVAR => "POPVAR ",
			op::ADJSP => "ADJSP ",
			op::POPARGS => "POPARGS ",
			op::CALL => "CALL ",
			op::PUSHCVAR => "PUSHCVAR ",
			op::POPCVAR => "POPCVAR ",
			op::SET_TRACE => "SET_TRACE",
			op::CLR_TRACE => "CLR_TRACE",
			op::CLR_INT_DIS => "CLR_INT_DIS",
			op::SET_INT_DIS => "SET_INT_DIS",
			op::ROTATE_LEFT => "ROTATE_LEFT",
			op::ROTATE_RIGHT => "ROTATE_RIGHT",
			_ => "ILLEGAL",
		};
		let mut inst = String::from(name);
		match op {
			op::POPARGS | op::PUSH | op::JMP | op::JMPUSER | op::ADJSP | op::CALL => {
				let operand = self.load_i32(offset + 4)?;
				inst.push_str(&operand.to_string());
			}
			op::JZ => {
				let cond = self.load_i32(self.sp - 4)?;
				let operand = self.load_i32(offset + 4)?;
				inst.push_str(&format!("{cond} {operand}"));
			}
			op::PUSHREG | op::POPREG => {
				let operand = self.load_i32(offset + 4)?;
				let reg = match operand {
					0 => "BP",
					1 => "LP",
					2 => "IP",
					3 => "SP",
					4 => "FP",
					5 => "FLAG",
					6 => "IVEC",
					_ => &format!("{operand}"),
				};
				inst.push_str(reg);
			}
			op::PUSHVAR | op::POPVAR => {
				let operand = self.load_i32(offset + 4)?;
				inst.push_str(&operand.to_string());
				let value = self.load_i32(self.fp + operand)?;
				inst.push(' ');
				inst.push_str(&value.to_string());
			}
			57..=i32::MAX | i32::MIN..0 => {
				inst.push('(');
				inst.push_str(&op.to_string());
				inst.push(')');
			}
			_ => {}
		};

		Ok(inst)
	}
}
