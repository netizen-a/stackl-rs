use lalrpop_util::lalrpop_mod;

pub mod ast;
mod gen;
mod lex;
pub mod op;
mod sym;
mod tok;

lalrpop_mod! {
    #[allow(clippy::ptr_arg)]
    grammar
}

#[derive(Debug)]
pub struct StacklFormat {
    magic: [u8; 4],
    version: u32,
    /// Reserved. Must be set to zero.
    flags: u32,
    int_vec: i32,
    trap_vec: i32,
    pub text: Vec<u8>,
}

impl StacklFormat {
    pub fn to_vec(self) -> Vec<u8> {
        let mut ret = Vec::from(self.magic);
        ret.extend(self.version.to_le_bytes());
        ret.extend(self.flags.to_le_bytes());
        ret.extend(self.int_vec.to_le_bytes());
        ret.extend(self.trap_vec.to_le_bytes());
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

impl TryFrom<&[u8]> for StacklFormat {
    type Error = ErrorKind;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 24 {
            return Err(ErrorKind::UnexpectedEof);
        }
        let magic: [u8; 4] = value[..4].try_into().unwrap();
        let version: u32 = u32::from_le_bytes(value[4..8].try_into().unwrap());
        let flags = u32::from_le_bytes(value[8..12].try_into().unwrap());
        let int_vec = i32::from_le_bytes(value[12..16].try_into().unwrap());
        let trap_vec = i32::from_le_bytes(value[16..20].try_into().unwrap());

        if magic != [b's', b'l', 0, 0] {
            return Err(ErrorKind::InvalidMagic);
        }
        if version != 0 {
            return Err(ErrorKind::InvalidVersion);
        }

        Ok(StacklFormat {
            magic,
            version,
            flags,
            int_vec,
            trap_vec,
            text: Vec::from(&value[20..]),
        })
    }
}
