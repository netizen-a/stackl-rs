use std::io::Write;
use std::sync::mpsc::Sender;
use std::{ffi, io, thread, time};

use crate::flag::{MachineFlags, MetaFlags, Status, MachineCheck};
use stackl::{op, StacklFlags, StacklFormatV2};

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
    pub meta: MetaFlags,
}

impl MachineState {
    pub fn new(program: StacklFormatV2, mem_size: usize) -> MachineState {
        let sp_addr = if program.text.len() % 4 != 0 {
            program.text.len() + 4 - (program.text.len() % 4)
        } else {
            program.text.len()
        };
        let mut meta = MetaFlags::empty();
        if program.flags.contains(StacklFlags::LEGACY_MODE) {
            meta.set(MetaFlags::LEGACY_MODE, true);
        }
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

        let mut ram = vec![0x79; mem_size];
        ram[..program.text.len()].copy_from_slice(&program.text);

        MachineState {
            bp: 0,
            lp: mem_size.try_into().unwrap(),
            ip: 8,
            sp: sp_addr as i32,
            fp: 0,
            flag: MachineFlags::new(),
            ivec: 0,
            vmem: 0,
            ram,
            meta,
        }
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

    // returns true if success, else false
    // This function does not check alignment.
    pub fn store_slice(&mut self, val: &[u8], offset: i32) -> Result<(), MachineCheck> {
        let mem = &mut self.ram;
        let offset = i32_to_offset(offset)?;
        if let Some(ram) = mem.get_mut(offset..offset + val.len()) {
            ram.clone_from_slice(val);
            Ok(())
        } else {
            Err(MachineCheck::ILLEGAL_ADDR)
        }
    }
    pub fn load_cstr(&self, offset: i32) -> Result<&ffi::CStr, MachineCheck> {
        let offset = i32_to_offset(offset)?;
        let bytes = self
            .ram
            .get(offset..)
            .ok_or(MachineCheck::ILLEGAL_ADDR);
        let Ok(c_str) = ffi::CStr::from_bytes_until_nul(bytes?) else {
            return Err(MachineCheck::ILLEGAL_ADDR);
        };
        Ok(c_str)
    }
    pub fn load_abs_i32(&self, offset: i32) -> Result<i32, MachineCheck> {
        let mem = &self.ram;
        check_align(offset)?;
        let offset = i32_to_offset(offset)?;
        if let Some(mem) = mem.get(offset..=(offset + 3)) {
            mem.try_into()
                .map(i32::from_le_bytes)
                .or(Err(MachineCheck::ILLEGAL_ADDR))
        } else {
            Err(MachineCheck::ILLEGAL_ADDR)
        }
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
        let mem = &self.ram;
        let offset = if self.is_user() {
            offset + self.bp
        } else {
            offset
        };
        let offset = i32_to_offset(offset)?;
        mem.get(offset)
            .copied()
            .ok_or(MachineCheck::ILLEGAL_ADDR)
    }
    // This function does not check alignment
    pub fn store_u8(&mut self, val: u8, offset: i32) -> Result<(), MachineCheck> {
        let offset = if self.is_user() {
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
            Err(MachineCheck::ILLEGAL_ADDR)
        }
    }
    // This function does not check alignment
    pub fn print(&self, offset: i32) -> Result<(), MachineCheck> {
        let mem = &self.ram;
        let offset = if self.is_user() {
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
            Err(MachineCheck::ILLEGAL_ADDR)
        } else {
            Err(MachineCheck::ILLEGAL_ADDR)
        }
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
            op::PUSHCVARIND => "PUSHCVARIND",
            op::OUTS => "OUTS",
            op::INP => "INP",
            op::PUSHFP => "PUSHFP",
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
            op::PUSHVARIND => "PUSHVARIND",
            op::POPCVARIND => "POPCVARIND",
            op::POPVARIND => "POPVARIND",
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
            op::PUSHVAR
            | op::PUSHCVAR
            | op::POPVAR
            | op::POPREG
            | op::POPCVAR
            | op::POPARGS
            | op::JZ
            | op::PUSH
            | op::JMP
            | op::JMPUSER
            | op::ADJSP
            | op::CALL => {
                let operand = self.load_i32(offset + 4)?;
                inst.push_str(&operand.to_string());
            }
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
    pub fn exec_interrupt(&mut self) -> Result<(), MachineCheck> {
        let was_user = self.is_user();

        // Find highest priority pending interrupt
        let pending_interrupt = self.flag.intvec.iter().enumerate().next();

        let Some((vector, int_flag)) = pending_interrupt else {
            // no pending interrupts
            return Ok(());
        };

        // turn off pending bit for HW interrupts
        self.flag.intvec.set(int_flag, false);

        self.push_i32(self.sp)?;
        self.push_i32(self.flag.as_u32() as i32)?;
        self.push_i32(self.bp)?;
        self.push_i32(self.lp)?;
        self.push_i32(self.ip)?;
        self.push_i32(self.fp)?;

        self.fp = self.sp;

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
}

// Helper function to convert i32 to usize.
// This function will return Err if val is negative
fn i32_to_offset(val: i32) -> Result<usize, MachineCheck> {
    val.try_into()
        .or(Err(MachineCheck::ILLEGAL_ADDR))
}

fn check_align(offset: i32) -> Result<(), MachineCheck> {
    if offset % 4 != 0 {
        return Err(MachineCheck::ILLEGAL_ADDR);
    }
    Ok(())
}
