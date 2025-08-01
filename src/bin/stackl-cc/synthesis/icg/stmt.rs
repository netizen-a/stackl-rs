use crate::analysis::syn::*;

impl super::IntermediateCodeGen {
	pub(super) fn compound_stmt(&mut self, stmt: &CompoundStmt) {
		for item in stmt.0.iter() {
			self.block_item(item)
		}
	}
	pub(super) fn block_item(&mut self, item: &BlockItem) {
		use BlockItem::*;
		match item {
			Declaration(decl) => self.declaration(decl),
			Statement(stmt) => self.statement(stmt),
		}
	}
	pub(super) fn statement(&mut self, stmt: &Stmt) {
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
