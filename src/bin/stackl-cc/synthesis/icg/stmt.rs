// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::{
	analysis::syn,
	diagnostics::Diagnostic,
	synthesis::icg::DataLayout,
};

impl super::SSACodeGen<'_> {
	pub(super) fn statement(&mut self, stmt: &syn::Stmt) -> Result<(), Diagnostic> {
		match stmt {
			syn::Stmt::Label(syn::LabeledStmt::Label(label, stmt)) => {
				let id = self.builder.label().unwrap();
				self.label_table.insert(label.name.as_str(), id);
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
				todo!("store return result of expr on stack");
			}
			other => todo!("{other:?}"),
		}
		Ok(())
	}
}
