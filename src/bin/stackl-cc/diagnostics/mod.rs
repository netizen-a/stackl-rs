mod kind;
pub mod lex;
mod span;
mod diag;

use crate::analysis::tok;
use std::{fs, io::Read, path::PathBuf, result};

use lalrpop_util::ErrorRecovery;

pub use kind::*;
pub use span::*;
pub use diag::*;

pub type ResultTriple<Tok, Loc> = result::Result<(Loc, Tok, Loc), Diagnostic>;
pub type Result<T> = result::Result<T, Diagnostic>;

#[derive(Default)]
pub struct DiagnosticEngine {
	pub file_map: bimap::BiHashMap<usize, PathBuf>,
	list_other: Vec<Diagnostic>,
	list_syntax: Vec<ErrorRecovery<usize, tok::Token, Diagnostic>>,
}

impl DiagnosticEngine {
	#[inline]
	pub fn new() -> Self {
		Self::default()
	}
	#[inline]
	pub fn push(&mut self, diag: Diagnostic) {
		self.list_other.push(diag)
	}
	#[inline]
	pub fn push_recov(&mut self, diag: ErrorRecovery<usize, tok::Token, Diagnostic>) {
		self.list_syntax.push(diag)
	}
	pub fn contains_error(&self) -> bool {
		for diag in self.list_other.iter() {
			if let DiagLevel::Error = diag.level {
				return true;
			}
		}
		if !self.list_syntax.is_empty() {
			return true;
		}
		false
	}
	pub fn print_errors(&self) {
		for diag in self.list_other.iter() {
			if let DiagLevel::Error = diag.level {
				self.print_error(diag)
			}
		}
		for diag in self.list_syntax.iter() {
			self.print_recov(diag)
		}
	}
	pub fn print_warnings(&self) {
		for diag in self.list_other.iter() {
			if let DiagLevel::Warning = diag.level {
				eprintln!("warning: {:?}", diag.kind);
			}
		}
	}
	fn print_error(&self, diag: &Diagnostic) {
		let file_path = self.file_map.get_by_left(&diag.span.file_id).unwrap();
		let file_name = file_path.to_string_lossy();
		let mut file = fs::File::open(file_path).unwrap();
		let mut source = String::new();
		file.read_to_string(&mut source);

        eprint!("\x1b[0;31merror: ");
        match &diag.kind {
            DiagKind::TypeError {
				found,
				expected,
			} => {
                eprintln!("mismatched types\x1b[0m");
                let (line, col) = diag.span.location(&source).unwrap();
                let mut line_len = line.to_string().len();
                line_len += line_len % 2;

                eprintln!("{}--> {file_name}:{line}:{col}", " ".repeat(line_len));

                line_len += 1;
                let line_space = " ".repeat(line_len);
                eprintln!("{line_space}|");
                if (line_len % 2) == 1 {
                    print!(" ");
                }
                for source_line in diag.span.to_string_vec(&source) {
                    eprintln!("{} |{}", line, source_line);
                    eprint!(
                        "{line_space}|{}{} ",
                        " ".repeat(col - 1),
                        "^".repeat(diag.span.loc.1 - diag.span.loc.0)
                    );
                    eprintln!("expected `{expected}`, found `{found}`");
                }
            }
			_ => todo!()
        }
	}
	fn print_recov(&self, diag: &ErrorRecovery<usize, tok::Token, Diagnostic>) {

	}
}
