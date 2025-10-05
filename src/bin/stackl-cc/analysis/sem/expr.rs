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
	pub(super) fn expr_prefix(&mut self, unary: &mut UnaryPrefix) -> Option<dtype::DataType> {
		let mut result = None;
		match unary.op {
			Prefix::Amp => {
				self.tree_builder.begin_child("expr-prefix &".to_string());
				let Some(inner_type) = self.expr(&mut *unary.expr) else {
					return Some(dtype::DataType {
						kind: dtype::TypeKind::Poison,
						qual: Default::default(),
					});
				};
				let kind = dtype::TypeKind::Pointer(dtype::PtrType(Box::new(inner_type)));
				result = Some(dtype::DataType{
					kind,
					qual: Default::default()
				})
			},
			_ => todo!()
		}
		self.tree_builder.end_child();
		result
	}
	pub(super) fn expr_postfix(&mut self, unary: &mut UnaryPostfix) {
		self.expr(&mut *unary.expr);
	}
	pub(super) fn expr_binary(&mut self, binary: &mut ExprBinary) {
		let _ = match &binary.op.kind {
			BinOpKind::Mul => self.tree_builder.begin_child("*".to_string()),
			BinOpKind::Div => self.tree_builder.begin_child("/".to_string()),
			BinOpKind::Rem => self.tree_builder.begin_child("%".to_string()),
			BinOpKind::Sub => self.tree_builder.begin_child("-".to_string()),
			BinOpKind::Add => self.tree_builder.begin_child("+".to_string()),
			BinOpKind::NotEqual => self.tree_builder.begin_child("!=".to_string()),
			BinOpKind::Equal => self.tree_builder.begin_child("==".to_string()),
			BinOpKind::And => self.tree_builder.begin_child("&".to_string()),
			BinOpKind::XOr => self.tree_builder.begin_child("^".to_string()),
			BinOpKind::Or => self.tree_builder.begin_child("|".to_string()),
			BinOpKind::LogicalAnd => self.tree_builder.begin_child("&&".to_string()),
			BinOpKind::LogicalOr => self.tree_builder.begin_child("||".to_string()),
			BinOpKind::Assign => self.tree_builder.begin_child("=".to_string()),
			BinOpKind::MulAssign => self.tree_builder.begin_child("*=".to_string()),
			BinOpKind::DivAssign => self.tree_builder.begin_child("/=".to_string()),
			BinOpKind::RemAssign => self.tree_builder.begin_child("%=".to_string()),
			BinOpKind::AddAssign => self.tree_builder.begin_child("&=".to_string()),
			BinOpKind::SubAssign => self.tree_builder.begin_child("-=".to_string()),
			BinOpKind::LShiftAssign => self.tree_builder.begin_child("<<=".to_string()),
			BinOpKind::RShiftAssign => self.tree_builder.begin_child(">>=".to_string()),
			BinOpKind::AmpAssign => self.tree_builder.begin_child("&=".to_string()),
			BinOpKind::XOrAssign => self.tree_builder.begin_child("^=".to_string()),
			BinOpKind::OrAssign => self.tree_builder.begin_child("|=".to_string()),
			BinOpKind::Comma => self.tree_builder.begin_child(",".to_string()),
			BinOpKind::Shl => self.tree_builder.begin_child("<<".to_string()),
			BinOpKind::Shr => self.tree_builder.begin_child(">>".to_string()),
			BinOpKind::LessEqual => self.tree_builder.begin_child("<=".to_string()),
			BinOpKind::GreatEqual => self.tree_builder.begin_child(">=".to_string()),
			BinOpKind::Less => self.tree_builder.begin_child("<".to_string()),
			BinOpKind::Great => self.tree_builder.begin_child(">".to_string()),
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
