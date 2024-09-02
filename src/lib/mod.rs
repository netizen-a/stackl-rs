use lalrpop_util::lalrpop_mod;

mod ast;
mod lex;
mod tok;

lalrpop_mod! { pub(crate) grammar }
