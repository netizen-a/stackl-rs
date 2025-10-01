use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc, time};

use crate::{
	analysis::tok::{self, Directive, PPToken, PPTokenKind, PPTokenTriple},
	diagnostics::DiagnosticEngine,
};

use super::lexer::Lexer;
use crate::diagnostics as diag;
use chrono::{Datelike, Timelike};
use lalrpop_util as lalr;
use crate::diagnostics::ToSpan;

pub enum StackKind {
	Buffer(Vec<diag::ResultTriple<PPToken, usize>>),
	Lexer(Lexer),
}

static MON_NAME: [&str; 12] = [
	"Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn current_date() -> String {
	let mut result = String::new();
	let utc = chrono::Local::now();
	let (day, month, year) = (utc.day(), utc.month(), utc.year());
	result.push_str(MON_NAME[(month - 1) as usize]);
	result.push_str(" ");
	if day < 10 {
		result.push_str(" ");
	}
	result.push_str(&format!("{day}"));
	result.push_str(" ");
	result.push_str(&format!("{year}"));

	return result;
}

fn current_time() -> String {
	let utc = chrono::Local::now();
	let (hour, min, sec) = (utc.hour(), utc.minute(), utc.second());
	format!("{hour:02}:{min:02}:{sec:02}")
}

#[derive(Default)]
pub struct PPTokenStack {
	stack: Vec<StackKind>,
	defines: HashMap<String, Vec<PPTokenTriple>>,
	file_map_ref: Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>>,
	line: usize,
	last_token_kind: Option<PPTokenKind>,
}

impl PPTokenStack {
	fn new(value: Lexer, file_map_ref: Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>>) -> Self {
		Self {
			stack: vec![StackKind::Lexer(value)],
			defines: HashMap::new(),
			file_map_ref,
			line: 1,
			last_token_kind: None,
		}
	}
	pub fn define_obj_macro(
		&mut self,
		name: String,
		replacement_list: Vec<PPTokenTriple>,
		span: diag::Span,
	) -> Option<diag::Diagnostic> {
		let error = diag::Diagnostic::error(diag::DiagKind::RedefPredef, span);
		match name.as_str() {
			"__DATE__"
			| "__FILE__"
			| "__LINE__"
			| "__STDC__"
			| "__STDC_HOSTED__"
			| "__STDC_MB_MIGHT_NEQ_WC__"
			| "__STDC_VERSION__"
			| "__TIME__"
			| "__STDC_IEC_559__"
			| "__STDC_IEC_559_COMPLEX__"
			| "__STDC_ISO_10646__" => Some(error),
			_ => {
				self.defines.insert(name, replacement_list);
				None
			}
		}
	}
	pub fn undef_macro(&mut self, name: String, span: diag::Span) -> Option<diag::Diagnostic> {
		let error = diag::Diagnostic::error(diag::DiagKind::UndefPredef, span);
		match name.as_str() {
			"__DATE__"
			| "__FILE__"
			| "__LINE__"
			| "__STDC__"
			| "__STDC_HOSTED__"
			| "__STDC_MB_MIGHT_NEQ_WC__"
			| "__STDC_VERSION__"
			| "__TIME__"
			| "__STDC_IEC_559__"
			| "__STDC_IEC_559_COMPLEX__"
			| "__STDC_ISO_10646__" => Some(error),
			_ => {
				self.defines.remove(&name);
				None
			}
		}
	}
	pub fn push_lexer(&mut self, lexer: Lexer) {
		self.stack.push(StackKind::Lexer(lexer));
	}
	pub fn push_token(&mut self, triple: PPTokenTriple) {
		match self.stack.last_mut() {
			Some(StackKind::Buffer(buffer)) => buffer.push(Ok(triple)),
			_ => {
				let buffer = vec![Ok(triple)];
				self.stack.push(StackKind::Buffer(buffer))
			}
		}
	}
	pub fn push_result_triple(&mut self, result_triple: diag::ResultTriple<PPToken, usize>) {
		match self.stack.last_mut() {
			Some(StackKind::Buffer(buffer)) => buffer.push(result_triple),
			_ => {
				let buffer = vec![result_triple];
				self.stack.push(StackKind::Buffer(buffer))
			}
		}
	}
	fn pop_token(&mut self) -> Option<diag::ResultTriple<PPToken, usize>> {
		while let Some(queue) = self.stack.last_mut() {
			if let StackKind::Buffer(buffer) = queue {
				if let Some(result) = buffer.pop() {
					return Some(result);
				}
			} else if let StackKind::Lexer(lexer) = queue {
				if let Some(result) = lexer.next() {
					return Some(result);
				}
			}
			self.stack.pop();
		}
		None
	}

	fn preprocess(&mut self, triple: PPTokenTriple) -> Option<diag::ResultTriple<PPToken, usize>> {
		let span = triple.1.to_span();
		let ident = match &triple.1.kind {
			tok::PPTokenKind::Ident(ident) => ident,
			tok::PPTokenKind::NewLine(_) => {
				self.line += 1;
				return Some(Ok(triple));
			}
			tok::PPTokenKind::Punct(tok::Punct::Hash) => {
				match self.pop_token() {
					Some(Ok((
						lo,
						PPToken {
							kind: PPTokenKind::Ident(ident),
							leading_space,
							span,
						},
						hi,
					))) => {
						let kind = match ident.name.as_str() {
							"include" => PPTokenKind::Directive(Directive::Include),
							"if" => PPTokenKind::Directive(Directive::If),
							"ifdef" => PPTokenKind::Directive(Directive::Ifdef),
							"ifndef" => PPTokenKind::Directive(Directive::Ifndef),
							"elif" => PPTokenKind::Directive(Directive::Elif),
							"else" => PPTokenKind::Directive(Directive::Else),
							"endif" => PPTokenKind::Directive(Directive::Endif),
							"define" => PPTokenKind::Directive(Directive::Define),
							"undef" => PPTokenKind::Directive(Directive::Undef),
							"line" => PPTokenKind::Directive(Directive::Line),
							"error" => PPTokenKind::Directive(Directive::Error),
							"pragma" => PPTokenKind::Directive(Directive::Pragma),
							_ => PPTokenKind::Ident(ident),
						};
						let new_tok = PPToken {
							kind,
							leading_space,
							span,
						};
						self.push_token((lo, new_tok, hi));
					}
					Some(other) => {
						self.push_result_triple(other);
					}
					None => {
						// don't care
					}
				}

				return Some(Ok(triple));
			}
			_ => {
				// we don't need this token, so return it to iterator
				return Some(Ok(triple));
			}
		};
		if !ident.expandable {
			return Some(Ok(triple));
		}

		if matches!(
			self.last_token_kind,
			Some(PPTokenKind::Directive(Directive::Define | Directive::Undef))
		) {
			return Some(Ok(triple));
		}

		match ident.name.as_str() {
			"__DATE__" => {
				let seq = current_date();
				let kind = tok::PPTokenKind::StrLit(tok::StrLit {
					seq,
					is_wide: false,
					file_id: span.file_id
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.1.leading_space,
					span,
				};
				return Some(Ok((triple.0, pp_token, triple.2)));
			}
			"__FILE__" => {
				let file_map = self.file_map_ref.borrow();
				let file_path = file_map.get_by_left(&span.file_id).unwrap();
				let seq = file_path.to_str().unwrap().to_owned();
				let kind = tok::PPTokenKind::StrLit(tok::StrLit {
					seq,
					is_wide: false,
					file_id: span.file_id
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.1.leading_space,
					span
				};
				return Some(Ok((triple.0, pp_token, triple.2)));
			}
			"__LINE__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: format!("{}", self.line),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.1.leading_space,
					span
				};
				return Some(Ok((triple.0, pp_token, triple.2)));
			}
			"__STDC__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "1".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.1.leading_space,
					span
				};
				return Some(Ok((triple.0, pp_token, triple.2)));
			}
			// This compiler is freestanding
			"__STDC_HOSTED__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "0".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.1.leading_space,
					span
				};
				return Some(Ok((triple.0, pp_token, triple.2)));
			}
			"__STDC_MB_MIGHT_NEQ_WC__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "0".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.1.leading_space,
					span
				};
				return Some(Ok((triple.0, pp_token, triple.2)));
			}
			"__STDC_VERSION__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "199901L".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.1.leading_space,
					span,
				};
				return Some(Ok((triple.0, pp_token, triple.2)));
			}
			"__TIME__" => {
				let seq = current_time();
				let kind = tok::PPTokenKind::StrLit(tok::StrLit {
					seq,
					is_wide: false,
					file_id: span.file_id
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.1.leading_space,
					span
				};
				return Some(Ok((triple.0, pp_token, triple.2)));
			}
			"__STDC_IEC_559__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "1".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.1.leading_space,
					span
				};
				return Some(Ok((triple.0, pp_token, triple.2)));
			}
			// freestanding implementations are not required to conform to informative annex G.
			"__STDC_IEC_559_COMPLEX__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "0".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.1.leading_space,
					span
				};
				return Some(Ok((triple.0, pp_token, triple.2)));
			}
			"__STDC_ISO_10646__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "0".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.1.leading_space,
					span
				};
				return Some(Ok((triple.0, pp_token, triple.2)));
			}
			_ => {
				// do nothing
			}
		}
		Some(Ok(triple))
	}

	fn read_and_expand_token(&mut self) -> Option<diag::ResultTriple<PPToken, usize>> {
		loop {
			match self.pop_token() {
				Some(Ok(triple)) => {
					if let Some(result) = self.preprocess(triple) {
						match &result {
							Ok((_, token, _)) => self.last_token_kind = Some(token.kind.clone()),
							Err(_) => self.last_token_kind = None,
						}
						return Some(result);
					}
				}
				result => return result,
			}
		}
	}
}

pub struct PPTokenIter {
	pub stack_ref: Rc<RefCell<PPTokenStack>>,
}

impl PPTokenIter {
	pub fn new(value: Lexer, file_map_ref: Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>>) -> Self {
		let pp_token_stack = PPTokenStack::new(value, file_map_ref);
		Self {
			stack_ref: Rc::new(RefCell::new(pp_token_stack)),
		}
	}
}

impl Iterator for PPTokenIter {
	type Item = diag::ResultTriple<PPToken, usize>;
	fn next(&mut self) -> Option<Self::Item> {
		self.stack_ref.borrow_mut().read_and_expand_token()
	}
}
