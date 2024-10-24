use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Status: u16 {
        const HALTED           = 1;
        const USR_MODE         = 1 << 1;
        const INT_MODE         = 1 << 2;
        const INT_DIS          = 1 << 3;
        const VMEM             = 1 << 4;
        const TRACE            = 1 << 5;
        const LEGACY_MODE      = 1 << 6;
        const FEATURE_GEN_IO   = 1 << 7;
        const FEATURE_PIO_TERM = 1 << 8;
        const FEATURE_DMA_TERM = 1 << 9;
        const FEATURE_DISK     = 1 << 10;
        const FEATURE_INP      = 1 << 11;
        const _                = !0;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct IntVec: u16 {
        const MACHINE_CHECK = 1;
        const TRAP          = 1 << 1;
        const I_PF          = 1 << 2;
        const I_ALL         = 0xFFFF;
        const _ = !0;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MachineFlags {
    pub status: Status,
    pub intvec: IntVec,
}

impl MachineFlags {
    pub const fn new() -> Self {
        Self {
            status: Status::empty(),
            intvec: IntVec::empty(),
        }
    }
    pub fn set_status(&mut self, flag: Status, value: bool) {
        self.status.set(flag, value);
    }
    pub fn get_status(&self, flag: Status) -> bool {
        self.status.contains(flag)
    }
    pub fn set_intvec(&mut self, flag: IntVec, value: bool) {
        self.intvec.set(flag, value);
    }
    pub fn get_intvec(&self, flag: IntVec) -> bool {
        self.intvec.contains(flag)
    }
    pub const fn as_u32(&self) -> u32 {
        let mut result: u32 = self.status.bits() as _;
        result |= (self.intvec.bits() as u32) << 16;
        result
    }
}

impl From<MachineFlags> for u32 {
    fn from(value: MachineFlags) -> Self {
        value.as_u32()
    }
}

impl From<u32> for MachineFlags {
    fn from(value: u32) -> Self {
        Self {
            status: Status::from_bits_retain(value as u16),
            intvec: IntVec::from_bits_retain((value >> 16) as u16),
        }
    }
}
