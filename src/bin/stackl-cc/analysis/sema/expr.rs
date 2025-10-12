use crate::diagnostics::*;
use crate::symbol_table as sym;
use crate::{
	analysis::{
		syn::*,
		tok::{Const, IntegerConstant},
	},
	data_type::*,
};

impl super::SemanticParser {
	pub(super) fn expr(&mut self, expr: &mut Expr, in_func: bool) -> DataType {
		match expr {
			Expr::Paren(inner) => {
				self.tree_builder.begin_child("( expression )".to_string());
				let result = self.expr(inner, in_func);
				self.tree_builder.end_child();
				result
			}
			Expr::Ident(inner) => self.expr_identifier(inner, in_func),
			Expr::Const(inner) => self.expr_const(inner),
			Expr::StrLit(_inner) => DataType::POISON,
			Expr::UnaryPrefix(unary) => self.expr_prefix(unary, in_func),
			Expr::UnaryPostfix(unary) => self.expr_postfix(unary, in_func),
			Expr::Binary(binary) => self.expr_binary(binary, in_func),
			Expr::Ternary(ternary) => self.expr_ternary(ternary, in_func),
			Expr::CompoundLiteral(_, _) => DataType::POISON,
			Expr::Sizeof(_) => DataType::POISON,
		}
	}

	pub(super) fn is_l_value(&mut self, expr: &mut Expr) -> bool {
		match expr {
			Expr::Paren(inner) => self.is_l_value(inner),
			Expr::Ident(inner) => {
				todo!()
			}
			Expr::StrLit(_) => true,
			Expr::UnaryPrefix(unary) => matches!(unary.op, Prefix::Star),
			_ => false
		}
	}

	fn expr_identifier(&mut self, ident: &mut Identifier, in_func: bool) -> DataType {
		let span = ident.to_span();
		let (actual_line, reported_line, col) = self.diagnostics.get_location(&span).unwrap();
		let maybe = self.ordinary_table.global_lookup(&ident.name);
		if let Some(entry) = maybe {
			self.tree_builder.add_empty_child(format!(
				"identifier <line:{actual_line}:{reported_line}, col:{col}> `{}` '{}'",
				ident.name, entry.data_type
			));
			if !in_func && !entry.is_constant() {
				let kind = DiagKind::InitializerNotConst;
				let error = Diagnostic::error(kind, span);
				self.diagnostics.push(error);
			}
		} else {
			let kind = DiagKind::SymbolUndeclared {
				name: ident.name.clone(),
				in_func,
			};
			let error = Diagnostic::error(kind, span);
			self.diagnostics.push(error);
			self.tree_builder
				.add_empty_child(format!("identifier `{}` '<unknown>'", ident.name));
		}
		DataType::POISON
	}
	pub(super) fn expr_prefix(&mut self, unary: &mut UnaryPrefix, in_func: bool) -> DataType {
		let mut result = DataType::POISON;
		match unary.op {
			Prefix::Amp => {
				self.tree_builder.begin_child("expr-prefix &".to_string());
				let inner_type = self.expr(&mut *unary.expr, in_func);
				if !inner_type.is_poisoned() {
					let kind = TypeKind::Pointer(Box::new(inner_type));
					result = DataType {
						kind,
						qual: Default::default(),
					}
				}
			}
			_ => todo!(),
		}
		self.tree_builder.end_child();
		result
	}
	pub(super) fn expr_postfix(&mut self, unary: &mut UnaryPostfix, in_func: bool) -> DataType {
		let _ = match unary.op {
			Postfix::Array(_) => self.tree_builder.begin_child("postfix `[ ]`".to_string()),
			Postfix::ArgExprList(_) => self.tree_builder.begin_child("postfix `( )`".to_string()),
			Postfix::Dot(_) => self.tree_builder.begin_child("postfix `.`".to_string()),
			Postfix::Arrow(_) => self.tree_builder.begin_child("postifx `->`".to_string()),
			Postfix::Inc => self.tree_builder.begin_child("postfix `++`".to_string()),
			Postfix::Dec => self.tree_builder.begin_child("postfix `--`".to_string()),
		};
		self.expr(&mut *unary.expr, in_func);
		self.tree_builder.end_child();
		DataType::POISON
	}
	pub(super) fn expr_binary(&mut self, binary: &mut ExprBinary, in_func: bool) -> DataType {
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
		let l_type = self.expr(&mut *binary.left, in_func);
		let r_type = self.expr(&mut *binary.right, in_func);
		self.tree_builder.end_child();
		match self.dtype_eq(&l_type, &r_type, binary.op.to_span()) {
			Ok(cond) => {
				if cond {
					l_type
				} else {
					let Some((_, _)) = self.try_convert(&binary.left, r_type) else {
						return DataType::POISON;
					};
					let Some((_, _)) = self.try_convert(&binary.right, l_type) else {
						return DataType::POISON;
					};
					todo!()
				}
			}
			Err(poison) => poison,
		}
	}
	pub(super) fn expr_ternary(&mut self, ternary: &mut ExprTernary, in_func: bool) -> DataType {
		self.tree_builder.begin_child("ternary `?:`".to_string());
		self.expr(&mut *ternary.expr_cond, in_func);
		self.expr(&mut *ternary.expr_then, in_func);
		self.expr(&mut *ternary.expr_else, in_func);
		self.tree_builder.end_child();
		DataType::POISON
	}
	pub(super) fn expr_const(&mut self, constant: &mut Const) -> DataType {
		match constant {
			Const::Integer(IntegerConstant::I32(inner)) => {
				self.tree_builder
					.add_empty_child(format!("constant `{inner}` 'int'"));
				DataType {
					kind: TypeKind::Scalar(ScalarType::I32),
					qual: Default::default(),
				}
			}
			Const::Integer(IntegerConstant::U32(inner)) => {
				self.tree_builder
					.add_empty_child(format!("constant `{inner}` 'unsigned int'"));
				DataType {
					kind: TypeKind::Scalar(ScalarType::U32),
					qual: Default::default(),
				}
			}
			Const::Integer(IntegerConstant::I64(inner)) => {
				self.tree_builder
					.add_empty_child(format!("constant `{inner}` 'long'"));
				DataType {
					kind: TypeKind::Scalar(ScalarType::I64),
					qual: Default::default(),
				}
			}
			Const::Integer(IntegerConstant::U64(inner)) => {
				self.tree_builder
					.add_empty_child(format!("constant `{inner}` 'unsigned long'"));
				DataType {
					kind: TypeKind::Scalar(ScalarType::U64),
					qual: Default::default(),
				}
			}
			Const::Integer(IntegerConstant::I128(inner)) => {
				self.tree_builder
					.add_empty_child(format!("constant `{inner}` 'long long'"));
				DataType {
					kind: TypeKind::Scalar(ScalarType::I128),
					qual: Default::default(),
				}
			}
			Const::Integer(IntegerConstant::U128(inner)) => {
				self.tree_builder
					.add_empty_child(format!("constant `{inner}` 'unsigned long long'"));
				DataType {
					kind: TypeKind::Scalar(ScalarType::U128),
					qual: Default::default(),
				}
			}
			other => DataType::POISON,
		}
	}
}
