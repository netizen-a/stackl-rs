use crate::analysis::{syn::*, tok::{Const, IntegerConstant}};
use crate::data_types as dtype;

impl super::SemanticParser {
	pub(super) fn expr(&mut self, expr: &mut Expr) -> bool {
		use Expr::*;
		let mut is_valid = true;
		match expr {
			Paren(paren) => {
				self.tree_builder.begin_child("( expression )".to_string());
				is_valid &= self.expr(paren);
				self.tree_builder.end_child();
			},
			Ident(_inner) => {
				self.tree_builder.add_empty_child("identifier".to_string());
				is_valid &= true
			},
			Const(inner) => is_valid &= self.expr_const(inner),
			StrLit(_inner) => is_valid &= true,
			UnaryPrefix(unary) => is_valid &= self.expr_prefix(unary),
			UnaryPostfix(unary) => is_valid &= self.expr_postfix(unary),
			Binary(binary) => is_valid &= self.expr_binary(binary),
			Ternary(ternary) => is_valid &= self.expr_ternary(ternary),
			CompoundLiteral(_, _) => is_valid &= true,
			Sizeof(_) => todo!("sizeof"),
		}
		is_valid
	}
	pub(super) fn expr_prefix(&mut self, unary: &mut UnaryPrefix) -> bool {
		self.expr(&mut *unary.expr)
	}
	pub(super) fn expr_postfix(&mut self, unary: &mut UnaryPostfix) -> bool {
		self.expr(&mut *unary.expr)
	}
	pub(super) fn expr_binary(&mut self, binary: &mut ExprBinary) -> bool {
		let mut is_valid = true;
		let _ = match &binary.op.kind {
			BinOpKind::Add => self.tree_builder.begin_child("+".to_string()),
			BinOpKind::Mul => self.tree_builder.begin_child("*".to_string()),
			_ => todo!(),
		};
		is_valid &= self.expr(&mut *binary.left);
		is_valid &= self.expr(&mut *binary.right);
		self.tree_builder.end_child();
		is_valid
	}
	pub(super) fn expr_ternary(&mut self, ternary: &mut ExprTernary) -> bool {
		let mut is_valid = true;
		is_valid &= self.expr(&mut *ternary.expr_cond);
		is_valid &= self.expr(&mut *ternary.expr_then);
		is_valid &= self.expr(&mut *ternary.expr_else);
		is_valid
	}
	pub(super) fn expr_const(&mut self, constant: &mut Const) -> bool {
		match constant {
			Const::Integer(IntegerConstant::I32(inner)) => {
				self.tree_builder.add_empty_child(format!("integer-constant {inner}"));
			}
			_ => {}
		}
		true
	}
	// pub(super) fn resolve_lvalue() -> dtype::DataType {
	// 	todo!()
	// }
	// pub(super) fn resolve_rvalue() -> dtype::DataType {
	// 	todo!()
	// }
}
