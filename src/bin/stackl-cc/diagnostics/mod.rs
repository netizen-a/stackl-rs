// Copyright (c) 2024-2026 Jonathan A. Thomason

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
	collections::{
		HashMap,
		HashSet,
	},
	fs,
	io::{
		self,
		BufReader,
		Read,
	},
	path::{
		Path,
		PathBuf,
	},
	process::exit,
	rc::Rc,
	result,
};

use lalrpop_util::ParseError;

pub use diag::*;
pub use kind::*;
pub use span::*;

pub type ResultTriple<Tok, Loc> = result::Result<(Loc, Tok, Loc), Diagnostic>;

#[derive(Default)]
pub struct DiagnosticEngine {
	enable_color: bool,
	file_map_ref: Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>>,
	source_map: HashMap<usize, String>,
	list_other: Vec<Diagnostic>,
	syntax_errors: Vec<ParseError<usize, tok::Token, Diagnostic>>,
	// this is for the ParseError 'unexpected EOF', which doesn't have a span.
	eof_span: Option<Span>,
}

impl DiagnosticEngine {
	#[inline]
	pub fn new(enable_color: bool) -> Self {
		Self {
			enable_color,
			..Self::default()
		}
	}
	pub fn set_eof_span<S: ToSpan>(&mut self, token: &S) {
		self.eof_span = Some(token.to_span());
	}
	pub fn get_file_map(&self) -> Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>> {
		self.file_map_ref.clone()
	}
	#[inline]
	pub fn push(&mut self, diagnostic: Diagnostic) {
		self.list_other.push(diagnostic)
	}
	#[inline]
	pub fn push_syntax_error(&mut self, diag: ParseError<usize, tok::Token, Diagnostic>) {
		self.syntax_errors.push(diag)
	}
	pub fn push_and_exit(&mut self, diagnostic: Diagnostic) -> ! {
		self.push(diagnostic);
		self.print_once();
		exit(1);
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
	pub fn get_file_data(&self, id: usize) -> Option<&str> {
		self.source_map.get(&id).map(|x| x.as_str())
	}
	/// This function returns (actual line, reported line, column)
	pub fn get_location(&self, span: &Span) -> Option<(usize, usize, usize)> {
		let source = self.get_file_data(span.file_id)?;
		let (line, col) = span.get_location(source)?;
		Some((line, span.line, col))
	}
	pub fn insert_file_info<P>(&mut self, id: usize, full_path: P) -> io::Result<&str>
	where
		P: AsRef<Path>,
	{
		self.file_map_ref
			.borrow_mut()
			.insert(id, full_path.as_ref().to_path_buf());
		let file = fs::File::open(&full_path)?;
		let mut reader = BufReader::new(file);
		let mut buf = String::new();
		reader.read_to_string(&mut buf).unwrap();
		self.source_map.insert(id, buf);
		Ok(self.source_map.get(&id).unwrap())
	}
	/// Must be called before print_once
	pub fn contains_error(&self) -> bool {
		for diag in self.list_other.iter() {
			if matches!(diag.level, DiagLevel::Error | DiagLevel::Fatal) {
				return true;
			}
		}
		!self.syntax_errors.is_empty()
	}
	/// consume and print the errors
	pub fn print_once(&mut self) {
		let parse_err_vec: Vec<_> = self.syntax_errors.drain(..).collect();
		let other_err_vec: Vec<_> = self.list_other.drain(..).collect();
		for mut diag in parse_err_vec {
			self.print_parse_errors(DiagLevel::Error, &mut diag)
		}
		for mut diag in other_err_vec {
			self.stderr_diagnostic(&mut diag)
		}
	}
	fn print_parse_errors(
		&self,
		level: DiagLevel,
		error: &mut ParseError<usize, tok::Token, Diagnostic>,
	) {
		match error {
			ParseError::ExtraToken { token } => {
				let span = token.1.to_span();
				let mut diag = Diagnostic {
					level,
					kind: DiagKind::ExtraToken,
					notes: vec![],
					span_list: vec![(span, String::new())],
				};
				self.stderr_diagnostic(&mut diag);
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
				let mut diag = Diagnostic {
					level,
					kind: DiagKind::UnexpectedEof,
					notes: vec![],
					span_list: self
						.eof_span
						.clone()
						.map(|span| vec![(span, String::new())])
						.unwrap_or_default(),
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
				diag.pop_first_msg(&msg1);
				let str_diag = self.format_diagnostic(&diag, msg0);
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

				let mut diag = Diagnostic {
					level,
					kind: DiagKind::UnrecognizedToken {
						token: format!("{:?}", token.1.kind),
						expected: pruned,
					},
					notes: vec![],
					span_list: vec![(token.1.to_span(), String::new())],
				};
				self.stderr_diagnostic(&mut diag);
			}
			ParseError::User { error } => self.stderr_diagnostic(error),
		}
	}
	fn stderr_diagnostic(&self, diag: &mut Diagnostic) {
		let str_diag = match &diag.kind {
			DiagKind::InvalidToken => {
				let msg0 = "invalid token";
				diag.pop_first_msg("consider don't ...");
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::InvalidRestrict => {
				let msg0 = "restrict requires a pointer or reference";
				self.format_diagnostic(&diag, msg0)
			}
			// DiagKind::TypeError { found, expected } => {
			// 	let msg0 = "mismatched types";
			// 	let msg1 = format!("expected `{expected}`, found `{found}`");
			// 	self.format_diagnostic(&diag, msg0, msg1.as_str())
			// }
			DiagKind::MultStorageClasses => {
				let msg0 = "multiple storage classes in declaration specifiers";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::DuplicateSpecifier(name) => {
				let msg0 = format!("duplicate '{name}' declaration specifier");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::BothSpecifiers(name0, name1) => {
				let msg0 = format!("both '{name0}' and '{name1}' in declaration specifier");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::MultipleTypes => {
				let msg0 = "two or more data types in declaration specifiers";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::TooLong => {
				let msg0 = "'long long long' is too long for stackl-cc";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::ImplicitInt(Some(ident)) => {
				let msg0 = format!("type defaults to 'int' in declaration of {ident}");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::ImplicitInt(None) => {
				let msg0 = format!("type defaults to 'int' in declaration of type name");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::ArrayOfFunctions {
				name: Some(name),
				dtype,
			} => {
				let msg0 = format!("'{name}' declared as array of functions of type '{dtype}'");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::ArrayOfFunctions { name: None, dtype } => {
				let msg0 = format!("type name declared as array of functions of type '{dtype}'");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::UnrecognizedToken { token, expected } => {
				let msg0 = format!("unrecognized token {token}");
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
				diag.pop_first_msg(&msg1);
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::FnRetFn(Some(name)) => {
				let msg0 = format!("'{name}' declared as function returning function");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::FnRetFn(None) => {
				let msg0 = format!("type name declared as function returning function");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::OmittedParamName => {
				let msg0 = "parameter name omitted";
				let msg1 = "ISO C does not support omitting parameter names in function definitions before C23";
				diag.pop_first_msg(msg1);
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::DeclIdentList => {
				let msg0 = "parameter names (without types) in function declaration";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::InvalidStar => {
				let msg0 = "star modifier used outside of function prototype";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::UnboundVLA => {
				let msg0 = "variable length array must be bound in function definition";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::IfAssign => {
				let msg0 = "using the result of an assignment as a condition without parenthesis";
				diag.push_note("place parentheses around the assignment to silence this warning");
				diag.push_note("use '==' to turn this assignment into an equality comparison");
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::OnlyVoid => {
				let msg0 = "'void' must be the only parameter and unnamed";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::ArrayOfVoid(None) => {
				let msg0 = "declaration of type name as array of voids";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::ArrayOfVoid(Some(name)) => {
				let msg0 = format!("declaration of '{name}' as array of voids");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::IllegalStorage(kind) => {
				let msg0 = format!("function definition declared '{kind}'");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::BitfieldRange(Some(name)) => {
				let msg0 = format!("width of bit-field '{name}' exceeds width of its type");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::BitfieldRange(None) => {
				let msg0 = format!("width of anonymous bit-field exceeds width of its type");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::BitfieldNonIntegral(Some(name)) => {
				let msg0 = format!("bit-field '{name}' has non-integral type");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::BitfieldNonIntegral(None) => {
				let msg0 = format!("anonymous bit-field has non-integral type");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::NonIntConstExpr => {
				let msg0 = "expression is not an integer constant expression";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::InitializerNotConst => {
				let msg0 = "initializer element is not a compile-time constant";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::EnumNonIntegral(name) => {
				let msg0 =
					format!("enumerator value for '{name}' is not an integer constant expression");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::EnumRange => {
				let msg0 = "enumerator value is out of range";
				let msg1 = "ISO C restricts enumerator values to range of 'int' before C23";
				diag.pop_first_msg(msg1);
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::ErrorDirective(err_msg) => {
				let msg0 = format!("{err_msg}");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::ArrayMaxRange => {
				// array range is 2 ^ 32 - 1 = 4,294,967,295
				let msg0 = "size of array exceeds maximum object size '4294967295'";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::ArrayMinRange => {
				let msg0 = "ISO C forbids zero-size array";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::DeclaratorLimit => {
				let msg0 =
					"declarators modifying a type in a declaration exceeds translation limit '12'";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::ParameterLimit => {
				let msg0 = "parameters in function definition exceeds translation limit '127'";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::UndefPredef => {
				let msg0 = "undefining builtin macro";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::RedefPredef => {
				let msg0 = "redefining builtin macro";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::DirectiveLineNotSimple => {
				let msg0 = "#line directive requires a simple digit sequence";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::DirectiveLineMinRange => {
				let msg0 = "ISO C forbids #line directive with zero argument";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::DirectiveLineMaxRange => {
				let msg0 = "#line directive requires a positive integer argument";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::DirectiveLineFilename => {
				let msg0 = "invalid filename for #line directive";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::DirectiveExtraTokens(directive) => {
				let msg0 = format!("extra tokens at end of {directive} directive");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::FileNotFound(file_path) => {
				let msg0 = format!(
					"cannot find {}: no such file or directory",
					file_path.display()
				);
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::StructNoNamedMembers => {
				let msg0 = "struct has no named members";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::Internal(err_msg) => {
				let msg0 = format!("internal compiler error: {err_msg}");
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::SymbolAlreadyExists(name, dtype) => {
				let msg0 = format!("redefinition of '{name}'");
				diag.pop_first_msg(&format!(
					"previous definition of `{name}` with type '{dtype}'"
				));
				self.format_diagnostic(&diag, msg0.as_str())
			}
			DiagKind::ArrayDeclIncomplete => {
				let msg0 = "definition of variable with array type needs an explicit size or an initializer";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::ArrayExcessElements => {
				let msg0 = "excess elements in array initializer";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::VlaInitList => {
				let msg0 = "variable-sized object may not be initialized";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::SymbolUndeclared { name, in_func } => {
				let context = if *in_func {
					"first use in this function"
				} else {
					"not in a function"
				};
				let msg0 = format!("'{name}' undeclared ({context})");
				if *in_func {
					diag.push_note(
						"each undeclared identifier is reported only once for each function it appears in",
					);
				}
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::LabeledDeclaration => {
				let msg0 =
					"a label can only be part of a statement and a declaration is not a statement";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::LabeledCompoundEnd => {
				let msg0 = "label at end of compound statement";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::PragmaCxLimitedRange => {
				let msg0 = "pragma 'CX_LIMITED_RANGE' is unsupported";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::PragmaIgnored => {
				let msg0 = "unrecognized pragma is ignored";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::CastError { from_type, to_type } => {
				let msg0 = "cast error";
				self.format_diagnostic(&diag, msg0)
			}
			DiagKind::Trace(trace) => self.format_diagnostic(&diag, trace),
			kind => unimplemented!("{kind:?}"),
		};
		eprint!("{str_diag}");
	}
	fn format_diagnostic<S>(&self, diag: &Diagnostic, msg0: S) -> String
	where
		S: AsRef<str>,
	{
		let color_default: &str = if self.enable_color { "\x1b[0m" } else { "" };
		let color_blue: &str = if self.enable_color { "\x1b[1;34m" } else { "" };
		let color_bold_red: &str = if self.enable_color { "\x1b[1;31m" } else { "" };
		let color_bold_yellow: &str = if self.enable_color { "\x1b[1;33m" } else { "" };
		let color_bold_white: &str = if self.enable_color { "\x1b[1;97m" } else { "" };
		let color_bold_cyan: &str = if self.enable_color { "\x1b[1;36m" } else { "" };

		let mut result = String::new();

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
			DiagLevel::Info => {
				result.push_str(&format!("{color_bold_cyan}info:{color_default} "));
				color_bold_cyan
			}
		};

		if diag.span_list.is_empty() {
			result.push_str(&format!(
				"{color_bold_white}{}{color_default}\n",
				msg0.as_ref()
			));
			return result;
		};

		let mut highest_len = 0;
		for (span, msg) in diag.span_list.iter() {
			let (_, mut reported_line, _) = self.get_location(&span).unwrap();
			let source = self.get_file_data(span.file_id).unwrap();
			let source_triple = span.to_vec(source.as_ref());
			reported_line += 1 - source_triple.len();
			let line_len = reported_line.to_string().len();
			highest_len = std::cmp::max(highest_len, line_len);
		}

		result.push_str(&format!(
			"{color_bold_white}{}{color_default}\n",
			msg0.as_ref()
		));

		let mut line_space = " ".repeat(highest_len);
		let mut last_line = 1;
		for (span_index, (span, msg1)) in diag.span_list.iter().enumerate() {
			let file_name = self.get_file_path(span.name_id).unwrap();
			let source = self.get_file_data(span.file_id).unwrap();

			let (_, mut line, col) = self.get_location(&span).unwrap();
			let source_triple = span.to_vec(source.as_ref());
			let triple_len = source_triple.len();
			let line_len = highest_len;

			if span_index == 0 {
				result.push_str(&format!(
					"{line_space}{color_blue}-->{color_default} {}:{line}:{col}\n",
					file_name.display()
				));
				line_space.push(' ');
				result.push_str(&format!("{line_space}{color_blue}|{color_default}\n"));
				last_line = line;
			} else if last_line + 1 != line {
				result.push_str(&format!("{color_blue}...{color_default}\n"));
			}
			for (triple_index, (lo, source_line, hi)) in source_triple.into_iter().enumerate() {
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
				if triple_index == triple_len - 1 {
					result.push_str(&format!("{level_color} {}{color_default}\n", msg1));
				} else {
					result.push('\n');
				}
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
