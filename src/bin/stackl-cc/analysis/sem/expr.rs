use crate::analysis::syn::*;

impl super::SemanticParser<'_> {
	pub(super) fn expr(&mut self, expr: &mut Expr) -> bool {
		use Expr::*;
		let mut is_valid = true;
		match expr {
			Paren(paren) => is_valid &= self.expr(paren),
			Ident(_inner) => is_valid &= true,
			Const(_inner) => is_valid &= true,
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
		is_valid &= self.expr(&mut *binary.left);
		is_valid &= self.expr(&mut *binary.right);
		is_valid
	}
	pub(super) fn expr_ternary(&mut self, ternary: &mut ExprTernary) -> bool {
		let mut is_valid = true;
		is_valid &= self.expr(&mut *ternary.expr_cond);
		is_valid &= self.expr(&mut *ternary.expr_then);
		is_valid &= self.expr(&mut *ternary.expr_else);
		is_valid
	}
}
