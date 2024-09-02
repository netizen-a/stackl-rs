use logos::{Lexer, Logos};
use std::fmt;
use std::num::ParseIntError;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    #[default]
    InvalidToken,
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

fn str_callback(lex: &mut Lexer<Token>) -> Option<String> {
    let slice = lex.slice();
    let s = slice[1..slice.len() - 1]
        .replace(r"\'", "'")
        .replace("\\\"", "\"")
        .replace(r"\`", "`")
        .replace(r"\?", "?")
        .replace(r"\a", "\x07")
        .replace(r"\b", "\x08")
        .replace(r"\t", "\t")
        .replace(r"\n", "\n")
        .replace(r"\v", "\x0b")
        .replace(r"\f", "\x0c")
        .replace(r"\r", "\r")
        .replace(r"\e", "\x1b");
    Some(s.to_string())
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t]+", skip r";(.(\\\n)?)+", error = LexicalError)]
pub enum Token {
    #[token("\n")]
    Newline,
    #[token("\\")]
    BackSlash,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("[")]
    DirectiveStart,
    #[token("]")]
    DirectiveEnd,
    #[regex("[_a-zA-Z.?$][_0-9a-zA-Z.?$]*", |lex| lex.slice().to_string())]
    Identifier(String),
    #[regex("-?[1-9][0-9]*", |lex| lex.slice().parse())]
    Integer(i32),
    #[regex("'[^'\n]*'", str_callback)]
    #[regex("`[^`\n]*`", str_callback)]
    #[regex("\"[^\"\n]*\"", str_callback)]
    String(String),
    // Opcodes
    #[token("NOP", ignore(ascii_case))]
    OpNop,
    #[token("PLUS", ignore(ascii_case))]
    OpPlus,
    #[token("MINUS", ignore(ascii_case))]
    OpMinus,
    #[token("TIMES", ignore(ascii_case))]
    OpTimes,
    #[token("DIVIDE", ignore(ascii_case))]
    OpDivide,
    #[token("MOD", ignore(ascii_case))]
    OpMod,
    #[token("EQ", ignore(ascii_case))]
    OpEq,
    #[token("NE", ignore(ascii_case))]
    OpNe,
    #[token("GT", ignore(ascii_case))]
    OpGt,
    #[token("LT", ignore(ascii_case))]
    OpLt,
    #[token("GE", ignore(ascii_case))]
    OpGe,
    #[token("LE", ignore(ascii_case))]
    OpLe,
    #[token("AND", ignore(ascii_case))]
    OpAnd,
    #[token("OR", ignore(ascii_case))]
    OpOr,
    #[token("NOT", ignore(ascii_case))]
    OpNot,
    #[token("SWAP", ignore(ascii_case))]
    OpSwap,
    #[token("DUP", ignore(ascii_case))]
    OpDup,
    #[token("HALT", ignore(ascii_case))]
    OpHalt,
    #[token("POP", ignore(ascii_case))]
    OpPop,
    #[token("RETURN", ignore(ascii_case))]
    OpReturn,
    #[token("RETURNV", ignore(ascii_case))]
    OpReturnv,
    #[token("NEG", ignore(ascii_case))]
    OpNeg,
    #[token("PUSHCVARIND", ignore(ascii_case))]
    OpPushCVarInd,
    #[token("OUTS", ignore(ascii_case))]
    OpOuts,
    #[token("INP", ignore(ascii_case))]
    OpInp,
    #[token("PUSHFP", ignore(ascii_case))]
    OpPushFP,
    #[token("JMPUSER", ignore(ascii_case))]
    OpJmpUser,
    #[token("TRAP", ignore(ascii_case))]
    OpTrap,
    #[token("RTI", ignore(ascii_case))]
    OpRti,
    #[token("CALLI", ignore(ascii_case))]
    OpCalli,
    #[token("PUSHREG", ignore(ascii_case))]
    OpPushReg,
    #[token("POPREG", ignore(ascii_case))]
    OpPopReg,
    #[token("BAND", ignore(ascii_case))]
    OpBAnd,
    #[token("BOR", ignore(ascii_case))]
    OpBOr,
    #[token("BXOR", ignore(ascii_case))]
    OpBXOr,
    #[token("SHIFTL", ignore(ascii_case))]
    OpShiftl,
    #[token("SHIFTR", ignore(ascii_case))]
    OpShiftr,
    #[token("PUSHVARIND", ignore(ascii_case))]
    OpPushVarInd,
    #[token("POPCVARIND", ignore(ascii_case))]
    OpPopCVarInd,
    #[token("POPVARIND", ignore(ascii_case))]
    OpPopVarInd,
    #[token("COMP", ignore(ascii_case))]
    OpComp,
    #[token("PUSH", ignore(ascii_case))]
    OpPush,
    #[token("JUMP", ignore(ascii_case))]
    OpJump,
    #[token("JUMPE", ignore(ascii_case))]
    OpJumpe,
    #[token("PUSHVAR", ignore(ascii_case))]
    OpPushVar,
    #[token("POPVAR", ignore(ascii_case))]
    OpPopVar,
    #[token("ADJSP", ignore(ascii_case))]
    OpAdjSP,
    #[token("POPARGS", ignore(ascii_case))]
    OpPopArgs,
    #[token("CALL", ignore(ascii_case))]
    OpCall,
    #[token("PUSHCVAR", ignore(ascii_case))]
    OpPushCVar,
    #[token("POPCVAR", ignore(ascii_case))]
    OpPopCVar,
    #[token("TRACE_ON", ignore(ascii_case))]
    OpTraceOn,
    #[token("TRACE_OFF", ignore(ascii_case))]
    OpTraceOff,
    #[token("CLID", ignore(ascii_case))]
    OpCLID,
    #[token("SEID", ignore(ascii_case))]
    OpSEID,
    #[token("ILLEGAL", ignore(ascii_case))]
    OpIllegal,
    // Pseudo Opcodes
    #[token("DB", ignore(ascii_case))]
    OpDB,
    #[token("DD", ignore(ascii_case))]
    OpDD,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
