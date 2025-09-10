mod diag;
mod kind;
pub mod lex;
mod span;

use crate::analysis::tok;
use std::{
	fs,
	io::Read,
	path::{Path, PathBuf},
	result,
};

use lalrpop_util::ErrorRecovery;

pub use diag::*;
pub use kind::*;
pub use span::*;

pub type ResultTriple<Tok, Loc> = result::Result<(Loc, Tok, Loc), Diagnostic>;
pub type Result<T> = result::Result<T, Diagnostic>;

const BLUE: &str = "\x1b[1;34m";
const DEFAULT: &str = "\x1b[0m";
const BOLD_RED: &str = "\x1b[1;31m";
const BOLD_YELLOW: &str = "\x1b[1;33m";
const BOLD_WHITE: &str = "\x1b[1;97m";

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
			match &diag.kind {
				DiagKind::InvalidRestrict => {
					let msg = "restrict requires a pointer or reference";
					self.print_fmt_diag(&diag, msg, "");
				}
				DiagKind::TypeError { found, expected } => {
					let msg0 = "mismatched types";
					let msg1 = format!("expected `{expected}`, found `{found}`");
					self.print_fmt_diag(&diag, msg0, msg1.as_str());
				}
				DiagKind::MultStorageClasses => {
					let msg = "multiple storage classes in declaration specifiers";
					self.print_fmt_diag(&diag, msg, "");
				}
				DiagKind::DuplicateSpecifier(name) => {
					let msg0 = format!("duplicate '{name}' declaration specifier");
					self.print_fmt_diag(&diag, msg0.as_str(), "");
				}
				DiagKind::BothSpecifiers(name0, name1) => {
					let msg0 = format!("both '{name0}' and '{name1}' in declaration specifier");
					self.print_fmt_diag(&diag, msg0.as_str(), "");
				}
				_ => todo!(),
			}
		}
	}
	fn print_recov(&self, diag: &ErrorRecovery<usize, tok::Token, Diagnostic>) {}
	fn print_fmt_diag<S>(&self, diag: &Diagnostic, msg0: S, msg1: S)
	where
		S: AsRef<str>,
	{
		let file_path = self.file_map.get_by_left(&diag.span.file_id).unwrap();
		let level_color = match diag.level {
			DiagLevel::Error => {
				eprint!("{BOLD_RED}error:{DEFAULT} ");
				BOLD_RED
			}
			DiagLevel::Warning => {
				eprint!("{BOLD_YELLOW}warning:{DEFAULT} ");
				BOLD_YELLOW
			}
		};
		let file_name = file_path.to_string_lossy();
		let mut file = fs::File::open(file_path).unwrap();
		let mut source = String::new();
		file.read_to_string(&mut source);

		eprintln!("{BOLD_WHITE}{}{DEFAULT}", msg0.as_ref());
		let (line, col) = diag.span.location(source.as_ref()).unwrap();
		let mut line_len = line.to_string().len();
		eprintln!(
			"{}{BLUE}-->{DEFAULT} {}:{line}:{col}",
			" ".repeat(line_len),
			file_name.as_ref()
		);
		line_len += 1;
		let line_space = " ".repeat(line_len);
		eprintln!("{line_space}{BLUE}|{DEFAULT}");
		if (line_len % 2) == 1 {
			print!(" ");
		}
		for source_line in diag.span.to_string_vec(source.as_ref()) {
			eprintln!("{BLUE}{} |{DEFAULT}{}", line, source_line);
			eprintln!(
				"{line_space}{BLUE}|{DEFAULT}{}{level_color}{}\x1b[0m {}",
				" ".repeat(col - 1),
				"^".repeat(1 + diag.span.loc.1 - diag.span.loc.0),
				msg1.as_ref()
			);
		}
	}
}
