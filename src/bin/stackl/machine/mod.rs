use std::io::Write;
use std::sync::mpsc::Sender;
use std::{io, thread, time};

use crate::chk;
use crate::chk::MachineCheck;
use crate::flag::{MachineFlags, Status};
use stackl::op;

pub mod step;

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
    pub ram: Vec<u8>,
    pub rom: Vec<u8>,
}

impl MachineState {
    pub fn new(ivec: i32, mem_size: usize) -> MachineState {
        MachineState {
            bp: 0,
            lp: mem_size.try_into().unwrap(),
            ip: 0,
            sp: 0,
            fp: 0,
            flag: MachineFlags::new(),
            ivec,
            vmem: 0,
            ram: vec![],
            rom: vec![],
        }
    }
    pub fn push_i32(&mut self, val: i32) -> Result<(), chk::MachineCheck> {
        self.store_i32(val, self.sp)?;
        self.sp += 4;
        Ok(())
    }
    pub fn pop_i32(&mut self) -> Result<i32, chk::MachineCheck> {
        self.sp -= 4;
        self.load_i32(self.sp)
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
    pub fn get_trap_addr(&self) -> Result<i32, MachineCheck> {
        if self.ivec == -1 {
            println!("default ivec");
            if let Some(mem) = self.rom.get(4..8) {
                mem.try_into()
                    .map(i32::from_le_bytes)
                    .or(Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr)))
            } else {
                Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
            }
        } else {
            println!("custom ivec");
            self.load_i32(self.ivec + 4)
        }
    }
    pub fn is_user_mode(&self) -> bool {
        self.flag.get_status(Status::USR_MODE)
    }

    pub fn resize_ram(&mut self, new_len: usize, value: u8) {
        self.ram.resize(new_len, value);
    }
    // returns true if success, else false
    // This function does not check alignment.
    pub fn store_slice(&mut self, val: &[u8], offset: i32) -> Result<(), chk::MachineCheck> {
        let mem = &mut self.ram;
        let offset = i32_to_offset(offset)?;
        if let Some(ram) = mem.get_mut(offset..offset + val.len()) {
            ram.clone_from_slice(val);
            Ok(())
        } else {
            Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
        }
    }
    // returns true if success, else false
    // This function does not check alignment.
    pub fn rom_store_slice(&mut self, val: &[u8], offset: i32) -> Result<(), chk::MachineCheck> {
        let mem = &mut self.rom;
        let offset = i32_to_offset(offset)?;
        if let Some(ram) = mem.get_mut(offset..offset + val.len()) {
            ram.clone_from_slice(val);
            Ok(())
        } else {
            Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
        }
    }
    pub fn load_slice<'a>(&'a self, offset: i32) -> Result<&'a [u8], chk::MachineCheck> {
        let offset = i32_to_offset(offset)?;
        self.ram.get(offset..)
            .ok_or(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
    }
    pub fn load_i32(&self, offset: i32) -> Result<i32, chk::MachineCheck> {
        let mem = &self.ram;
        let offset = i32_to_offset(offset)?;
        if offset % 4 != 0 {
            return Err(chk::MachineCheck::new(
                chk::CheckKind::IllegalAddr,
                format!("Misaligned Address at {offset}"),
            ));
        }
        if let Some(mem) = mem.get(offset..=(offset + 3)) {
            mem.try_into()
                .map(i32::from_le_bytes)
                .or(Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr)))
        } else {
            Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
        }
    }
    pub fn store_i32(&mut self, val: i32, offset: i32) -> Result<(), chk::MachineCheck> {
        if offset % 4 != 0 {
            return Err(chk::MachineCheck::new(
                chk::CheckKind::IllegalAddr,
                format!("Misaligned Address at {offset}"),
            ));
        }
        let bytes = i32::to_le_bytes(val);
        self.store_slice(&bytes, offset)
    }
    pub fn rom_store_i32(&mut self, val: i32, offset: i32) -> Result<(), chk::MachineCheck> {
        if offset % 4 != 0 {
            return Err(chk::MachineCheck::new(
                chk::CheckKind::IllegalAddr,
                format!("Misaligned Address at {offset}"),
            ));
        }
        let bytes = i32::to_le_bytes(val);
        self.rom_store_slice(&bytes, offset)
    }
    // This function does not check alignment
    pub fn load_u8(&self, offset: i32) -> Result<u8, chk::MachineCheck> {
        let mem = &self.ram;
        let offset = i32_to_offset(offset)?;
        mem.get(offset)
            .copied()
            .ok_or(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
    }
    // This function does not check alignment
    pub fn store_u8(&mut self, val: u8, offset: i32) -> Result<(), chk::MachineCheck> {
        let mem = &mut self.ram;
        let offset = i32_to_offset(offset)?;
        if let Some(byte) = mem.get_mut(offset) {
            *byte = val;
            Ok(())
        } else {
            Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
        }
    }
    // This function does not check alignment
    pub fn print(&self, offset: i32) -> Result<(), chk::MachineCheck> {
        let mem = &self.ram;
        let offset = i32_to_offset(offset)?;
        if let Some(bytes) = mem.get(offset..) {
            for chunk in bytes.utf8_chunks() {
                for ch in chunk.valid().chars() {
                    thread::sleep(time::Duration::from_micros(100));
                    if ch == '\0' {
                        return Ok(());
                    }
                    print!("{ch}");
                    io::stdout().flush().unwrap()
                }
                for byte in chunk.invalid() {
                    thread::sleep(time::Duration::from_micros(100));
                    print!("\\x{:02X}", byte);
                    io::stdout().flush().unwrap();
                }
            }
            Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
        } else {
            Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
        }
    }
    pub fn trace_inst(&self, offset: i32) -> Result<String, chk::MachineCheck> {
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
            op::PUSHCVARIND => "PUSHCVARIND",
            op::OUTS => "OUTS",
            op::INP => "INP",
            op::PUSHFP => "PUSHFP",
            op::JMPUSER => "JMPUSER",
            op::TRAP => "TRAP",
            op::RTI => "RTI",
            op::CALLI => "CALLI",
            op::PUSHREG => "PUSHREG",
            op::POPREG => "POPREG",
            op::BAND => "BAND",
            op::BOR => "BOR",
            op::BXOR => "BXOR",
            op::SHIFT_LEFT => "SHIFT_LEFT",
            op::SHIFT_RIGHT => "SHIFT_RIGHT",
            op::PUSHVARIND => "PUSHVARIND",
            op::POPCVARIND => "POPCVARIND",
            op::POPVARIND => "POPVARIND",
            op::COMP => "COMP",
            op::PUSH => "PUSH ",
            op::JMP => "JMP ",
            op::JZ => "JZ ",
            op::PUSHVAR => "PUSHVAR",
            op::POPVAR => "POPVAR",
            op::ADJSP => "ADJSP",
            op::POPARGS => "POPARGS",
            op::CALL => "CALL",
            op::PUSHCVAR => "PUSHCVAR",
            op::POPCVAR => "POPCVAR",
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
            op::JZ | op::PUSH | op::JMP => {
                let operand = self.load_i32(offset + 4)?;
                inst.push_str(&operand.to_string());
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



// Helper function to convert i32 to usize.
// This function will return Err if val is negative
fn i32_to_offset(val: i32) -> Result<usize, chk::MachineCheck> {
    val.try_into()
        .or(Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr)))
}