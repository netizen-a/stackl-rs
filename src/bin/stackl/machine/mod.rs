use std::io::Write;
use std::sync::mpsc::Sender;
use std::{ffi, io, thread, time};

use crate::chk;
use crate::chk::MachineCheck;
use crate::flag::{MachineFlags, Status};
use stackl::{op, StacklFormat};

pub mod step;

#[derive(Debug)]
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
    pub fn new(program: StacklFormat, mem_size: usize) -> MachineState {
        let sp_addr = if program.text.len() % 2 != 0 {
            program.text.len() + 2 - (program.text.len() % 2)
        } else {
            program.text.len()
        };
        let mut rom = vec![0u8; 64];
        for slot in rom.chunks_exact_mut(4) {
            slot.copy_from_slice(&1i32.to_le_bytes());
        }
        if program.trap_vec != -1 {
            rom[4..8].copy_from_slice(&program.trap_vec.to_le_bytes());
        }
        let flag = MachineFlags::new();

        let mut ram = vec![0x79; mem_size];
        ram[..program.text.len()].copy_from_slice(&program.text);

        MachineState {
            bp: 0,
            lp: mem_size.try_into().unwrap(),
            ip: 0,
            sp: sp_addr as i32,
            fp: 0,
            flag,
            ivec: program.int_vec,
            vmem: 0,
            ram,
            rom,
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
            if let Some(mem) = self.rom.get(4..8) {
                mem.try_into()
                    .map(i32::from_le_bytes)
                    .or(Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr)))
            } else {
                Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
            }
        } else {
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
    pub fn load_cstr(&self, offset: i32) -> Result<&ffi::CStr, chk::MachineCheck> {
        let offset = i32_to_offset(offset)?;
        let bytes = self.ram
            .get(offset..)
            .ok_or(chk::MachineCheck::from(chk::CheckKind::IllegalAddr));
        let Ok(c_str) = ffi::CStr::from_bytes_until_nul(bytes?) else {
            return Err(chk::MachineCheck::from(chk::CheckKind::Other));
        };
        Ok(c_str)
    }
    pub fn load_i32(&self, offset: i32) -> Result<i32, chk::MachineCheck> {
        let mem = &self.ram;
        let offset = if self.is_user_mode() {
            offset + self.bp
        } else {
            offset
        };
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
        let offset = if self.is_user_mode() {
            offset + self.bp
        } else {
            offset
        };
        self.store_slice(&bytes, offset)
    }
    // This function does not check alignment
    pub fn load_u8(&self, offset: i32) -> Result<u8, chk::MachineCheck> {
        let mem = &self.ram;
        let offset = if self.is_user_mode() {
            offset + self.bp
        } else {
            offset
        };
        let offset = i32_to_offset(offset)?;
        mem.get(offset)
            .copied()
            .ok_or(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
    }
    // This function does not check alignment
    pub fn store_u8(&mut self, val: u8, offset: i32) -> Result<(), chk::MachineCheck> {
        let offset = if self.is_user_mode() {
            offset + self.bp
        } else {
            offset
        };
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
        let offset = if self.is_user_mode() {
            offset + self.bp
        } else {
            offset
        };
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
            op::JMPUSER => "JMPUSER ",
            op::TRAP => "TRAP",
            op::RTI => "RTI",
            op::CALLI => "CALLI",
            op::PUSHREG => "PUSHREG ",
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
            op::JZ | op::PUSH | op::JMP | op::JMPUSER => {
                let operand = self.load_i32(offset + 4)?;
                inst.push_str(&operand.to_string());
            },
            op::PUSHREG => {
                let operand = self.load_i32(offset + 4)?;
                match operand {
                    0 => inst.push_str("BP"),
                    1 => inst.push_str("LP"),
                    2 => inst.push_str("IP"),
                    3 => inst.push_str("SP"),
                    4 => inst.push_str("FP"),
                    5 => inst.push_str("FLAG"),
                    6 => inst.push_str("IVEC"),
                    _ => inst.push_str(&operand.to_string()),
                }
            },
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
