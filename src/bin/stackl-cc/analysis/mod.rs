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

pub fn parse<P>(in_file: P, diagnostics: &mut DiagnosticEngine) -> Option<Vec<ExternalDeclaration>>
where
	P: AsRef<Path>,
{
	let mut errors = Vec::new();
	diagnostics.file_map.insert(0, in_file.as_ref().to_owned());
	let mut file = fs::File::open(in_file.as_ref()).unwrap();
	let mut text = String::new();
	file.read_to_string(&mut text).unwrap();
	let lexer = lex::lexer::Lexer::new(text, 0);
	let pp_iter = lex::PPTokenIter::from(lexer);
	let pp_ref = Rc::clone(&pp_iter.stack_ref);
	let tokens: Vec<TokenTriple> = TokensParser::new()
		.parse(&mut diagnostics.file_map, &pp_ref, pp_iter)
		.unwrap();

	let tk_iter = syn::TokenIter::from(tokens.into_boxed_slice());
	let tk_ref = Rc::clone(&tk_iter.inner);
	let unit = SyntaxParser::new()
		.parse(&mut errors, &tk_ref, tk_iter)
		.unwrap();
	for err in errors {
		diagnostics.push_recov(err)
	}
	sem::SemanticParser::new(diagnostics).parse(unit)
}
