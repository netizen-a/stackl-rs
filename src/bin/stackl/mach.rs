use std::sync::mpsc::{Receiver, Sender};

use crate::chk::MachineCheck;
use crate::flag::{MachineFlags, Status};
use crate::ram;
use crate::{chk, msg};
use stackl::op;

#[allow(dead_code)]
pub struct MachineState {
    pub bp: i32,
    pub lp: i32,
    pub ip: i32,
    pub sp: i32,
    pub fp: i32,
    pub flag: MachineFlags,
    pub ivec: i32,
    pub vmem: i32,
}

impl MachineState {
    pub fn new(mem_size: usize) -> MachineState {
        MachineState {
            bp: 0,
            lp: mem_size.try_into().unwrap(),
            ip: 0,
            sp: 0,
            fp: 0,
            flag: MachineFlags::new(),
            ivec: 0,
            vmem: 0,
        }
    }
    pub fn push_i32(&mut self, val: i32) -> Result<(), chk::MachineCheck> {
        let mut ram_lock = ram::VM_MEM.write().unwrap();
        ram_lock.store_i32(val, self.sp)?;
        self.sp += 4;
        Ok(())
    }
    pub fn pop_i32(&mut self) -> Result<i32, chk::MachineCheck> {
        let ram_lock = ram::VM_MEM.read().unwrap();
        self.sp -= 4;
        ram_lock.load_i32(self.sp)
    }
    pub fn load_i32(&self, offset: i32) -> Result<i32, chk::MachineCheck> {
        let ram_lock = ram::VM_MEM.read().unwrap();
        ram_lock.load_i32(offset)
    }
    pub fn load_u8(&self, offset: i32) -> Result<u8, chk::MachineCheck> {
        let ram_lock = ram::VM_MEM.read().unwrap();
        ram_lock.load_u8(offset)
    }
    pub fn store_u8(&mut self, val: u8, offset: i32) -> Result<(), chk::MachineCheck> {
        let mut ram_lock = ram::VM_MEM.write().unwrap();
        ram_lock.store_u8(val, offset)
    }
    pub fn store_i32(&mut self, val: i32, offset: i32) -> Result<(), chk::MachineCheck> {
        let mut ram_lock = ram::VM_MEM.write().unwrap();
        ram_lock.store_i32(val, offset)
    }
    pub fn set_trace(&mut self, value: bool) {
        self.flag.set_status(Status::TRACE, value);
        if value {
            eprintln!(
                "\n{:>8} {:>6} {:>6} {:>6} {:>6} {:>6}",
                "Flag", "BP", "LP", "IP", "SP", "FP"
            );
        }
    }
    pub fn is_user_mode(&self) -> bool {
        self.flag.get_status(Status::USER_MODE)
    }
    pub fn run(
        &mut self,
        request_send: Sender<msg::MachineRequest>,
        response_recv: Receiver<msg::MachineResponse>,
    ) {
        loop {
            for recv in response_recv.try_iter() {
                println!("machine: {recv:?}");
            }
            if self.flag.get_status(Status::HALTED) {
                return;
            }
            execute_op(self, &request_send).expect("uncaught machine check");
        }
    }
}

fn execute_op(
    cpu: &mut MachineState,
    request_send: &Sender<msg::MachineRequest>,
) -> Result<(), chk::MachineCheck> {
    if cpu.flag.get_status(Status::TRACE) {
        eprintln!(
            "{:08x} {:6} {:6} {:6} {:6} {:6} {}",
            cpu.flag.as_u32(),
            cpu.bp,
            cpu.lp,
            cpu.ip,
            cpu.sp,
            cpu.fp,
            cpu.load_i32(cpu.ip)?
        );
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
                return Err(MachineCheck::new(
                    chk::CheckKind::IllegalOp,
                    "Divide by Zero",
                ));
            }
        }
        op::MOD => {
            let rhs = cpu.pop_i32()?;
            let lhs = cpu.pop_i32()?;
            if let Some(result) = lhs.checked_rem_euclid(rhs) {
                cpu.push_i32(result)?;
            } else {
                return Err(MachineCheck::new(
                    chk::CheckKind::IllegalOp,
                    "Divide by Zero",
                ));
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
            if cpu.is_user_mode() {
                return Err(MachineCheck::from(chk::CheckKind::ProtInst));
            }
            cpu.flag.set_status(Status::HALTED, true);
            return Ok(());
        }
        op::POP => {
            cpu.sp -= 4;
        }
        op::RET => {
            cpu.sp = cpu.fp - 4;
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
            if cpu.is_user_mode() {
                return Err(MachineCheck::from(chk::CheckKind::ProtInst));
            }
            let ram_lock = ram::VM_MEM.read().unwrap();
            let offset = ram_lock.load_i32(cpu.sp - 4)?;
            ram_lock.print(offset)?;
        }
        op::INP => {
            if cpu.is_user_mode() {
                return Err(MachineCheck::from(chk::CheckKind::ProtInst));
            }
            let offset = cpu.pop_i32()?;
            let op = cpu.load_i32(offset)?;
            let request = match op {
                3 => {
                    let param1 = cpu.load_i32(offset + 4)?;
                    msg::MachineRequest::Prints(param1)
                }
                _ => msg::MachineRequest::Unknown,
            };
            request_send.send(request).unwrap();
        }
        op::PUSHFP => {
            cpu.push_i32(cpu.fp)?;
        }
        op::JMPUSER => {
            if cpu.is_user_mode() {
                return Err(MachineCheck::from(chk::CheckKind::ProtInst));
            }
            cpu.ip += 4;
            cpu.ip = cpu.load_i32(cpu.ip)?;
            cpu.flag.set_status(Status::USER_MODE, true);
            return Ok(());
        }
        op::TRAP => {
            if cpu.is_user_mode() {
                return Err(MachineCheck::from(chk::CheckKind::ProtInst));
            }
            todo!("trap")
        }
        op::RTI => {
            if cpu.is_user_mode() {
                return Err(MachineCheck::from(chk::CheckKind::ProtInst));
            }
            todo!("rti")
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
                _ => {
                    return Err(chk::MachineCheck::new(
                        chk::CheckKind::IllegalOp,
                        "Invalid Register",
                    ))
                }
            }
        }
        op::POPREG => {
            if cpu.is_user_mode() {
                return Err(MachineCheck::from(chk::CheckKind::ProtInst));
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
                _ => {
                    return Err(chk::MachineCheck::new(
                        chk::CheckKind::IllegalOp,
                        "invalid register",
                    ))
                }
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
            let rhs = cpu.pop_i32()?;
            let lhs = cpu.pop_i32()?;
            cpu.push_i32(lhs.wrapping_shl(rhs as u32))?;
        }
        op::SHIFT_RIGHT => {
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
            if cpu.is_user_mode() {
                return Err(MachineCheck::from(chk::CheckKind::ProtInst));
            }
            cpu.flag.set_status(Status::INT_DIS, false);
        }
        op::SET_INT_DIS => {
            if cpu.is_user_mode() {
                return Err(MachineCheck::from(chk::CheckKind::ProtInst));
            }
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
        57..=i32::MAX | i32::MIN..0 => return Err(MachineCheck::from(chk::CheckKind::IllegalInst)),
    }
    cpu.ip += 4;
    Ok(())
}
