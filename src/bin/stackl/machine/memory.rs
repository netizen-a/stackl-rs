use super::flag::MachineCheck;
use std::ops::{Bound, RangeBounds};

#[derive(Debug)]
pub struct MachineMemory {
    /// mapped addr: 0x0B00_0000
    gen_io: [u8; 16],
    /// mapped addr: 0x0C00_0000
    timer: [u8; 16],
    /// mapped addr: 0x0D00_0000
    disk: [u8; 16],
    /// mapped addr: 0x0E00_0000
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
    pub fn get<I>(&self, index: I) -> Result<&[u8], MachineCheck>
    where
        I: RangeBounds<usize>,
    {
        let start: usize = match index.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(&i) => i,
            Bound::Excluded(_) => unreachable!(),
        };
        let slice = match index.end_bound() {
            Bound::Unbounded => self.ram.get(start..),
            Bound::Excluded(&i) => self.ram.get(start..i),
            Bound::Included(&i) => self.ram.get(start..=i),
        };
        slice.ok_or(MachineCheck::ILLEGAL_ADDR)
    }
    pub fn set<I>(&mut self, index: I, value: &[u8]) -> Result<(), MachineCheck>
    where
        I: RangeBounds<usize>,
    {
        let start: usize = match index.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(&i) => i,
            Bound::Excluded(_) => unreachable!(),
        };
        let slice = match index.end_bound() {
            Bound::Unbounded => self.ram.get_mut(start..),
            Bound::Excluded(&i) => self.ram.get_mut(start..i),
            Bound::Included(&i) => self.ram.get_mut(start..=i),
        };
        if let Some(slice) = slice {
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
