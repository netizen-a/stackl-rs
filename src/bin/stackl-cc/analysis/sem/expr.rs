use crate::analysis::syn::*;

impl super::SemanticParser<'_> {
	pub(super) fn expr(&mut self, expr: &mut Expr) {
		use Expr::*;
		match expr {
			Ident(_inner) => {}
			Const(_inner) => {}
			StrLit(_inner) => {}
			Unary(unary) => self.expr_unary(unary),
			Binary(binary) => self.expr_binary(binary),
			Ternary(ternary) => self.expr_ternary(ternary),
			CompoundLiteral(_, _) => todo!("compound-literal"),
			Sizeof(_) => todo!("sizeof"),
		}
	}
	pub(super) fn expr_unary(&mut self, unary: &mut ExprUnary) {
		self.expr(&mut *unary.expr);
	}
	pub(super) fn expr_binary(&mut self, binary: &mut ExprBinary) {
		let _lhs_id = self.expr(&mut *binary.left);
		let _rhs_id = self.expr(&mut *binary.right);
	}
	pub(super) fn expr_ternary(&mut self, ternary: &mut ExprTernary) {
		self.expr(&mut *ternary.cond);
		self.expr(&mut *ternary.then_branch);
		self.expr(&mut *ternary.else_branch);
	}
}
