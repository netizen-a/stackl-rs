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
    pub fn store_slice(&self, val: &[u8], offset: usize) -> bool {
        let mut ram = self.inner.write().unwrap();
        if ram.len() > val.len() + offset {
            ram[offset..offset + val.len()].clone_from_slice(val);
            true
        } else {
            false
        }
    }
    pub fn load_i32(&self, offset: usize) -> Option<i32> {
        let mem = self.inner.read().unwrap();
        let bytes = &mem[offset..=(offset + 3)];
        bytes.try_into().map(i32::from_le_bytes).ok()
    }
    pub fn store_i32(&self, val: i32, offset: usize) -> bool {
        let bytes = i32::to_le_bytes(val);
        self.store_slice(&bytes, offset)
    }
    pub fn load_u8(&self, offset: usize) -> Option<u8> {
        let mem = self.inner.read().unwrap();
        mem.get(offset).copied()
    }
    pub fn print_str(&self, offset: usize) -> Result<(), MachineCheck> {
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
                Err(MachineCheck {
                    error: String::from("cannot print outside ram"),
                })
            }
            None => Err(MachineCheck {
                error: String::from("out of bounds"),
            }),
        }
    }
}

#[derive(Debug)]
pub struct MachineCheck {
    #[allow(dead_code)]
    pub error: String,
}
