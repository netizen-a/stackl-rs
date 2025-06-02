mod decl;
mod expr;
mod stmt;

use crate::syn::TranslationUnit;

pub struct SemanticParser {}

impl SemanticParser {
	pub fn new() -> Self {
		Self {}
	}
	pub fn parse(&mut self, _unit: TranslationUnit) {
		todo!()
	}
}
