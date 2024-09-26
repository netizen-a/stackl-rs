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
    pub fn store_slice(&self, val: &[u8], offset: i32) -> Result<(), chk::MachineCheck> {
        let mut lock = self.inner.write().unwrap();
        let offset = i32_to_offset(offset)?;
        if let Some(ram) = lock.get_mut(offset..offset + val.len()) {
            ram.clone_from_slice(val);
            Ok(())
        } else {
            Err(chk::MachineCheck::new(
                chk::MachineCode::IllegalAddr,
                "failed to write slice",
            ))
        }
    }
    pub fn load_i32(&self, offset: i32) -> Result<i32, chk::MachineCheck> {
        let lock = self.inner.read().unwrap();
        let offset = i32_to_offset(offset)?;
        if let Some(mem) = lock.get(offset..=(offset + 3)) {
            mem.try_into()
                .map(i32::from_le_bytes)
                .or(Err(chk::MachineCheck::new(
                    chk::MachineCode::IllegalAddr,
                    "failed to load bytes",
                )))
        } else {
            Err(chk::MachineCheck::new(
                chk::MachineCode::IllegalAddr,
                "out of range",
            ))
        }
    }
    pub fn store_i32(&self, val: i32, offset: i32) -> Result<(), chk::MachineCheck> {
        let bytes = i32::to_le_bytes(val);
        self.store_slice(&bytes, offset)
    }
    pub fn load_u8(&self, offset: i32) -> Result<u8, chk::MachineCheck> {
        let mem = self.inner.read().unwrap();
        let offset = i32_to_offset(offset)?;
        mem.get(offset).copied().ok_or(chk::MachineCheck::new(
            chk::MachineCode::IllegalAddr,
            "out of bounds",
        ))
    }
    pub fn store_u8(&self, val: u8, offset: i32) -> Result<(), chk::MachineCheck> {
        let mut mem = self.inner.write().unwrap();
        let offset = i32_to_offset(offset)?;
        if let Some(byte) = mem.get_mut(offset) {
            *byte = val;
            Ok(())
        } else {
            Err(chk::MachineCheck::new(
                chk::MachineCode::IllegalAddr,
                "failed to write byte",
            ))
        }
    }
    pub fn print_str(&self, offset: i32) -> Result<(), chk::MachineCheck> {
        let lock = self.inner.read().unwrap();
        let offset = i32_to_offset(offset)?;
        if let Some(bytes) = lock.get(offset..) {
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
            Err(chk::MachineCheck::new(
                chk::MachineCode::IllegalAddr,
                "cannot print outside ram",
            ))
        } else {
            Err(chk::MachineCheck::new(
                chk::MachineCode::IllegalAddr,
                "out of bounds",
            ))
        }
    }
}

fn i32_to_offset(val: i32) -> Result<usize, chk::MachineCheck> {
    val.try_into().or(Err(chk::MachineCheck::new(
        chk::MachineCode::IllegalAddr,
        "Invalid Address",
    )))
}
