mod lex;
mod sem;
pub mod syn;
pub mod tok;

use lex::grammar::TokensParser;
use std::io::Read;
use std::{
	fs,
	path::{Path, PathBuf},
	rc::Rc,
};
use syn::grammar::SyntaxParser;

use crate::analysis::syn::ExternalDeclaration;
use crate::analysis::tok::TokenTriple;
use crate::diagnostics::DiagnosticEngine;
use lalrpop_util as lalr;

pub fn parse<P>(
	in_file: P,
	diagnostics: &mut DiagnosticEngine,
	is_traced: bool,
) -> Option<Vec<ExternalDeclaration>>
where
	P: AsRef<Path>,
{
	let mut syntax_errors = Vec::new();
	diagnostics.insert_file_info(0, in_file.as_ref().to_owned());
	let mut file = fs::File::open(in_file.as_ref()).unwrap();
	let mut text = String::new();
	file.read_to_string(&mut text).unwrap();
	let lexer = lex::lexer::Lexer::new(text, 0);
	let pp_iter = lex::PPTokenIter::from(lexer);
	let pp_ref = Rc::clone(&pp_iter.stack_ref);
	let tokens: Vec<TokenTriple> = match TokensParser::new().parse(diagnostics, &pp_ref, pp_iter) {
		Ok(tokens) => tokens,
		Err(error) => {
			diagnostics.push_fatal_error(error);
			return None;
		}
	};

	let tk_iter = syn::TokenIter::from(tokens.into_boxed_slice());
	let tk_ref = Rc::clone(&tk_iter.inner);
	let unit = SyntaxParser::new()
		.parse(&mut syntax_errors, &tk_ref, tk_iter)
		.unwrap();
	for error_recov in syntax_errors {
		diagnostics.push_syntax_error(error_recov.error)
	}
	sem::SemanticParser::new(diagnostics, is_traced).parse(unit)
}
