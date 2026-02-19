// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::{
	analysis::syn,
	diagnostics::{
		DiagKind,
		Diagnostic,
	},
	synthesis::icg::{
		DataLayout,
		IntegerLayout,
	},
};

impl super::SSACodeGen<'_> {
	pub(super) fn statement(&mut self, stmt: &syn::Stmt) -> Result<(), Diagnostic> {
		match stmt {
			syn::Stmt::Label(syn::LabeledStmt::Label(label, stmt)) => {
				// Check if label already exists in label_table
				if let Some(&label_id) = self.label_table.global_lookup(&label.name) {
					self.builder.label(label_id).unwrap();
				} else {
					let label_id = self.builder.id();
					self.builder.label(label_id).unwrap();
					self.label_table.insert(label.name.as_str(), label_id);
				}
				self.statement(stmt);
			}
			syn::Stmt::Expr(syn::ExprStmt(None)) => {
				self.builder.nop();
			}
			syn::Stmt::Expr(syn::ExprStmt(Some(stmt))) => {
				self.expr(stmt);
			}
			syn::Stmt::Compound(syn::CompoundStmt { blocks, .. }) => {
				self.increase_scope();
				for block in blocks.iter() {
					match block {
						syn::BlockItem::Declaration(decl) => self.declaration(decl)?,
						syn::BlockItem::Statement(stmt) => self.statement(stmt)?,
						_ => todo!(),
					}
				}
				self.decrease_scope();
			}
			syn::Stmt::Jump(syn::JumpStmt::Goto(label)) => {
				let target_label = self.label_table.global_lookup(&label.name).unwrap();
				self.builder.branch(*target_label).unwrap();
			}
			syn::Stmt::Jump(syn::JumpStmt::Return(None)) => {
				self.builder.ret();
			}
			syn::Stmt::Jump(syn::JumpStmt::Return(Some(expr))) => {
				// TODO: This is incomplete - only works with int values
				// Need to check expression type and handle floats, pointers, etc.
				let (result_id, _result_layout) = self.expr(expr);
				self.builder.ret_val(result_id).unwrap();
			}
			syn::Stmt::Select(syn::SelectStmt::If {
				stmt_cond,
				stmt_then,
				stmt_else,
			}) => {
				let (cond_id, cond_layout) = self.expr(stmt_cond);
				let then_label_id = self.builder.id();
				let after_label_id = self.builder.id();

				self.builder
					.branch_conditional(cond_id, then_label_id, after_label_id)
					.unwrap();
				self.builder.label(then_label_id).unwrap();
				self.statement(stmt_then)?;
				self.builder.branch(after_label_id).unwrap();

				if let Some(else_stmt) = stmt_else {
					let else_label_id = after_label_id;
					let after_label_id = self.builder.id();
					self.builder.branch(after_label_id).unwrap();
					self.builder.label(else_label_id).unwrap();
					self.statement(else_stmt)?;
					self.builder.label(after_label_id).unwrap();
				} else {
					self.builder.label(after_label_id).unwrap();
				}
			}
			syn::Stmt::Select(syn::SelectStmt::Switch { expr, stmt }) => {
				todo!()
			}
			syn::Stmt::Iter(syn::IterStmt::While { cond, stmt }) => {
				let loop_label_id = self.builder.id();
				let true_label = self.builder.id();
				let false_label = self.builder.id();

				self.builder.label(loop_label_id).unwrap();
				let (cond_id, _) = self.expr(cond);
				self.current_loop_label = Some(loop_label_id);
				self.builder
					.branch_conditional(cond_id, true_label, false_label)
					.unwrap();
				self.builder.label(true_label).unwrap();
				self.statement(stmt)?;
				self.current_loop_label = None;
				self.builder.label(false_label).unwrap();
			}
			syn::Stmt::Iter(syn::IterStmt::DoWhile { stmt, cond }) => {
				let true_label = self.builder.id();
				let false_label = self.builder.id();

				self.current_loop_label = Some(true_label);
				self.builder.label(true_label).unwrap();
				self.statement(stmt)?;
				let (cond_id, _) = self.expr(cond);

				self.builder
					.branch_conditional(cond_id, true_label, false_label)
					.unwrap();
				self.builder.label(false_label).unwrap();
				self.current_loop_label = None;
			}
			syn::Stmt::Iter(syn::IterStmt::ForExpr {
				init_expr,
				cond,
				iteration,
				stmt,
			}) => {
				todo!()
			}
			syn::Stmt::Iter(syn::IterStmt::ForDecl {
				init_decl,
				cond,
				iteration,
				stmt,
			}) => {
				todo!()
			}
			syn::Stmt::Jump(syn::JumpStmt::Continue) => {
				if let Some(loop_label) = self.current_loop_label {
					self.builder.branch(loop_label).unwrap();
				} else {
					self.diag_engine.push_and_exit(Diagnostic::fatal(
						DiagKind::Internal("continue statement outside of loop"),
						None,
					));
				}
			}
			syn::Stmt::Jump(syn::JumpStmt::Break) => {
				// Break usually jumps to the after_label of the loop
				// For simplicity, we'll implement this as a jump to the loop exit label
				// In a more complete implementation, we'd track the specific loop exit label
				todo!("break statement - need loop exit label tracking");
			}
			other => todo!("{other:?}"),
		}
		Ok(())
	}
}
