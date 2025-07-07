use crate::analysis::syn::*;

impl super::SemanticParser {
	pub(super) fn expr(&mut self, expr: &mut Expr) {
		use Expr::*;
		match expr {
			Ident(_) => todo!(),
			Const(_) => todo!(),
			StrLit(_) => todo!(),
			Paren(inner) => self.expr(inner),
			Unary(unary) => self.expr_unary(unary),
			Binary(binary) => self.expr_binary(binary),
			Ternary(ternary) => self.expr_ternary(ternary),
			CompoundLiteral(_, _) => todo!("compound-literal"),
			Sizeof(_) => todo!("sizeof"),
		}
	}
	pub(super) fn expr_unary(&mut self, unary: &mut ExprUnary) {
		self.expr(&mut *unary.expr);
		todo!()
	}
	pub(super) fn expr_binary(&mut self, binary: &mut ExprBinary) {
		let _lhs_id = self.expr(&mut *binary.left);
		let _rhs_id = self.expr(&mut *binary.right);
		match binary.op {
			BinOp::Add => {
				todo!()
			}
			_ => todo!(),
		}
	}
	pub(super) fn expr_ternary(&mut self, ternary: &mut ExprTernary) {
		self.expr(&mut *ternary.cond);
		self.expr(&mut *ternary.then_branch);
		self.expr(&mut *ternary.else_branch);
		todo!()
	}
}
