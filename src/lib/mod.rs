use lalrpop_util::lalrpop_mod;
use tok::{LexicalError, Token};
use lalrpop_util::ErrorRecovery;

mod lex;
pub mod ast;
pub mod tok;

lalrpop_mod! {
    #[allow(clippy::ptr_arg)]
    grammar
}

pub fn parse_grammar(input: &str) -> Result<Vec<ast::Stmt>, Vec<ErrorRecovery<usize, Token, LexicalError>>> {
    let tokens = lex::Lexer::new(input);
    let mut errors = Vec::new();
    let ast = match grammar::ProgramParser::new().parse(&mut errors, tokens) {
        Ok(v) => v,
        Err(_) => return Err(errors),
    };
    if errors.is_empty() {
        Ok(ast)
    } else {
        Err(errors)
    }
}
