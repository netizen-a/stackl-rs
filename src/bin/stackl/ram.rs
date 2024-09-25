use crate::chk;
use std::io::Write;
use std::{io, sync};
pub struct Memory {
    inner: sync::RwLock<Vec<u8>>,
}

impl Memory {
    pub fn new(mem_size: usize) -> Self {
        Memory {
            inner: sync::RwLock::new(vec![0x79; mem_size]),
        }
    }
    // returns true if success, else false
    pub fn store_slice(&self, val: &[u8], offset: usize) -> Result<(), chk::MachineCheck> {
        let mut ram = self.inner.write().unwrap();
        if ram.len() > val.len() + offset {
            ram[offset..offset + val.len()].clone_from_slice(val);
            Ok(())
        } else {
            Err(chk::MachineCheck::new(chk::MachineCode::IllegalAddr, "failed to write slice"))
        }
    }
    pub fn load_i32(&self, offset: usize) -> Result<i32, chk::MachineCheck> {
        let mem = self.inner.read().unwrap();
        let bytes = &mem[offset..=(offset + 3)];
        bytes
            .try_into()
            .map(i32::from_le_bytes)
            .or(Err(chk::MachineCheck::new(chk::MachineCode::IllegalAddr, "failed to load bytes")))
    }
    pub fn store_i32(&self, val: i32, offset: usize) -> Result<(), chk::MachineCheck> {
        let bytes = i32::to_le_bytes(val);
        self.store_slice(&bytes, offset)
    }
    pub fn load_u8(&self, offset: usize) -> Result<u8, chk::MachineCheck> {
        let mem = self.inner.read().unwrap();
        mem.get(offset).copied().ok_or(chk::MachineCheck::new(chk::MachineCode::IllegalAddr,"out of bounds"))
    }
    pub fn store_u8(&self, val: u8, offset: usize) -> Result<(), chk::MachineCheck> {
        let mut mem = self.inner.write().unwrap();
        if let Some(byte) = mem.get_mut(offset) {
            *byte = val;
            Ok(())
        } else {
            Err(chk::MachineCheck::new(chk::MachineCode::IllegalAddr,"failed to write byte"))
        }
    }
    pub fn print_str(&self, offset: usize) -> Result<(), chk::MachineCheck> {
        let mem = self.inner.read().unwrap();
        match mem.split_at_checked(offset) {
            Some((_, bytes)) => {
                let mut lock = io::stdout().lock();
                for chunk in bytes.utf8_chunks() {
                    for ch in chunk.valid().chars() {
                        if ch == '\0' {
                            io::stdout().flush().unwrap();
                            return Ok(());
                        }
                        write!(lock, "{ch}").unwrap();
                    }
                    for byte in chunk.invalid() {
                        write!(lock, "\\x{:02X}", byte).unwrap();
                    }
                }
                io::stdout().flush().unwrap();
                Err(chk::MachineCheck::new(chk::MachineCode::IllegalAddr, "cannot print outside ram"))
            }
            None => Err(chk::MachineCheck::new(chk::MachineCode::IllegalAddr, "out of bounds")),
        }
    }
}
