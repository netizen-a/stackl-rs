mod iter;
pub mod lexer;

pub use iter::*;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar, "/bin/stackl-cc/lex/grammar.rs");

#[macro_export]
macro_rules! directive {
	($kind:expr , $name:literal , $lo:ident, $hi:ident) => {
		if matches!($kind, PPTokenKind::Ident(Ident{name, ..}) if name == $name) {
			Ok(())
		} else {
			Err(lalr::ParseError::User {
				error: diag::Error{
					kind: diag::ErrorKind::InvalidToken,
					loc: ($lo, $hi)
				}
			})
		}
	}
}
