use crate::{analysis::{syn::*, tok::{Const, IntegerConstant}}, diagnostics::ToSpan};
use crate::data_types as dtype;

impl super::SemanticParser {
	pub(super) fn expr(&mut self, expr: &mut Expr) -> dtype::DataType {
		use Expr::*;
		match expr {
			Paren(inner) => {
				self.tree_builder.begin_child("( expression )".to_string());
				let result = self.expr(inner);
				self.tree_builder.end_child();
				result
			},
			Ident(inner) => {
				self.tree_builder.add_empty_child(format!("identifier {}", inner.name));
				dtype::DataType::POISON
			},
			Const(inner) => self.expr_const(inner),
			StrLit(_inner) => dtype::DataType::POISON,
			UnaryPrefix(unary) => self.expr_prefix(unary),
			UnaryPostfix(unary) => self.expr_postfix(unary),
			Binary(binary) => self.expr_binary(binary),
			Ternary(ternary) => self.expr_ternary(ternary),
			CompoundLiteral(_, _) => dtype::DataType::POISON,
			Sizeof(_) => dtype::DataType::POISON,
		}
	}
	pub(super) fn expr_prefix(&mut self, unary: &mut UnaryPrefix) -> dtype::DataType {
		let mut result = dtype::DataType::POISON;
		match unary.op {
			Prefix::Amp => {
				self.tree_builder.begin_child("expr-prefix &".to_string());
				let inner_type = self.expr(&mut *unary.expr);
				if !inner_type.is_poisoned() {
					let kind = dtype::TypeKind::Pointer(dtype::PtrType(Box::new(inner_type)));
					result = dtype::DataType{
						kind,
						qual: Default::default()
					}
				}
			},
			_ => todo!()
		}
		self.tree_builder.end_child();
		result
	}
	pub(super) fn expr_postfix(&mut self, unary: &mut UnaryPostfix) -> dtype::DataType {
		let _ = match unary.op {
			Postfix::Array(_) => self.tree_builder.begin_child("postfix `[ ]`".to_string()),
			Postfix::ArgExprList(_) => self.tree_builder.begin_child("postfix `( )`".to_string()),
			Postfix::Dot(_) => self.tree_builder.begin_child("postfix `.`".to_string()),
			Postfix::Arrow(_) => self.tree_builder.begin_child("postifx `->`".to_string()),
			Postfix::Inc => self.tree_builder.begin_child("postfix `++`".to_string()),
			Postfix::Dec => self.tree_builder.begin_child("postfix `--`".to_string()),
		};
		self.expr(&mut *unary.expr);
		self.tree_builder.end_child();
		dtype::DataType::POISON
	}
	pub(super) fn expr_binary(&mut self, binary: &mut ExprBinary) -> dtype::DataType {
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
		let l_type = self.expr(&mut *binary.left);
		let r_type = self.expr(&mut *binary.right);
		self.tree_builder.end_child();
		match self.dtype_eq(&l_type, &r_type, binary.op.to_span()) {
			Ok(cond) => if cond {
				l_type
			} else {
				let Some((_,_)) = self.try_convert(&binary.left, r_type) else {
					return dtype::DataType::POISON;
				};
				let Some((_,_)) = self.try_convert(&binary.right, l_type) else {
					return dtype::DataType::POISON;
				};
				todo!()
			}
			Err(poison) => poison
		}
	}
	pub(super) fn expr_ternary(&mut self, ternary: &mut ExprTernary) -> dtype::DataType {
		self.tree_builder.begin_child("ternary `?:`".to_string());
		self.expr(&mut *ternary.expr_cond);
		self.expr(&mut *ternary.expr_then);
		self.expr(&mut *ternary.expr_else);
		self.tree_builder.end_child();
		dtype::DataType::POISON
	}
	pub(super) fn expr_const(&mut self, constant: &mut Const) -> dtype::DataType {
		match constant {
			Const::Integer(IntegerConstant::I32(inner)) => {
				self.tree_builder.add_empty_child(format!("constant `{inner}` <signed int>"));
				dtype::DataType {
					kind: dtype::TypeKind::Scalar(dtype::ScalarType::I32),
					qual: Default::default(),
				}
			},
			Const::Integer(IntegerConstant::U32(inner)) => {
				self.tree_builder.add_empty_child(format!("constant `{inner}` <unsigned int>"));
				dtype::DataType {
					kind: dtype::TypeKind::Scalar(dtype::ScalarType::U32),
					qual: Default::default(),
				}
			},
			Const::Integer(IntegerConstant::I64(inner)) => {
				self.tree_builder.add_empty_child(format!("constant `{inner}` <signed long int>"));
				dtype::DataType {
					kind: dtype::TypeKind::Scalar(dtype::ScalarType::I64),
					qual: Default::default(),
				}
			},
			Const::Integer(IntegerConstant::U64(inner)) => {
				self.tree_builder.add_empty_child(format!("constant `{inner}` <unsigned long int>"));
				dtype::DataType {
					kind: dtype::TypeKind::Scalar(dtype::ScalarType::U64),
					qual: Default::default(),
				}
			},
			Const::Integer(IntegerConstant::I128(inner)) => {
				self.tree_builder.add_empty_child(format!("constant `{inner}` <signed long long int>"));
				dtype::DataType {
					kind: dtype::TypeKind::Scalar(dtype::ScalarType::I128),
					qual: Default::default(),
				}
			},
			Const::Integer(IntegerConstant::U128(inner)) => {
				self.tree_builder.add_empty_child(format!("constant `{inner}` <unsigned long long int>"));
				dtype::DataType {
					kind: dtype::TypeKind::Scalar(dtype::ScalarType::U128),
					qual: Default::default(),
				}
			},
			other => dtype::DataType::POISON,
		}
	}
}
