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
			Error => {}
		}
	}
	pub(super) fn statement(&mut self, stmt: &Stmt) {
		match stmt {
			Stmt::Labeled(_labeled_stmt) => (),
			Stmt::Compound(_compound_stmt) => (),
			Stmt::Expr(_expr_stmt) => (),
			Stmt::Selection(_selection_stmt) => (),
			Stmt::Iter(_iter_stmt) => (),
			Stmt::Jump(_jmp_stmt) => (),
			Stmt::Asm(_asm_stmt) => (),
		}
	}
}
