use std::io::Write;
use std::sync::mpsc::Sender;
use std::{ffi, io, thread, time};

use crate::flag::{IntVec, MachineCheck, MachineFlags, MetaFlags, Status};
use stackl::{op, StacklFlags, StacklFormatV2};

pub mod step;
mod trace;

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
    pub last_trace: u8,
}

impl MachineState {
    pub fn new(mem_size: usize) -> MachineState {
        MachineState {
            bp: 0,
            lp: mem_size as i32,
            ip: 8,
            sp: 0,
            fp: 0,
            flag: MachineFlags::new(),
            ivec: 0,
            vmem: 0,
            ram: vec![0x79; mem_size],
            meta: MetaFlags::empty(),
            last_trace: 0,
        }
    }
    pub fn store_program(&mut self, program: StacklFormatV2, boot: bool, bp: i32) -> Result<(), MachineCheck> {
        let text_len = program.text.len();
        if boot {
            let sp_addr = if text_len % 4 != 0 {
                text_len + 4 - (text_len % 4)
            } else {
                text_len
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
            self.meta = meta;
            self.sp = (sp_addr + 4) as i32;
            self.fp = self.sp;
        }

        // put the stack size just above the text segment
        self.store_i32(program.stack_size, text_len as i32)?;

        let addr = if bp < 0 {
            self.bp
        } else {
            bp
        };

        // copy text segment to memory
        let offset = addr as usize;
        self.ram[offset..(text_len+offset)].copy_from_slice(&program.text);
        Ok(())
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
    pub fn machine_check(&mut self, value: MachineCheck)
    {
        self.flag.intvec.set(IntVec::MACHINE_CHECK, true);
        self.flag.check.set(value, true);
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
