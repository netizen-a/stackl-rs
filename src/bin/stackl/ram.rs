use crate::chk;
use std::io;
use std::io::Write;
pub struct Memory {
    inner: Vec<u8>,
}

impl Memory {
    pub fn new(mem_size: usize) -> Self {
        Memory {
            inner: vec![0x79; mem_size],
        }
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
    pub fn print_c_str(&self, offset: i32) -> Result<(), chk::MachineCheck> {
        let mem = &self.inner;
        let offset = i32_to_offset(offset)?;
        if let Some(bytes) = mem.get(offset..) {
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
            Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
        } else {
            Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr))
        }
    }
}

// Helper function to convert i32 to usize.
// This function will return Err if val is negative
fn i32_to_offset(val: i32) -> Result<usize, chk::MachineCheck> {
    val.try_into()
        .or(Err(chk::MachineCheck::from(chk::CheckKind::IllegalAddr)))
}
