use crate::analysis::syn::*;

impl super::SemanticParser {
	pub(super) fn compound_stmt(&mut self, stmt: &mut CompoundStmt) {
		for item in stmt.0.iter_mut() {
			self.block_item(item)
		}
	}
	pub(super) fn block_item(&mut self, item: &mut BlockItem) {
		use BlockItem::*;
		match item {
			Declaration(decl) => self.declaration(decl),
			Statement(stmt) => self.statement(stmt),
		}
	}
	pub(super) fn statement(&mut self, stmt: &mut Stmt) {
		use Stmt::*;
		match stmt {
			LabeledStatement(_labeled_stmt) => (),
			CompoundStatement(_compound_stmt) => (),
			ExpressionStatement(_expr_stmt) => (),
			SelectionStatement(_selection_stmt) => (),
			IterationStatement(_iter_stmt) => (),
			JumpStatement(_jmp_stmt) => (),
		}
	}
}
