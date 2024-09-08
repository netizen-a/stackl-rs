use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod gen;
mod lex;
pub mod sym;
pub mod tok;

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
