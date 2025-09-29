use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

use crate::{analysis::tok::{self, PPToken, PPTokenTriple}, diagnostics::DiagnosticEngine};

use super::lexer::Lexer;
use crate::diagnostics as diag;

pub enum StackKind {
	Buffer(Vec<diag::ResultTriple<PPToken, usize>>),
	Lexer(Lexer),
}

#[derive(Default)]
pub struct PPTokenStack {
	stack: Vec<StackKind>,
	defines: HashMap<String, Vec<PPTokenTriple>>,
	file_map_ref: Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>>,
	line: usize,
}

impl PPTokenStack {
	pub fn define_obj_macro(&mut self, name: String, replacement_list: Vec<PPTokenTriple>) {
		self.defines.insert(name, replacement_list);
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

	fn preprocess(&mut self, triple: PPTokenTriple) -> Option<PPTokenTriple> {
		let file_id = triple.1.file_id;
		let ident = match &triple.1.kind {
			tok::PPTokenKind::Ident(ident) => ident,
			tok::PPTokenKind::NewLine(_) => {
				self.line += 1;
				return Some(triple);
			},
			_ => {
				// we don't need this token, so return it to iterator
				return Some(triple);
			}
		};
		if !ident.expandable {
			return Some(triple);
		}

		match ident.name.as_str() {
			"__DATE__" => {},
			"__FILE__" => {
				let file_map = self.file_map_ref.borrow();
				let file_path = file_map.get_by_left(&file_id).unwrap();
				let seq = file_path.to_str().unwrap().to_owned();
				let kind = tok::PPTokenKind::StrLit(tok::StrLit { seq, is_wide: false, file_id });
				let pp_token = tok::PPToken {file_id, kind, leading_space: triple.1.leading_space};
				return Some((triple.0, pp_token, triple.2));
			},
			"__LINE__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {name: format!("{}", self.line)});
				let pp_token = tok::PPToken {file_id, kind, leading_space: triple.1.leading_space};
				return Some((triple.0, pp_token, triple.2));
			},
			"__STDC__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {name: "1".to_string()});
				let pp_token = tok::PPToken {file_id, kind, leading_space: triple.1.leading_space};
				return Some((triple.0, pp_token, triple.2));
			},
			"__STDC_HOSTED__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {name: "0".to_string()});
				let pp_token = tok::PPToken {file_id, kind, leading_space: triple.1.leading_space};
				return Some((triple.0, pp_token, triple.2));
			},
			"__STDC_MB_MIGHT_NEQ_WC__" => {},
			"__STDC_VERSION__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {name: "199901L".to_string()});
				let pp_token = tok::PPToken {file_id, kind, leading_space: triple.1.leading_space};
				return Some((triple.0, pp_token, triple.2));
			},
			"__TIME__" => {},
			"__STDC_IEC_559__" => {},
			// freestanding implementations are not required to conform to informative annex G.
			"__STDC_IEC_559_COMPLEX__" => {
				let kind = tok::PPTokenKind::PPNumber(tok::PPNumber {name: "0".to_string()});
				let pp_token = tok::PPToken {file_id, kind, leading_space: triple.1.leading_space};
				return Some((triple.0, pp_token, triple.2));
			},
			"__STDC_ISO_10646__" => {},
			_ => {
				// do nothing
			},
		}
		Some(triple)
	}

	fn read_and_expand_token(&mut self) -> Option<diag::ResultTriple<PPToken, usize>> {
		loop {
			let maybe = self.pop_token();
			match maybe {
				Some(Ok(triple)) => {
					if let Some(result) = self.preprocess(triple) {
						return Some(Ok(result));
					}
				}
				Some(Err(_)) | None => return maybe,
			}
		}
	}
}

pub struct PPTokenIter {
	pub stack_ref: Rc<RefCell<PPTokenStack>>,
}

impl PPTokenIter {
	pub fn new(value: Lexer, file_map_ref: Rc<RefCell<bimap::BiHashMap<usize, PathBuf>>>) -> Self {
		let pp_token_stack = PPTokenStack {
			stack: vec![StackKind::Lexer(value)],
			defines: HashMap::new(),
			file_map_ref,
			line: 1,
		};
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
