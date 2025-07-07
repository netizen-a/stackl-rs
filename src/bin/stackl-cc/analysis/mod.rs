mod lex;
mod sem;
mod syn;
mod tok;

use lex::grammar::TokensParser;
use std::io::Read;
use std::{
	fs,
	path::{Path, PathBuf},
	rc::Rc,
};
use syn::grammar::SyntaxParser;

pub fn parse<P>(in_file: P)
where
	P: AsRef<Path>,
{
	let mut file_map = bimap::BiHashMap::<usize, PathBuf>::new();
	file_map.insert(0, in_file.as_ref().to_owned());
	let mut file = fs::File::open(in_file.as_ref()).unwrap();
	let mut text = String::new();
	file.read_to_string(&mut text).unwrap();
	let lexer = lex::lexer::Lexer::new(text, 0);
	let pp_iter = lex::PPTokenIter::from(lexer);
	let pp_ref = Rc::clone(&pp_iter.stack_ref);
	let tokens: Vec<(usize, tok::Token, usize)> = TokensParser::new()
		.parse(&mut file_map, &pp_ref, pp_iter)
		.unwrap();

	let tok_iter = syn::TokenIter::from(tokens.into_boxed_slice());
	let _tok_ref = Rc::clone(&tok_iter.stack_ref);
	let mut unit = SyntaxParser::new().parse(tok_iter).unwrap();
	sem::SemanticParser::new().parse(&mut unit);
}
