mod iter;
pub mod lexer;

pub use iter::*;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar, "/bin/stackl-cc/analysis/lex/grammar.rs");

#[macro_export]
macro_rules! directive {
	($kind:expr , $name:literal , $lo:ident, $hi:ident) => {
		if matches!($kind, PPTokenKind::Ident(Ident{name, ..}) if name == $name) {
			Ok(())
		} else {
			let kind = $crate::diagnostics::DiagKind::InvalidToken;
			let span = $crate::diagnostics::Span{ loc: ($lo, $hi), file_id: usize::MAX };
			Err(lalr::ParseError::User {
				error: $crate::diagnostics::Diagnostic::error(kind, span)
			})
		}
	}
}

pub use grammar::TokensParser;
