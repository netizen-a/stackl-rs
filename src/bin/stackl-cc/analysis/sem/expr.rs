use crate::analysis::syn::*;

impl super::SemanticParser {
	pub(super) fn expr(&mut self, expr: Expr) {
		use Expr::*;
		match expr {
			Ident(_) => todo!(),
			Const(_) => todo!(),
			StrLit(_) => todo!(),
			Paren(expr) => self.expr(*expr),
			Unary(unary) => self.expr_unary(unary),
			Binary(binary) => self.expr_binary(binary),
			Ternary(ternary) => self.expr_ternary(ternary),
			CompoundLiteral(_, _) => todo!("compound-literal"),
			Sizeof(_) => todo!("sizeof"),
		}
	}
	pub(super) fn expr_unary(&mut self, unary: ExprUnary) {
		self.expr(*unary.expr);
		todo!()
	}
	pub(super) fn expr_binary(&mut self, binary: ExprBinary) {
		let _lhs_id = self.expr(*binary.left);
		let _rhs_id = self.expr(*binary.right);
		match binary.op {
			BinOp::Add => {
				todo!()
			}
			_ => todo!(),
		}
	}
	pub(super) fn expr_ternary(&mut self, ternary: ExprTernary) {
		self.expr(*ternary.cond);
		self.expr(*ternary.then_branch);
		self.expr(*ternary.else_branch);
		todo!()
	}
}
