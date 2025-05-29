use std::{cell::RefCell, fs, io::Read, path::PathBuf, process::ExitCode, rc::Rc};

// use ast::ExternalDeclaration;
// use cli::PreprocStdout;
// use tok::Token;

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
	let tokens = lex::grammar::TokensParser::new()
		.parse(&mut file_map, &stack_ref, queue)
		.unwrap();
	eprintln!("{tokens:?}");
	// if args.pp_stdout != PreprocStdout::Disabled {
	// 	let preproc_string = preproc.to_string(args.pp_stdout);
	// 	let guard = diagnostics.lexical_errors.lock().unwrap();
	// 	if guard.is_empty() {
	// 		print!("{}", preproc_string);
	// 	} else {
	// 		for error in guard.iter() {
	// 			print!("{:?}", error);
	// 		}
	// 	}
	// 	return ExitCode::SUCCESS;
	// }

	// let (snd_tok, _rcv_tok) = mpsc::channel::<Token>();
	// let (snd_syn, _rcv_syn) = mpsc::channel::<ExternalDeclaration>();
	// let syntax = syn::SyntaxParser::new(rcv_tok, &diagnostics);
	// thread::scope(|s| {
	// 	s.spawn(|| {
	// 		for token in preproc {
	// 			snd_tok.send(token).expect("failed to send token");
	// 		}
	// 	});
	// 	// s.spawn(|| {
	// 	// 	for external_declaration in syntax {
	// 	// 		snd_syn
	// 	// 			.send(external_declaration)
	// 	// 			.expect("failed to send external-declaration");
	// 	// 	}
	// 	// });
	// });

	ExitCode::SUCCESS
}
