use lalrpop_util::lalrpop_mod;

pub mod ast;
mod gen;
mod lex;
mod sym;
mod tok;

lalrpop_mod! {
    #[allow(clippy::ptr_arg)]
    grammar
}

#[derive(Debug)]
pub struct StacklFormat {
    pub magic: [u8; 4],
    pub version: u32,
    /// Must be set to zero.
    _reserved: u32,
    pub text: Vec<u8>,
}

impl From<StacklFormat> for Vec<u8> {
    fn from(value: StacklFormat) -> Self {
        let mut ret = Vec::from(value.magic);
        ret.extend(&value.version.to_le_bytes());
        ret.extend(value._reserved.to_le_bytes());
        ret.extend((value.text.len() as u32).to_le_bytes());
        ret.extend(value.text);
        ret
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedEof,
    InvalidVersion,
    InvalidMagic,
}

impl TryFrom<&[u8]> for StacklFormat {
    type Error = ErrorKind;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 16 {
            return Err(ErrorKind::UnexpectedEof);
        }
        let magic: [u8; 4] = value[..=3].try_into().unwrap();
        let version: u32 = u32::from_le_bytes(value[4..=7].try_into().unwrap());
        let _reserved = u32::from_le_bytes(value[8..=11].try_into().unwrap());
        let text_size = u32::from_le_bytes(value[12..=15].try_into().unwrap());

        if magic != [b's', b'l', 0, 0] {
            return Err(ErrorKind::InvalidMagic);
        }
        if version != 0 {
            return Err(ErrorKind::InvalidVersion);
        }

        let mut text: Vec<u8> = Vec::with_capacity(text_size as _);
        text.extend_from_slice(&value[16..]);
        Ok(StacklFormat {
            magic,
            version,
            _reserved,
            text,
        })
    }
}
