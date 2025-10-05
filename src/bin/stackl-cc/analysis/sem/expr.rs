use crate::{analysis::{syn::*, tok::{Const, IntegerConstant}}, diagnostics::ToSpan};
use crate::data_types as dtype;

impl super::SemanticParser {
	pub(super) fn expr(&mut self, expr: &mut Expr) -> Option<dtype::DataType> {
		use Expr::*;
		match expr {
			Paren(paren) => {
				self.tree_builder.begin_child("( expression )".to_string());
				self.tree_builder.end_child();
				None
			},
			Ident(inner) => {
				let span = inner.to_span();
				self.tree_builder.add_empty_child(format!("identifier <{}:{}> {}", span.loc.0, span.loc.1, inner.name));
				None
			},
			Const(inner) => {
				Some(self.expr_const(inner))
			},
			StrLit(_inner) => {None},
			UnaryPrefix(unary) => {self.expr_prefix(unary); None},
			UnaryPostfix(unary) => {self.expr_postfix(unary); None},
			Binary(binary) => {self.expr_binary(binary); None},
			Ternary(ternary) => {self.expr_ternary(ternary); None},
			CompoundLiteral(_, _) => None,
			Sizeof(_) => None,
		}
	}
	pub(super) fn expr_prefix(&mut self, unary: &mut UnaryPrefix) {
		match unary.op {
			Prefix::Amp => {
				self.tree_builder.begin_child("expr-prefix &".to_string());
			},
			_ => todo!()
		}
		self.expr(&mut *unary.expr);
		self.tree_builder.end_child();
	}
	pub(super) fn expr_postfix(&mut self, unary: &mut UnaryPostfix) {
		self.expr(&mut *unary.expr);
	}
	pub(super) fn expr_binary(&mut self, binary: &mut ExprBinary) {
		let _ = match &binary.op.kind {
			BinOpKind::Add => self.tree_builder.begin_child("+".to_string()),
			BinOpKind::Mul => self.tree_builder.begin_child("*".to_string()),
			_ => todo!(),
		};
		self.expr(&mut *binary.left);
		self.expr(&mut *binary.right);
		self.tree_builder.end_child();
	}
	pub(super) fn expr_ternary(&mut self, ternary: &mut ExprTernary) {
		self.expr(&mut *ternary.expr_cond);
		self.expr(&mut *ternary.expr_then);
		self.expr(&mut *ternary.expr_else);
	}
	pub(super) fn expr_const(&mut self, constant: &mut Const) -> dtype::DataType {
		match constant {
			Const::Integer(IntegerConstant::I32(inner)) => {
				self.tree_builder.add_empty_child(format!("integer-constant {inner}"));
				let kind = dtype::TypeKind::Scalar(dtype::ScalarType::I32);
				dtype::DataType {
					kind,
					qual: Default::default(),
				}
			}
			_ => todo!()
		}
	}
}
