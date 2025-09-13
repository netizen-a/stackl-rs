use crate::analysis::syn::*;

impl super::SemanticParser<'_> {
	pub(super) fn compound_stmt(&mut self, stmt: &mut CompoundStmt) {
		self.symtab.increase_scope();
		for item in stmt.0.iter_mut() {
			self.block_item(item)
		}
		self.symtab.decrease_scope();
	}
	pub(super) fn block_item(&mut self, item: &mut BlockItem) {
		use BlockItem::*;
		match item {
			Declaration(decl) => self.declaration(decl, StorageClass::Auto),
			Statement(stmt) => self.statement(stmt),
			Error => {}
		}
	}
	pub(super) fn statement(&mut self, stmt: &mut Stmt) {
		match stmt {
			Stmt::Labeled(_labeled_stmt) => (),
			Stmt::Compound(inner) => self.compound_stmt(inner),
			Stmt::Expr(_expr_stmt) => (),
			Stmt::Selection(_selection_stmt) => (),
			Stmt::Iter(_iter_stmt) => (),
			Stmt::Jump(_jmp_stmt) => (),
			Stmt::Asm(_asm_stmt) => (),
		}
	}
}
