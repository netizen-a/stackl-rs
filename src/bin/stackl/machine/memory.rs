use std::slice::SliceIndex;

use super::flag::MachineCheck;

#[derive(Debug)]
pub struct MachineMemory {
    /// mapped addr: 0x0B000000
    gen_io: [u8; 16],
    /// mapped addr: 0x0C000000
    timer: [u8; 16],
    /// mapped addr: 0x0D000000
    disk: [u8; 16],
    /// mapped addr: 0x0E000000
    pio_term: [u8; 16],
    ram: Vec<u8>,
}

impl MachineMemory {
    pub fn new(size: usize) -> Self {
        Self {
            gen_io: [0; 16],
            timer: [0; 16],
            disk: [0; 16],
            pio_term: [0; 16],
            ram: vec![0x79; size],
        }
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Result<&I::Output, MachineCheck>
    where
        I: SliceIndex<[u8]>,
    {
        self.ram.get(index).ok_or(MachineCheck::ILLEGAL_ADDR)
    }
    pub fn set<I>(&mut self, index: I, value: &[u8]) -> Result<(), MachineCheck>
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        if let Some(slice) = self.ram.get_mut(index) {
            if slice.len() != value.len() {
                Err(MachineCheck::ILLEGAL_ADDR)
            } else {
                slice.copy_from_slice(value);
                Ok(())
            }
        } else {
            Err(MachineCheck::ILLEGAL_ADDR)
        }
    }
    pub fn len(&self) -> usize {
        self.ram.len()
    }
}
