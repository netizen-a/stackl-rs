use bitflags::bitflags;
use lalrpop_util::lalrpop_mod;

pub mod ast;
mod code_gen;
mod lex;
pub mod op;
mod sym;
pub mod tok;

lalrpop_mod! {
    #[allow(clippy::ptr_arg)]
    grammar
}

bitflags! {
    #[derive(Debug)]
    pub struct StacklFlags: u32 {
        const FEATURE_PIO_TERM = 1;
        const FEATURE_DMA_TERM = 1 << 1;
        const FEATURE_DISK     = 1 << 2;
        const FEATURE_INP      = 1 << 3;
        const _ = !0;
    }
}

#[derive(Debug)]
pub struct StacklFormatV1 {
    pub header: String,
    pub text: Vec<u8>,
}

#[derive(Debug)]
pub struct StacklFormatV2 {
    pub magic: [u8; 4],
    pub version: u32,
    pub flags: StacklFlags,
    pub stack_size: i32,
    pub text: Vec<u8>,
}

impl StacklFormatV2 {
    pub fn to_vec(self) -> Vec<u8> {
        let mut ret = Vec::from(self.magic);
        ret.extend(self.version.to_le_bytes());
        ret.extend(self.flags.bits().to_le_bytes());
        ret.extend(self.stack_size.to_le_bytes());
        ret.extend(self.text);
        ret
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedEof,
    InvalidVersion,
    InvalidMagic,
}

impl TryFrom<&[u8]> for StacklFormatV2 {
    type Error = ErrorKind;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 24 {
            return Err(ErrorKind::UnexpectedEof);
        }
        let magic: [u8; 4] = value[..4].try_into().unwrap();
        let version: u32 = u32::from_le_bytes(value[4..8].try_into().unwrap());
        let flags = u32::from_le_bytes(value[8..12].try_into().unwrap());
        let stack_size = i32::from_le_bytes(value[12..16].try_into().unwrap());

        if magic != [b's', b'l', 0, 0] {
            return Err(ErrorKind::InvalidMagic);
        }
        if version != 0 {
            return Err(ErrorKind::InvalidVersion);
        }

        Ok(StacklFormatV2 {
            magic,
            version,
            flags: StacklFlags::from_bits_retain(flags),
            stack_size,
            text: Vec::from(&value[16..]),
        })
    }
}
