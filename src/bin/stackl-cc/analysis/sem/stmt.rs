use crate::analysis::syn::*;

impl super::SemanticParser<'_> {
	pub(super) fn compound_stmt(&mut self, stmt: &mut CompoundStmt) {
		self.symtab.increase_scope();
		for item in stmt.blocks.iter_mut() {
			self.block_item(item)
		}
		self.decrease_scope();
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
			Stmt::Label(_labeled_stmt) => (),
			Stmt::Compound(inner) => self.compound_stmt(inner),
			Stmt::Expr(_expr_stmt) => (),
			Stmt::Select(stmt) => {
				self.selection_stmt(stmt);
			}
			Stmt::Iter(_iter_stmt) => (),
			Stmt::Jump(_jmp_stmt) => (),
			Stmt::Asm(_asm_stmt) => (),
		}
	}
	fn selection_stmt(&mut self, stmt: &mut SelectStmt) {
		// TODO
	}
}
