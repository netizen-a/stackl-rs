use std::sync;
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
            let count = val.len();
            unsafe {
                ram.as_mut_ptr()
                    .add(offset)
                    .copy_from_nonoverlapping(val.as_ptr(), count);
            }
            true
        } else {
            false
        }
    }
    pub fn load_i16(&self, offset: usize) -> Option<i16> {
        let mem = self.inner.read().unwrap();
        let bytes = &mem[offset..=(offset + 1)];
        bytes.try_into().map(i16::from_le_bytes).ok()
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
}
