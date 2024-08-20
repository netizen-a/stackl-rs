// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::device::inp::Request;
use crate::io;

use super::*;

pub fn next_opcode(
	cpu: &mut MachineState,
	request_send: &Sender<Request>,
) -> Result<(), MachineCheck> {
	if !cpu.flag.intvec.is_empty()
		&& !cpu.flag.get_status(Status::INT_MODE)
		&& !cpu.flag.get_status(Status::INT_DIS)
	{
		return cpu.interrupt(false);
	}

	if cpu.meta.contains(MetaFlags::TRACE) {
		cpu.print_trace()?;
	}

	let op: i32 = cpu.load_i32(cpu.ip)?;

	match op {
		op::NOP => {}
		op::ADD => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32(lhs.wrapping_add(rhs))?;
		}
		op::SUB => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32(lhs.wrapping_sub(rhs))?;
		}
		op::MUL => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32(lhs.wrapping_mul(rhs))?;
		}
		op::DIV => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			if let Some(result) = lhs.checked_div(rhs) {
				cpu.push_i32(result)?;
			} else {
				cpu.machine_check(MachineCheck::DIVIDE_ZERO);
			}
		}
		op::MOD => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			if let Some(result) = lhs.checked_rem_euclid(rhs) {
				cpu.push_i32(result)?;
			} else {
				cpu.machine_check(MachineCheck::DIVIDE_ZERO);
			}
		}
		op::EQ => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32((lhs == rhs) as i32)?;
		}
		op::NE => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32((lhs != rhs) as i32)?;
		}
		op::GT => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32((lhs > rhs) as i32)?;
		}
		op::LT => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32((lhs < rhs) as i32)?;
		}
		op::GE => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32((lhs >= rhs) as i32)?;
		}
		op::LE => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32((lhs <= rhs) as i32)?;
		}
		op::AND => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32((lhs != 0 && rhs != 0) as i32)?;
		}
		op::OR => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32((lhs != 0 || rhs != 0) as i32)?;
		}
		op::NOT => {
			let val = cpu.pop_i32()?;
			cpu.push_i32((val == 0) as i32)?;
		}
		op::SWAP => {
			let tmp0 = cpu.pop_i32()?;
			let tmp1 = cpu.pop_i32()?;
			cpu.push_i32(tmp0)?;
			cpu.push_i32(tmp1)?;
		}
		op::DUP => {
			let val = cpu.load_i32(cpu.sp - 4)?;
			cpu.store_i32(val, cpu.sp)?;
			cpu.sp += 4;
		}
		op::HALT => {
			if cpu.is_user() {
				return Err(MachineCheck::PROT_INST);
			}
			cpu.flag.set_status(Status::HALTED, true);
			return Ok(());
		}
		op::POP => {
			cpu.sp -= 4;
		}
		op::RET => {
			cpu.sp = cpu.fp - 8;
			cpu.ip = cpu.load_i32(cpu.fp - 8)?;
			cpu.fp = cpu.load_i32(cpu.fp - 4)?;
			return Ok(());
		}
		op::RETV => {
			let tmp = cpu.load_i32(cpu.sp - 4)?;
			cpu.sp = cpu.fp - 4;
			cpu.ip = cpu.load_i32(cpu.fp - 8)?;
			cpu.fp = cpu.load_i32(cpu.fp - 4)?;
			cpu.store_i32(tmp, cpu.sp - 4)?;
			return Ok(());
		}
		op::NEG => {
			let val = cpu.pop_i32()?;
			cpu.push_i32(-val)?;
		}
		op::PUSHCVARIND => {
			let offset = cpu.pop_i32()?;
			let val = cpu.load_u8(offset)?;
			cpu.push_i32(val as i32)?;
		}
		op::OUTS => {
			if cpu.is_user() {
				return Err(MachineCheck::PROT_INST);
			}
			let offset = cpu.pop_i32()? as usize;
			let buf = cpu.mem.get(offset..)?;
			io::try_print(buf);
		}
		op::INP => {
			if !cpu.meta.contains(MetaFlags::FEATURE_INP) {
				return Err(MachineCheck::ILLEGAL_INST);
			}
			if cpu.is_user() {
				return Err(MachineCheck::PROT_INST);
			}
			let offset = cpu.pop_i32()?;
			let request = Request {
				offset,
				op: cpu.load_i32(offset)?,
				param1: cpu.load_i32(offset + 4)?,
				param2: cpu.load_i32(offset + 8)?,
				bp: cpu.bp,
			};
			request_send.send(request).unwrap();
		}
		op::PUSHFP => {
			cpu.push_i32(cpu.fp)?;
		}
		op::JMPUSER => {
			if cpu.is_user() {
				return Err(MachineCheck::PROT_INST);
			}
			cpu.ip = cpu.load_i32(cpu.ip + 4)?;
			cpu.flag.set_status(Status::USR_MODE, true);
			return Ok(());
		}
		op::TRAP => {
			cpu.ip += 4;
			return cpu.interrupt(true);
		}
		op::RTI => {
			return cpu.rti();
		}
		op::CALLI => {
			let tmp = cpu.pop_i32()?;
			cpu.push_i32(cpu.ip + 4)?;
			cpu.push_i32(cpu.fp)?;
			cpu.fp = cpu.sp;
			cpu.ip = tmp;
			return Ok(());
		}
		op::PUSHREG => {
			cpu.ip += 4;
			let reg = cpu.load_i32(cpu.ip)?;
			match reg {
				0 => cpu.push_i32(cpu.bp)?,
				1 => cpu.push_i32(cpu.lp)?,
				2 => cpu.push_i32(cpu.ip)?,
				3 => cpu.push_i32(cpu.sp)?,
				4 => cpu.push_i32(cpu.fp)?,
				5 => cpu.push_i32(cpu.flag.as_u32() as i32)?,
				6 => cpu.push_i32(cpu.ivec)?,
				_ => return Err(MachineCheck::ILLEGAL_INST),
			}
		}
		op::POPREG => {
			if cpu.is_user() {
				return Err(MachineCheck::PROT_INST);
			}
			cpu.ip += 4;
			let reg = cpu.load_i32(cpu.ip)?;
			match reg {
				0 => cpu.bp = cpu.pop_i32()?,
				1 => cpu.lp = cpu.pop_i32()?,
				2 => {
					cpu.ip = cpu.pop_i32()?;
					return Ok(());
				}
				3 => cpu.sp = cpu.pop_i32()?,
				4 => cpu.fp = cpu.pop_i32()?,
				5 => {
					let val = cpu.pop_i32()? as u32;
					cpu.flag = MachineFlags::from(val)
				}
				6 => cpu.ivec = cpu.pop_i32()?,
				_ => return Err(MachineCheck::ILLEGAL_INST),
			}
		}
		op::BAND => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32(lhs & rhs)?;
		}
		op::BOR => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32(lhs | rhs)?;
		}
		op::BXOR => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32(lhs ^ rhs)?;
		}
		op::SHIFT_LEFT => {
			// TODO: fix incorrect shift when right operand is negative
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32(lhs.wrapping_shl(rhs as u32))?;
		}
		op::SHIFT_RIGHT => {
			// TODO: fix incorrect shift when right operand is negative
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32(lhs.wrapping_shr(rhs as u32))?;
		}
		op::PUSHVARIND => {
			let offset = cpu.pop_i32()?;
			let val = cpu.load_i32(offset)?;
			cpu.push_i32(val)?;
		}
		op::POPCVARIND => {
			let offset = cpu.pop_i32()?;
			let val = cpu.pop_i32()?;
			cpu.store_u8(val as u8, offset)?;
		}
		op::POPVARIND => {
			let offset = cpu.pop_i32()?;
			let val = cpu.pop_i32()?;
			cpu.store_i32(val, offset)?;
		}
		op::COMP => {
			let val = cpu.pop_i32()?;
			cpu.push_i32(!val)?;
		}
		op::PUSH => {
			cpu.ip += 4;
			let val = cpu.load_i32(cpu.ip)?;
			cpu.push_i32(val)?;
		}
		op::JMP => {
			cpu.ip += 4;
			cpu.ip = cpu.load_i32(cpu.ip)?;
			return Ok(());
		}
		op::JZ => {
			let val = cpu.pop_i32()?;
			if val == 0 {
				cpu.ip += 4;
				cpu.ip = cpu.load_i32(cpu.ip)?;
			} else {
				cpu.ip += 8;
			}
			return Ok(());
		}
		op::PUSHVAR => {
			cpu.ip += 4;
			let offset = cpu.load_i32(cpu.ip)?;
			let val = cpu.load_i32(cpu.fp + offset)?;
			cpu.push_i32(val)?;
		}
		op::POPVAR => {
			cpu.ip += 4;
			let offset = cpu.load_i32(cpu.ip)?;
			let val = cpu.pop_i32()?;
			cpu.store_i32(val, cpu.fp + offset)?;
		}
		op::ADJSP => {
			cpu.ip += 4;
			cpu.sp += cpu.load_i32(cpu.ip)?;
		}
		op::POPARGS => {
			let tmp = cpu.pop_i32()?;
			cpu.ip += 4;
			cpu.sp -= cpu.load_i32(cpu.ip)?;
			cpu.push_i32(tmp)?;
		}
		op::CALL => {
			cpu.push_i32(cpu.ip + 8)?;
			cpu.push_i32(cpu.fp)?;
			cpu.fp = cpu.sp;
			cpu.ip = cpu.load_i32(cpu.ip + 4)?;
			return Ok(());
		}
		op::PUSHCVAR => {
			cpu.ip += 4;
			let offset = cpu.load_i32(cpu.ip)?;
			let val = cpu.load_u8(cpu.fp + offset)?;
			cpu.push_i32(val.into())?;
		}
		op::POPCVAR => {
			cpu.ip += 4;
			let offset = cpu.load_i32(cpu.ip)?;
			let val = cpu.pop_i32()?;
			cpu.store_u8(val as u8, cpu.fp + offset)?;
		}
		op::SET_TRACE => {
			cpu.set_trace(true);
		}
		op::CLR_TRACE => {
			cpu.set_trace(false);
		}
		op::CLR_INT_DIS => {
			if cpu.is_user() {
				return Err(MachineCheck::PROT_INST);
			}
			let value = cpu.flag.get_status(Status::INT_DIS);
			cpu.push_i32(value as i32)?;
			cpu.flag.set_status(Status::INT_DIS, false);
		}
		op::SET_INT_DIS => {
			if cpu.is_user() {
				return Err(MachineCheck::PROT_INST);
			}
			let value = cpu.flag.get_status(Status::INT_DIS);
			cpu.push_i32(value as i32)?;
			cpu.flag.set_status(Status::INT_DIS, true);
		}
		op::ROTATE_LEFT => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32(lhs.rotate_left(rhs as u32))?;
		}
		op::ROTATE_RIGHT => {
			let rhs = cpu.pop_i32()?;
			let lhs = cpu.pop_i32()?;
			cpu.push_i32(lhs.rotate_right(rhs as u32))?;
		}
		57..=i32::MAX | i32::MIN..0 => return Err(MachineCheck::ILLEGAL_INST),
	}
	cpu.ip += 4;
	Ok(())
}
