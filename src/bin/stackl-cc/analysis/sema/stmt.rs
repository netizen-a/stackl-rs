// Copyright (c) 2024-2026 Jonathan A. Thomason

use super::expr::ExprContext;
use crate::analysis::syn::*;
use crate::diagnostics as diag;

impl super::SemanticParser<'_> {
	pub(super) fn compound_stmt(&mut self, stmt: &mut CompoundStmt) {
		self.tree_builder
			.begin_child("compund-statement { }".to_string());
		self.increase_scope();
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
			Declaration(decl) => is_valid &= self.declaration(decl, StorageClass::Auto, true),
			Statement(stmt) => is_valid &= self.statement(stmt),
			Pragma(_) => {}
			Error => is_valid &= false,
		}
		is_valid
	}
	pub(super) fn statement(&mut self, stmt: &mut Stmt) -> bool {
		let mut is_valid = true;
		self.tree_builder.begin_child("statement".to_string());
		match stmt {
			Stmt::Label(_labeled_stmt) => (),
			Stmt::Compound(inner) => self.compound_stmt(inner),
			Stmt::Expr(expr_stmt) => {
				if let Some(expr) = &mut expr_stmt.0 {
					let expr_context = ExprContext {
						in_func: true,
						is_mut: true,
						enabled_diag: true,
					};
					is_valid &= !self.expr(expr, &expr_context).is_poisoned();
				}
			}
			Stmt::Select(stmt) => {
				self.selection_stmt(stmt);
			}
			Stmt::Iter(_iter_stmt) => (),
			Stmt::Jump(inner) => self.jmp_stmt(inner),
			Stmt::Asm(_asm_stmt) => (),
			Stmt::Error => {
				is_valid = false;
			}
		}
		self.tree_builder.end_child();
		is_valid
	}
	fn selection_stmt(&mut self, stmt: &mut SelectStmt) {
		self.tree_builder
			.begin_child("selection-statement".to_string());
		match stmt {
			SelectStmt::If {
				stmt_cond,
				stmt_then,
				stmt_else,
			} => {
				self.stmt_if(stmt_cond);
				self.stmt_then(stmt_then);
				if let Some(stmt_else) = stmt_else {
					self.stmt_else(stmt_else)
				}
			}
			_ => {}
		}
		self.tree_builder.end_child();
	}
	fn jmp_stmt(&mut self, stmt: &mut JumpStmt) {
		self.tree_builder
			.begin_child("selection-statement".to_string());
		match stmt {
			JumpStmt::Return(_) => {
				self.tree_builder.add_empty_child("return".to_string());
			}
			other => todo!("{other:?}"),
		}
		self.tree_builder.end_child();
	}
	fn stmt_if(&mut self, stmt_cond: &mut Expr) {
		self.tree_builder
			.begin_child("if ( expression )".to_string());
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
			self.diagnostics.push(diag);
		}
		let expr_context = ExprContext {
			in_func: true,
			is_mut: true,
			enabled_diag: true,
		};
		self.expr(stmt_cond, &expr_context);
		self.tree_builder.end_child();
	}
	fn stmt_then(&mut self, stmt_then: &mut Stmt) {
		self.tree_builder.begin_child("then-statement".to_string());
		self.statement(stmt_then);
		self.tree_builder.end_child();
	}
	fn stmt_else(&mut self, stmt_then: &mut Stmt) {
		self.tree_builder.begin_child("else-statement".to_string());
		self.statement(stmt_then);
		self.tree_builder.end_child();
	}
}
