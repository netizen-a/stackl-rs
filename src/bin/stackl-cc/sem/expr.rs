use crate::syn::*;

impl super::SemanticParser {
	pub(super) fn expr(&mut self, expr: Expr) {
		use Expr::*;
		match expr {
			Ident(_) => (),
			Const(_) => (),
			StrLit(_) => (),
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
	}
	pub(super) fn expr_binary(&mut self, binary: ExprBinary) {
		self.expr(*binary.left);
		self.expr(*binary.right);
	}
	pub(super) fn expr_ternary(&mut self, ternary: ExprTernary) {
		self.expr(*ternary.cond);
		self.expr(*ternary.then_branch);
		self.expr(*ternary.else_branch);
	}
}
