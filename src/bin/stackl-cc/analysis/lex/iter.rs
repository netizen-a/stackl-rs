// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::{
	cell::RefCell,
	collections::HashMap,
	path::PathBuf,
	rc::Rc,
	time,
};

use crate::{
	analysis::tok::{
		self,
		Directive,
		PPToken,
		PPTokenKind,
	},
	diagnostics::DiagnosticEngine,
};

use super::lexer::Lexer;
use crate::diagnostics as diag;
use crate::diagnostics::ToSpan;
use chrono::{
	Datelike,
	Timelike,
};
use lalrpop_util as lalr;

pub enum StackKind {
	Buffer(Vec<Result<PPToken, diag::Diagnostic>>),
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
	defines: HashMap<String, Vec<PPToken>>,
	file_map_ref: Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>>,
	line: usize,
	last_token_kind: Option<PPTokenKind>,
	id_map: HashMap<usize, usize>,
}

impl PPTokenStack {
	fn new(value: Lexer, file_map_ref: Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>>) -> Self {
		Self {
			stack: vec![StackKind::Lexer(value)],
			defines: HashMap::new(),
			file_map_ref,
			line: 1,
			last_token_kind: None,
			id_map: HashMap::new(),
		}
	}
	pub fn define_obj_macro(
		&mut self,
		name: String,
		replacement_list: Vec<PPToken>,
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

	pub fn directive_line(&mut self, tokens: Vec<PPToken>) -> Option<diag::Diagnostic> {
		let mut line_num: usize = self.line;
		let mut file_name = None;
		let mut id_pair = None;
		for (index, token) in tokens.iter().enumerate() {
			if index == 0 {
				match &token.kind {
					PPTokenKind::PPNumber(num) => match num.name.parse::<usize>() {
						Ok(new_line_num @ 1..=2147483647) => line_num = new_line_num,
						Ok(0) => {
							let error = diag::Diagnostic::error(
								diag::DiagKind::DirectiveLineMinRange,
								token.to_span(),
							);
							return Some(error);
						}
						Ok(2147483648..) => {
							let error = diag::Diagnostic::error(
								diag::DiagKind::DirectiveLineMinRange,
								token.to_span(),
							);
							return Some(error);
						}
						Err(_) => {
							let error = diag::Diagnostic::error(
								diag::DiagKind::DirectiveLineNotSimple,
								token.to_span(),
							);
							return Some(error);
						}
					},
					_ => {
						let error = diag::Diagnostic::error(
							diag::DiagKind::DirectiveLineNotSimple,
							token.to_span(),
						);
						return Some(error);
					}
				}
			} else if index == 1 {
				match &token.kind {
					PPTokenKind::StrLit(str_lit) => {
						if str_lit.is_wide {
							let error = diag::Diagnostic::error(
								diag::DiagKind::DirectiveLineFilename,
								token.to_span(),
							);
							return Some(error);
						} else {
							let file_map = self.file_map_ref.borrow();
							let name_path = PathBuf::from(str_lit.seq.clone());
							let span = token.to_span();
							if let Some(name_id) = file_map.get_by_right(&name_path) {
								id_pair = Some((span.file_id, *name_id));
							} else {
								id_pair = Some((span.file_id, file_map.len()));
								file_name = Some(str_lit.seq.clone());
							}
						}
					}
					_ => {
						let error = diag::Diagnostic::error(
							diag::DiagKind::DirectiveLineFilename,
							token.to_span(),
						);
						return Some(error);
					}
				}
			} else {
				let error = diag::Diagnostic::error(
					diag::DiagKind::DirectiveExtraTokens(Directive::Line),
					token.to_span(),
				);
				return Some(error);
			}
		}
		self.line = line_num;
		if let Some((file_id, file_name_id)) = id_pair {
			self.id_map.insert(file_id, file_name_id);
			if let Some(file_name) = file_name {
				let mut file_map = self.file_map_ref.borrow_mut();
				file_map.insert(file_name_id, PathBuf::from(file_name));
			}
		}
		None
	}
	pub fn push_lexer(&mut self, lexer: Lexer) {
		self.stack.push(StackKind::Lexer(lexer));
	}
	pub fn push_token(&mut self, triple: PPToken) {
		match self.stack.last_mut() {
			Some(StackKind::Buffer(buffer)) => buffer.push(Ok(triple)),
			_ => {
				let buffer = vec![Ok(triple)];
				self.stack.push(StackKind::Buffer(buffer))
			}
		}
	}
	pub fn push_result_triple(&mut self, result_triple: Result<PPToken, diag::Diagnostic>) {
		match self.stack.last_mut() {
			Some(StackKind::Buffer(buffer)) => buffer.push(result_triple),
			_ => {
				let buffer = vec![result_triple];
				self.stack.push(StackKind::Buffer(buffer))
			}
		}
	}
	fn pop_token(&mut self) -> Option<Result<PPToken, diag::Diagnostic>> {
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

	fn preprocess(&mut self, mut triple: PPToken) -> Option<Result<PPToken, diag::Diagnostic>> {
		if let Some(name_id) = self.id_map.get(&triple.span.file_id) {
			triple.span.name_id = *name_id;
		}
		triple.span.line = self.line;

		let span = triple.to_span();
		let ident = match &triple.kind {
			tok::PPTokenKind::Ident(ident) => ident,
			tok::PPTokenKind::NewLine(_) => {
				self.line += 1;
				triple.span.line = self.line;
				return Some(Ok(triple));
			}
			tok::PPTokenKind::Punct(tok::Punct::Hash) => {
				match self.pop_token() {
					Some(Ok(PPToken {
						kind: PPTokenKind::Ident(ident),
						leading_space,
						span,
					})) => {
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
						self.push_token(new_tok);
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

		// Prevents predef'd macros from expanding at the start of define and undef directives.
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
					file_id: span.file_id,
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			"__FILE__" => {
				let file_map = self.file_map_ref.borrow();
				let file_path = file_map.get_by_left(&span.file_id).unwrap();
				let seq = file_path.to_str().unwrap().to_owned();
				let kind = tok::PPTokenKind::StrLit(tok::StrLit {
					seq,
					is_wide: false,
					file_id: span.file_id,
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			"__LINE__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: format!("{}", self.line),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			"__STDC__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "1".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			"__STACKLC__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "1".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			// This compiler is freestanding
			"__STDC_HOSTED__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "0".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			"__STDC_MB_MIGHT_NEQ_WC__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "0".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			"__STDC_VERSION__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "199901L".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			"__TIME__" => {
				let seq = current_time();
				let kind = tok::PPTokenKind::StrLit(tok::StrLit {
					seq,
					is_wide: false,
					file_id: span.file_id,
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			"__STDC_IEC_559__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "1".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			// freestanding implementations are not required to conform to informative annex G.
			"__STDC_IEC_559_COMPLEX__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "0".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			"__STDC_ISO_10646__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {
					name: "0".to_string(),
				});
				let pp_token = tok::PPToken {
					kind,
					leading_space: triple.leading_space,
					span,
				};
				return Some(Ok(pp_token));
			}
			_ => {
				// do nothing
			}
		}
		Some(Ok(triple))
	}

	fn read_and_expand_token(&mut self) -> Option<Result<PPToken, diag::Diagnostic>> {
		loop {
			match self.pop_token() {
				Some(Ok(triple)) => {
					if let Some(result) = self.preprocess(triple) {
						match &result {
							Ok(token) => self.last_token_kind = Some(token.kind.clone()),
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
	pub stack_ref: PPTokenStack,
}

impl PPTokenIter {
	pub fn new(value: Lexer, file_map_ref: Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>>) -> Self {
		let pp_token_stack = PPTokenStack::new(value, file_map_ref);
		Self {
			stack_ref: pp_token_stack,
		}
	}
}

impl Iterator for PPTokenIter {
	type Item = Result<PPToken, diag::Diagnostic>;
	fn next(&mut self) -> Option<Self::Item> {
		self.stack_ref.read_and_expand_token()
	}
}
