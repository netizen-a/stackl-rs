use std::{fs, io::Read, path::PathBuf, process::ExitCode, rc::Rc};

mod cli;
mod diag;
mod lex;
mod sem;
mod syn;
mod tok;

use lex::grammar::TokensParser;
use syn::grammar::SyntaxParser;

fn main() -> ExitCode {
	let args = cli::Args::parse();
	let mut file_map = bimap::BiHashMap::<usize, PathBuf>::new();
	file_map.insert(0, args.in_file.clone());
	let mut file = fs::File::open(args.in_file).unwrap();
	let mut buffer = String::new();
	file.read_to_string(&mut buffer).unwrap();
	let lexer = lex::lexer::Lexer::new(buffer, 0);
	let pp_iter = lex::PPTokenIter::from(lexer);
	let pp_ref = Rc::clone(&pp_iter.stack_ref);
	let tokens: Vec<(usize, tok::Token, usize)> = TokensParser::new()
		.parse(&mut file_map, &pp_ref, pp_iter)
		.unwrap();

	let rev_tokens: Vec<(usize, tok::Token, usize)> = tokens.into_iter().rev().collect();
	let tok_iter = syn::TokenIter::from(rev_tokens);
	let _tok_ref = Rc::clone(&tok_iter.stack_ref);
	let unit = SyntaxParser::new().parse(tok_iter).unwrap();

	println!("{:#?}", unit);

	sem::SemanticParser::new().parse(unit);

	ExitCode::SUCCESS
}
