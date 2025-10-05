use crate::analysis::syn::*;
use crate::diagnostics as diag;

impl super::SemanticParser {
	pub(super) fn compound_stmt(&mut self, stmt: &mut CompoundStmt) {
		self.tree_builder.begin_child("compund-statement { }".to_string());
		self.symtab.increase_scope();
		for item in stmt.blocks.iter_mut() {
			self.block_item(item);
		}
		self.decrease_scope();
		self.tree_builder.end_child();
	}
	pub(super) fn block_item(&mut self, item: &mut BlockItem) -> bool {
		use BlockItem::*;
		let mut is_valid = true;
		match item {
			Declaration(decl) => is_valid &= self.declaration(decl, StorageClass::Auto),
			Statement(stmt) => is_valid &= self.statement(stmt),
			Error => is_valid &= false,
		}
		is_valid
	}
	pub(super) fn statement(&mut self, stmt: &mut Stmt) -> bool {
		let is_valid = true;
		self.tree_builder.begin_child("statement".to_string());
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
		self.tree_builder.end_child();
		is_valid
	}
	fn selection_stmt(&mut self, stmt: &mut SelectStmt) {
		self.tree_builder.begin_child("selection-statement".to_string());
		match stmt {
			SelectStmt::If { stmt_cond, .. } => {
				self.stmt_if(stmt_cond);
			}
			_ => {}
		}
		self.tree_builder.end_child();
	}
	fn stmt_if(&mut self, stmt_cond: &mut Expr) {
		self.tree_builder.begin_child("if-statement".to_string());
		if let Expr::Binary(ExprBinary {
			op: BinOp {
				span,
				kind: BinOpKind::Assign,
			},
			..
		}) = stmt_cond
		{
			// I added this warning just to prove I'm no scrub.
			let mut diag = diag::Diagnostic::warn(diag::DiagKind::IfAssign, span.clone());
			diag.push_note("place parentheses around the assignment to silence this warning");
			diag.push_note("use '==' to turn this assignment into an equality comparison");
			self.diagnostics.push(diag);
		}
		self.expr(stmt_cond);
		self.tree_builder.end_child();
	}
}
