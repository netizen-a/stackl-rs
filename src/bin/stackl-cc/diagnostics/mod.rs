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
		!self.list_syntax.is_empty()
	}
	pub fn print_diagnostics(&self) {
		for diag in self.list_syntax.iter() {
			self.print_recov(diag)
		}
		for diag in self.list_other.iter() {
			match diag.level {
				DiagLevel::Error => self.print_error(diag),
				DiagLevel::Warning => self.print_warning(diag),
			}
		}
	}
	fn print_error(&self, diag: &Diagnostic) {
		let file_path = self.file_map.get_by_left(&diag.span.file_id).unwrap();
		let file_name = file_path.to_string_lossy();
		let mut file = fs::File::open(file_path).unwrap();
		let mut source = String::new();
		file.read_to_string(&mut source);

        eprint!("\x1b[1;31merror:\x1b[0m ");
        match &diag.kind {
			DiagKind::InvalidRestrict => {
				eprintln!("\x1b[1;97mrestrict requires a pointer or reference\x1b[0m");
                let (line, col) = diag.span.location(&source).unwrap();
                let mut line_len = line.to_string().len();

                eprintln!("{}--> {file_name}:{line}:{col}", " ".repeat(line_len));

                line_len += 1;
                let line_space = " ".repeat(line_len);
                eprintln!("{line_space}|");
                if (line_len % 2) == 1 {
                    print!(" ");
                }
                for source_line in diag.span.to_string_vec(&source) {
                    eprintln!("{} |{}", line, source_line);
                    eprintln!(
                        "{line_space}|{}\x1b[1;31m{}\x1b[0m",
                        " ".repeat(col - 1),
                        "^".repeat(1 + diag.span.loc.1 - diag.span.loc.0)
                    );
                }
			}
            DiagKind::TypeError {
				found,
				expected,
			} => {
                eprintln!("mismatched types");
                let (line, col) = diag.span.location(&source).unwrap();
                let mut line_len = line.to_string().len();

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
                        "^".repeat(1 + diag.span.loc.1 - diag.span.loc.0)
                    );
                    eprintln!("expected `{expected}`, found `{found}`");
                }
            }
			_ => todo!()
        }
	}
	fn print_recov(&self, diag: &ErrorRecovery<usize, tok::Token, Diagnostic>) {

	}
	fn print_warning(&self, diag: &Diagnostic) {
		let file_path = self.file_map.get_by_left(&diag.span.file_id).unwrap();
		let file_name = file_path.to_string_lossy();
		let mut file = fs::File::open(file_path).unwrap();
		let mut source = String::new();
		file.read_to_string(&mut source);

        eprint!("\x1b[1;33mwarning:\x1b[0m ");
        match &diag.kind {
            DiagKind::DuplicateSpecifier(name) => {
                eprintln!("\x1b[1;97mduplicate '{name}' declaration specifier\x1b[0m");
                let (line, col) = diag.span.location(&source).unwrap();
                let mut line_len = line.to_string().len();

                eprintln!("{}--> {file_name}:{line}:{col}", " ".repeat(line_len));

                line_len += 1;
                let line_space = " ".repeat(line_len);
                eprintln!("{line_space}|");
                if (line_len % 2) == 1 {
                    print!(" ");
                }
                for source_line in diag.span.to_string_vec(&source) {
                    eprintln!("{} |{}", line, source_line);
                    eprintln!(
                        "{line_space}|{}\x1b[1;33m{}\x1b[0m",
                        " ".repeat(col - 1),
                        "^".repeat(1 + diag.span.loc.1 - diag.span.loc.0)
                    );
                }
            }
			_ => todo!()
        }
	}
}
