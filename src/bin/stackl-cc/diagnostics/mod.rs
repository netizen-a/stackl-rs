mod diag;
mod kind;
pub mod lex;
mod span;

use crate::analysis::tok::{self, file_id::FileId};
use std::{
	collections::HashMap,
	fs,
	io::{BufReader, Read},
	path::{Path, PathBuf},
	result,
};

use lalrpop_util::ErrorRecovery;
use lalrpop_util::ParseError;

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
	file_map: bimap::BiHashMap<usize, PathBuf>,
	source_map: HashMap<usize, String>,
	list_other: Vec<Diagnostic>,
	syntax_errors: Vec<ParseError<usize, tok::Token, Diagnostic>>,
	fatal_errors: Vec<ParseError<usize, tok::PPToken, Diagnostic>>,
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
	pub fn push_syntax_error(&mut self, diag: ParseError<usize, tok::Token, Diagnostic>) {
		self.syntax_errors.push(diag)
	}
	#[inline]
	pub fn push_fatal_error(&mut self, diag: ParseError<usize, tok::PPToken, Diagnostic>) {
		self.fatal_errors.push(diag)
	}
	pub fn get_file_path(&self, id: usize) -> Option<&PathBuf> {
		self.file_map.get_by_left(&id)
	}
	pub fn id(&self) -> usize {
		self.file_map.len()
	}
	pub fn get_file_id<P>(&self, name: &P) -> Option<usize>
	where
		P: AsRef<Path>,
	{
		self.file_map
			.get_by_right::<Path>(name.as_ref())
			.map(|p| *p)
	}
	pub fn get_file_data(&self, id: usize) -> Option<&String> {
		self.source_map.get(&id)
	}
	pub fn insert_file_info<P>(&mut self, id: usize, full_path: P)
	where
		P: AsRef<Path>,
	{
		self.file_map.insert(id, full_path.as_ref().to_path_buf());
		let file = fs::File::open(&full_path).unwrap();
		let mut reader = BufReader::new(file);
		let mut buf = String::new();
		reader.read_to_string(&mut buf).unwrap();
		self.source_map.insert(id, buf);
	}
	pub fn contains_error(&self) -> bool {
		for diag in self.list_other.iter() {
			if matches!(
				diag.level,
				DiagLevel::Error | DiagLevel::Fatal | DiagLevel::Internal
			) {
				return true;
			}
		}
		!self.fatal_errors.is_empty() || !self.syntax_errors.is_empty()
	}
	pub fn print_diagnostics(&self) {
		for diag in self.fatal_errors.iter() {
			self.print_parse_errors(DiagLevel::Fatal, diag)
		}
		for diag in self.syntax_errors.iter() {
			self.print_parse_errors(DiagLevel::Error, diag)
		}
		for diag in self.list_other.iter() {
			self.stderr_diagnostic(diag)
		}
	}
	fn print_parse_errors<T>(&self, level: DiagLevel, error: &ParseError<usize, T, Diagnostic>)
	where
		T: FileId,
	{
		match &error {
			ParseError::ExtraToken { token } => {
				let span = Span {
					file_id: token.1.file_id(),
					loc: (token.0, token.2),
				};
				let diag = Diagnostic {
					level,
					kind: DiagKind::ExtraToken,
					span,
				};
				self.stderr_diagnostic(&diag);
			}
			ParseError::InvalidToken { location } => unreachable!("invalid token"),
			ParseError::UnrecognizedEof { location, expected } => {
				let file_id = 0;
				let file_path = self.file_map.get_by_left(&file_id).unwrap();
				let mut file = fs::File::open(file_path).unwrap();
				let mut source = String::new();
				file.read_to_string(&mut source);
				let span = Span {
					file_id,
					loc: (*location, *location),
				};
				let diag = Diagnostic {
					level,
					kind: DiagKind::UnexpectedEof,
					span,
				};
				let msg0 = "unexpected EOF";
				let mut msg1 = String::from("expected ");
				let mut is_first = true;
				for (i, elem) in expected.iter().enumerate() {
					if is_first {
						is_first = false;
					} else {
						msg1.push(',');
					}
					if i >= 4 {
						msg1.push_str(" ...");
						break;
					} else {
						msg1.push_str(&format!(" {elem}"));
					}
				}
				let str_diag = self.format_diagnostic(&diag, msg0, &msg1);
				eprint!("{str_diag}");
			}
			ParseError::UnrecognizedToken { token, expected } => {
				let span = Span {
					file_id: token.1.file_id(),
					loc: (token.0, token.2),
				};
				let diag = Diagnostic {
					level,
					kind: DiagKind::UnrecognizedToken {
						expected: expected.clone(),
					},
					span,
				};
				self.stderr_diagnostic(&diag);
			}
			ParseError::User { error } => self.stderr_diagnostic(error),
		}
	}
	fn stderr_diagnostic(&self, diag: &Diagnostic) {
		let str_diag = match &diag.kind {
			DiagKind::InvalidRestrict => {
				let msg0 = "restrict requires a pointer or reference";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::TypeError { found, expected } => {
				let msg0 = "mismatched types";
				let msg1 = format!("expected `{expected}`, found `{found}`");
				self.format_diagnostic(&diag, msg0, msg1.as_str())
			}
			DiagKind::MultStorageClasses => {
				let msg0 = "multiple storage classes in declaration specifiers";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::DuplicateSpecifier(name) => {
				let msg0 = format!("duplicate '{name}' declaration specifier");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::BothSpecifiers(name0, name1) => {
				let msg0 = format!("both '{name0}' and '{name1}' in declaration specifier");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::MultipleTypes => {
				let msg0 = "two or more data types in declaration specifiers";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::TooLong => {
				let msg0 = "'long long long' is too long for stackl-cc";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::ImplicitInt(ident) => {
				let msg0 = format!("type defaults to 'int' in declaration of {ident}");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::Internal(msg) => {
				format!("{BOLD_RED}internal error: {BOLD_WHITE}{msg}{DEFAULT}")
			}
			DiagKind::ArrayOfFunctions(ident) => {
				let msg0 =
					format!("'{ident}' declared as array of functions of type '<NOT IMPLEMENTED>'");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::UnrecognizedToken { expected } => {
				let msg0 = "unrecognized token";
				let mut msg1 = String::from("expected ");
				let mut is_first = true;
				for (i, elem) in expected.iter().enumerate() {
					if is_first {
						is_first = false;
					} else {
						msg1.push_str(", ");
						if i > 3 {
							msg1.push_str("...");
							break;
						} else if i == expected.len() - 1 {
							msg1.push_str("or ");
						}
					}

					match elem.as_str() {
						"IDENTIFIER" => msg1.push_str("identifier"),
						"TYPE_NAME" => msg1.push_str("type-name"),
						"CONSTANT" => msg1.push_str("constant"),
						"STRING_LITERAL" => msg1.push_str("string-literal"),
						elem_str => msg1.push_str(elem_str),
					}
				}
				self.format_diagnostic(&diag, msg0, &msg1)
			}
			DiagKind::FnRetFn(name) => {
				let msg0 = format!("'{name}' declared as function returning function");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::OmittedParamName => {
				let msg0 = "parameter name omitted";
				let msg1 = "ISO C does not support omitting parameter names in function definitions before C23";
				self.format_diagnostic(&diag, msg0, msg1)
			}
			_ => unimplemented!(),
		};
		eprint!("{str_diag}");
	}
	fn format_diagnostic<S>(&self, diag: &Diagnostic, msg0: S, msg1: S) -> String
	where
		S: AsRef<str>,
	{
		let mut result = String::new();
		let file_path = self.get_file_path(diag.span.file_id()).unwrap();
		let source = self.get_file_data(diag.span.file_id()).unwrap();
		let level_color = match diag.level {
			DiagLevel::Internal => {
				result.push_str(&format!("{BOLD_RED}internal error:{DEFAULT} "));
				BOLD_RED
			}
			DiagLevel::Fatal => {
				result.push_str(&format!("{BOLD_RED}fatal error:{DEFAULT} "));
				BOLD_RED
			}
			DiagLevel::Error => {
				result.push_str(&format!("{BOLD_RED}error:{DEFAULT} "));
				BOLD_RED
			}
			DiagLevel::Warning => {
				result.push_str(&format!("{BOLD_YELLOW}warning:{DEFAULT} "));
				BOLD_YELLOW
			}
		};

		result.push_str(&format!("{BOLD_WHITE}{}{DEFAULT}\n", msg0.as_ref()));
		let (line, col) = diag.span.location(source.as_ref()).unwrap();
		let mut line_len = line.to_string().len();
		result.push_str(&format!(
			"{}{BLUE}-->{DEFAULT} {}:{line}:{col}\n",
			" ".repeat(line_len),
			file_path.display()
		));
		line_len += 1;
		let line_space = " ".repeat(line_len);
		result.push_str(&format!("{line_space}{BLUE}|{DEFAULT}\n"));
		for source_line in diag.span.to_string_vec(source.as_ref()) {
			result.push_str(&format!("{BLUE}{line} |{DEFAULT} {source_line}\n"));
			result.push_str(&format!(
				"{line_space}{BLUE}|{DEFAULT} {}{level_color}{}{BOLD_RED} {}{DEFAULT}\n",
				" ".repeat(col - 1),
				"^".repeat(1 + diag.span.loc.1 - diag.span.loc.0),
				msg1.as_ref()
			));
		}
		result
	}
}
