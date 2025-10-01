mod diag;
mod kind;
pub mod lex;
mod span;

use crate::analysis::{
	syn,
	tok,
};
use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	fs,
	io::{BufReader, Read},
	path::{Path, PathBuf},
	rc::Rc,
	result,
};

use lalrpop_util::ParseError;

pub use diag::*;
pub use kind::*;
pub use span::*;

pub type ResultTriple<Tok, Loc> = result::Result<(Loc, Tok, Loc), Diagnostic>;
pub type Result<T> = result::Result<T, Diagnostic>;

#[derive(Default)]
pub struct DiagnosticEngine {
	enable_color: bool,
	file_map_ref: Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>>,
	source_map: HashMap<usize, String>,
	list_other: Vec<Diagnostic>,
	syntax_errors: Vec<ParseError<usize, tok::Token, Diagnostic>>,
	preproc_errors: Vec<ParseError<usize, tok::PPToken, Diagnostic>>,
}

impl DiagnosticEngine {
	#[inline]
	pub fn new(enable_color: bool) -> Self {
		Self {
			enable_color,
			..Self::default()
		}
	}
	pub fn get_file_map(&self) -> Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>> {
		self.file_map_ref.clone()
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
	pub fn push_preproc_error(&mut self, diag: ParseError<usize, tok::PPToken, Diagnostic>) {
		self.preproc_errors.push(diag)
	}
	pub fn get_file_path(&self, id: usize) -> Option<PathBuf> {
		self.file_map_ref
			.borrow()
			.get_by_left(&id)
			.map(|p| p.clone())
	}
	pub fn id(&self) -> usize {
		self.file_map_ref.borrow().len()
	}
	pub fn get_file_id<P>(&self, name: &P) -> Option<usize>
	where
		P: AsRef<Path>,
	{
		self.file_map_ref
			.borrow()
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
		self.file_map_ref
			.borrow_mut()
			.insert(id, full_path.as_ref().to_path_buf());
		let file = fs::File::open(&full_path).unwrap();
		let mut reader = BufReader::new(file);
		let mut buf = String::new();
		reader.read_to_string(&mut buf).unwrap();
		self.source_map.insert(id, buf);
	}
	pub fn contains_error(&self) -> bool {
		for diag in self.list_other.iter() {
			if matches!(diag.level, DiagLevel::Error | DiagLevel::Fatal) {
				return true;
			}
		}
		!self.preproc_errors.is_empty() || !self.syntax_errors.is_empty()
	}
	pub fn print_diagnostics(&self) {
		for diag in self.preproc_errors.iter() {
			self.print_parse_errors(DiagLevel::Error, diag)
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
		T: ToSpan,
	{
		match &error {
			ParseError::ExtraToken { token } => {
				let span = token.1.to_span();
				let diag = Diagnostic {
					level,
					kind: DiagKind::ExtraToken,
					span,
					notes: vec![],
				};
				self.stderr_diagnostic(&diag);
			}
			ParseError::InvalidToken { .. } => unreachable!("invalid token"),
			ParseError::UnrecognizedEof { location, expected } => {
				let file_id = 0;
				let file_path = self
					.file_map_ref
					.borrow()
					.get_by_left(&file_id)
					.unwrap()
					.clone();
				let mut file = fs::File::open(file_path).unwrap();
				let mut source = String::new();
				let _ = file.read_to_string(&mut source);
				let span = Span {
					file_id,
					loc: (*location, *location),
					// TODO: get line from external source
					line: usize::MAX,
				};
				let diag = Diagnostic {
					level,
					kind: DiagKind::UnexpectedEof,
					span,
					notes: vec![],
				};
				let msg0 = "unexpected EOF";
				let mut msg1 = String::from("expected ");
				for (i, elem) in expected.iter().enumerate() {
					if i != 0 {
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
				let mut seen = HashSet::<String>::new();
				// in case duplicate ')' is found
				let mut pruned = expected.clone();
				pruned.retain(|c| {
					let is_first = !seen.contains(c);
					seen.insert(c.to_string());
					is_first
				});

				let diag = Diagnostic {
					level,
					kind: DiagKind::UnrecognizedToken { expected: pruned },
					span: token.1.to_span(),
					notes: vec![],
				};
				self.stderr_diagnostic(&diag);
			}
			ParseError::User { error } => self.stderr_diagnostic(error),
		}
	}
	fn stderr_diagnostic(&self, diag: &Diagnostic) {
		let str_diag = match &diag.kind {
			DiagKind::InvalidToken => {
				let msg0 = "invalid token";
				self.format_diagnostic(&diag, msg0, "consider don't ...")
			}
			DiagKind::InvalidRestrict => {
				let msg0 = "restrict requires a pointer or reference";
				self.format_diagnostic(&diag, msg0, "")
			}
			// DiagKind::TypeError { found, expected } => {
			// 	let msg0 = "mismatched types";
			// 	let msg1 = format!("expected `{expected}`, found `{found}`");
			// 	self.format_diagnostic(&diag, msg0, msg1.as_str())
			// }
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
			DiagKind::ArrayOfFunctions(ident) => {
				let msg0 =
					format!("'{ident}' declared as array of functions of type '<NOT IMPLEMENTED>'");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::UnrecognizedToken { expected } => {
				let msg0 = "unrecognized token";
				let mut msg1 = String::from("expected ");
				for (i, elem) in expected.iter().enumerate() {
					if i != 0 {
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
			DiagKind::FnRetFn(Some(name)) => {
				let msg0 = format!("'{name}' declared as function returning function");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::FnRetFn(None) => {
				let msg0 = format!("type name declared as function returning function");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::OmittedParamName => {
				let msg0 = "parameter name omitted";
				let msg1 = "ISO C does not support omitting parameter names in function definitions before C23";
				self.format_diagnostic(&diag, msg0, msg1)
			}
			DiagKind::DeclIdentList => {
				let msg0 = "parameter names (without types) in function declaration";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::InvalidStar => {
				let msg0 = "star modifier used outside of function prototype";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::UnboundVLA => {
				let msg0 = "variable length array must be bound in function definition";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::IfAssign => {
				let msg0 = "using the result of an assignment as a condition without parenthesis";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::OnlyVoid => {
				let msg0 = "'void' must be the only parameter and unnamed";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::ArrayOfVoid(None) => {
				let msg0 = "declaration of type name as array of voids";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::ArrayOfVoid(Some(name)) => {
				let msg0 = format!("declaration of '{name}' as array of voids");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::IllegalStorage(kind) => {
				let msg0 = format!("function definition declared '{kind}'");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::BitfieldRange(Some(name)) => {
				let msg0 = format!("width of bit-field '{name}' exceeds width of its type");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::BitfieldRange(None) => {
				let msg0 = format!("width of anonymous bit-field exceeds width of its type");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::BitfieldNonIntegral(Some(name)) => {
				let msg0 = format!("bit-field '{name}' has non-integral type");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::BitfieldNonIntegral(None) => {
				let msg0 = format!("anonymous bit-field has non-integral type");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::NonConstExpr => {
				let msg0 = "expression is not an integer constant expression";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::EnumNonIntegral(name) => {
				let msg0 =
					format!("enumerator value for '{name}' is not an integer constant expression");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::EnumRange => {
				let msg0 = "enumerator value is out of range";
				let msg1 = "ISO C restricts enumerator values to range of 'int' before C23";
				self.format_diagnostic(&diag, msg0, msg1)
			}
			DiagKind::ErrorDirective(err_msg) => {
				let msg0 = format!("{err_msg}");
				self.format_diagnostic(&diag, msg0.as_str(), "")
			}
			DiagKind::ArrayMaxRange => {
				// array range is 2 ^ 32 - 1 = 4,294,967,295
				let msg0 = "size of array exceeds maximum object size '4294967295'";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::ArrayMinRange => {
				let msg0 = "ISO C forbids zero-size array";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::DeclaratorLimit => {
				let msg0 =
					"declarators modifying a type in a declaration exceeds translation limit '12'";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::ParameterLimit => {
				let msg0 = "parameters in function definition exceeds translation limit '127'";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::UndefPredef => {
				let msg0 = "undefining builtin macro";
				self.format_diagnostic(&diag, msg0, "")
			}
			DiagKind::RedefPredef => {
				let msg0 = "redefining builtin macro";
				self.format_diagnostic(&diag, msg0, "")
			}
			kind => unimplemented!("{kind:?}"),
		};
		eprint!("{str_diag}");
	}
	fn format_diagnostic<S>(&self, diag: &Diagnostic, msg0: S, msg1: S) -> String
	where
		S: AsRef<str>,
	{
		let color_blue: &str = if self.enable_color { "\x1b[1;34m" } else { "" };
		let color_default: &str = if self.enable_color { "\x1b[0m" } else { "" };
		let color_bold_red: &str = if self.enable_color { "\x1b[1;31m" } else { "" };
		let color_bold_yellow: &str = if self.enable_color { "\x1b[1;33m" } else { "" };
		let color_bold_white: &str = if self.enable_color { "\x1b[1;97m" } else { "" };

		let mut result = String::new();
		let file_path = self.get_file_path(diag.span.file_id).unwrap();
		let source = self.get_file_data(diag.span.file_id).unwrap();
		let level_color = match diag.level {
			DiagLevel::Fatal => {
				result.push_str(&format!("{color_bold_red}fatal error:{color_default} "));
				color_bold_red
			}
			DiagLevel::Error => {
				result.push_str(&format!("{color_bold_red}error:{color_default} "));
				color_bold_red
			}
			DiagLevel::Warning => {
				result.push_str(&format!("{color_bold_yellow}warning:{color_default} "));
				color_bold_yellow
			}
		};

		result.push_str(&format!(
			"{color_bold_white}{}{color_default}\n",
			msg0.as_ref()
		));
		let mut line = diag.span.line;
		let col = diag.span.column(source.as_ref()).unwrap();
		let mut hi_line = line;
		let source_triple = diag.span.to_vec(source.as_ref());
		let triple_len = source_triple.len();
		hi_line += 1 - source_triple.len();

		let line_len = hi_line.to_string().len();
		let mut line_space = " ".repeat(line_len);

		result.push_str(&format!(
			"{line_space}{color_blue}-->{color_default} {}:{line}:{col}\n",
			file_path.display()
		));
		line_space.push(' ');
		result.push_str(&format!("{line_space}{color_blue}|{color_default}\n"));
		for (index, (lo, source_line, hi)) in source_triple.into_iter().enumerate() {
			for _ in 0..(line_len - line.to_string().len()) {
				result.push(' ');
			}
			result.push_str(&format!(
				"{color_blue}{line} |{color_default} {source_line}\n"
			));
			line += 1;
			if hi < lo {
				continue;
			}
			result.push_str(&format!(
				"{line_space}{color_blue}|{color_default} {}{level_color}{}{color_default}",
				" ".repeat(lo),
				"^".repeat(1 + hi - lo),
			));
			if index == triple_len - 1 {
				result.push_str(&format!(
					"{color_bold_red} {}{color_default}\n",
					msg1.as_ref()
				));
			} else {
				result.push('\n');
			}
		}
		for note in diag.notes.iter() {
			result.push_str(&format!("{color_blue}{line_space}|{color_default}\n"));
			result.push_str(&format!(
				"{color_blue}{line_space}= {color_bold_white}note:{color_default} {note}\n"
			));
		}
		result
	}
}
