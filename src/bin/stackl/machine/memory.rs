use std::slice::SliceIndex;

#[derive(Debug)]
pub struct MachineMemory {
    ram: Vec<u8>,
}

impl MachineMemory {
    pub fn new(size: usize) -> Self {
        Self {
            ram: vec![0x79; size],
        }
    }

    #[inline]
    #[must_use]
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.ram.get(index)
    }
    pub fn set<I>(&mut self, index: I, value: &[u8]) -> bool
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        if let Some(slice) = self.ram.get_mut(index) {
            if slice.len() != value.len() {
                false
            } else {
                slice.copy_from_slice(value);
                true
            }
        } else {
            false
        }
    }
    pub fn len(&self) -> usize {
        self.ram.len()
    }
}
