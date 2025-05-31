use std::{fs, io::Read, path::PathBuf, process::ExitCode, rc::Rc};

mod cli;
mod diag;
mod lex;
mod sem;
mod syn;
mod tok;

fn main() -> ExitCode {
	let args = cli::Args::parse();
	// let diagnostics = diag::DiagnosticEngine::new();
	let mut file_map = bimap::BiHashMap::<usize, PathBuf>::new();
	file_map.insert(0, args.in_file.clone());
	let mut file = fs::File::open(args.in_file).unwrap();
	let mut buffer = String::new();
	file.read_to_string(&mut buffer).unwrap();
	let lexer = lex::lexer::Lexer::new(buffer, 0);
	let queue = lex::PPTokenIter::from(lexer);
	let stack_ref = Rc::clone(&queue.stack_ref);
	let tokens: Vec<(usize, tok::Token, usize)> = lex::grammar::TokensParser::new()
		.parse(&mut file_map, &stack_ref, queue)
		.unwrap();
	let tokens_triple: Vec<diag::syn::ResultTriple<tok::Token, usize>> =
		tokens.into_iter().map(Ok).collect();
	let unit = syn::grammar::TranslationUnitParser::new()
		.parse(tokens_triple)
		.unwrap();

	println!("{:#?}", unit);

	ExitCode::SUCCESS
}
