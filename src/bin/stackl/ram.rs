use stackl::op;

use crate::chk;
use core::time;
use std::io::Write;
use std::sync::RwLock;
use std::{io, thread};

pub static VM_RAM: RwLock<Memory> = RwLock::new(Memory::new());
pub static VM_ROM: RwLock<Memory> = RwLock::new(Memory::new());

pub struct Memory {
    inner: Vec<u8>,
}

impl Memory {
    pub fn resize(&mut self, new_len: usize, value: u8) {
        self.inner.resize(new_len, value);
    }
    pub const fn new() -> Self {
        Memory { inner: Vec::new() }
    }
    // returns true if success, else false
    // This function does not check alignment.
    pub fn store_slice(&mut self, val: &[u8], offset: i32) -> Result<(), chk::MachineCheck> {
        let mem = &mut self.inner;
        let offset = i32_to_offset(offset)?;
        if let Some(ram) = mem.get_mut(offset..offset + val.len()) {
            ram.clone_from_slice(val);
            Ok(())
        } else {
            Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
        }
    }
    pub fn load_i32(&self, offset: i32) -> Result<i32, chk::MachineCheck> {
        let mem = &self.inner;
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
    // This function does not check alignment
    pub fn load_u8(&self, offset: i32) -> Result<u8, chk::MachineCheck> {
        let mem = &self.inner;
        let offset = i32_to_offset(offset)?;
        mem.get(offset)
            .copied()
            .ok_or(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
    }
    // This function does not check alignment
    pub fn store_u8(&mut self, val: u8, offset: i32) -> Result<(), chk::MachineCheck> {
        let mem = &mut self.inner;
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
        let mem = &self.inner;
        let offset = i32_to_offset(offset)?;
        if let Some(bytes) = mem.get(offset..) {
            for chunk in bytes.utf8_chunks() {
                for ch in chunk.valid().chars() {
                    thread::sleep(time::Duration::from_micros(100));
                    if ch == '\0' {
                        return Ok(());
                    }
                    print!("{ch}");
                    io::stdout().flush().unwrap();
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
