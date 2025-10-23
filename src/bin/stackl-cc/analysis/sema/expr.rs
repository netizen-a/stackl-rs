use crate::diagnostics::*;
use crate::symbol_table as sym;
use crate::{
	analysis::{
		syn,
		tok::{
			Const,
			IntegerConstant,
		},
	},
	data_type::*,
};

impl super::SemanticParser {
	pub(super) fn expr(&mut self, expr: &mut syn::Expr, in_func: bool, mut_self: bool) -> DataType {
		match expr {
			syn::Expr::Paren(inner) => {
				self.tree_builder.begin_child("( expression )".to_string());
				let result = self.expr(inner, in_func, mut_self);
				self.tree_builder.end_child();
				result
			}
			syn::Expr::Ident(inner) => self.expr_identifier(inner, in_func, mut_self),
			syn::Expr::Const(inner) => self.expr_const(inner, mut_self),
			syn::Expr::StrLit(_inner) => DataType::POISON,
			syn::Expr::UnaryPrefix(unary) => self.expr_prefix(unary, in_func, mut_self),
			syn::Expr::UnaryPostfix(unary) => self.expr_postfix(unary, in_func, mut_self),
			syn::Expr::Binary(binary) => self.expr_binary(binary, in_func, mut_self),
			syn::Expr::Ternary(ternary) => self.expr_ternary(ternary, in_func, mut_self),
			syn::Expr::CompoundLiteral(_, _) => DataType::POISON,
			syn::Expr::Sizeof(_) => DataType::POISON,
			syn::Expr::Cast(kind, expr) => self.expr_cast(kind, expr, in_func, mut_self),
			syn::Expr::Error => unreachable!(),
		}
	}

	fn expr_cast(
		&mut self,
		kind: &syn::CastKind,
		expr: &mut syn::Expr,
		in_func: bool,
		mut_self: bool,
	) -> DataType {
		if self.print_ast {
			match kind {
				syn::CastKind::BitCast => self.tree_builder.begin_child("bit-cast".to_string()),
				syn::CastKind::FnToPtr => self.tree_builder.begin_child("fn-to-ptr".to_string()),
				syn::CastKind::Trunc => self.tree_builder.begin_child("trunc".to_string()),
				syn::CastKind::ZExt => self.tree_builder.begin_child("z-ext".to_string()),
				syn::CastKind::SExt => self.tree_builder.begin_child("s-ext".to_string()),
				syn::CastKind::FpTrunc => self.tree_builder.begin_child("fp-trunc".to_string()),
				syn::CastKind::FpExt => self.tree_builder.begin_child("fp-ext".to_string()),
				syn::CastKind::PtrToInt => self.tree_builder.begin_child("ptr-to-int".to_string()),
				syn::CastKind::IntToPtr => self.tree_builder.begin_child("int-to-ptr".to_string()),
				syn::CastKind::LValueToRValue => {
					self.tree_builder.begin_child("lval-to-rval".to_string())
				}

				syn::CastKind::UIToFP => self.tree_builder.begin_child("ui-to-fp".to_string()),
				syn::CastKind::SIToFP => self.tree_builder.begin_child("si-to-fp".to_string()),
				syn::CastKind::FPToUI => self.tree_builder.begin_child("fp-to-ui".to_string()),
				syn::CastKind::FPToSI => self.tree_builder.begin_child("fp-to-si".to_string()),
				syn::CastKind::Explicit(_) => self.tree_builder.begin_child("explicit".to_string()),
			};
		}
		let result = self.expr(expr, in_func, mut_self);
		if self.print_ast {
			self.tree_builder.end_child();
		}
		result
	}

	pub(super) fn is_l_value(&self, expr: &syn::Expr) -> bool {
		match expr {
			syn::Expr::Paren(inner) => self.is_l_value(inner),
			syn::Expr::Ident(inner) => {
				if let Some(entry) = self.ordinary_table.global_lookup(&inner.name) {
					!matches!(entry.storage, sym::StorageClass::Constant)
				} else {
					todo!("is_l_value: undeclared identifier")
				}
			}
			syn::Expr::StrLit(_) => true,
			syn::Expr::UnaryPrefix(unary) => matches!(unary.op, syn::Prefix::Star),
			_ => false,
		}
	}

	fn expr_identifier(
		&mut self,
		ident: &mut syn::Identifier,
		in_func: bool,
		mut_self: bool,
	) -> DataType {
		let span = ident.to_span();
		let (actual_line, reported_line, col) = self.diagnostics.get_location(&span).unwrap();
		let maybe = self.ordinary_table.global_lookup(&ident.name);
		if let Some(entry) = maybe {
			if mut_self {
				self.tree_builder.add_empty_child(format!(
					"identifier <line:{actual_line}:{reported_line}, col:{col}> `{}` '{}'",
					ident.name, entry.data_type
				));
			}
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
	pub(super) fn expr_prefix(
		&mut self,
		unary: &mut syn::UnaryPrefix,
		in_func: bool,
		mut_self: bool,
	) -> DataType {
		let mut result = DataType::POISON;
		match &unary.op {
			syn::Prefix::Amp => {
				self.tree_builder.begin_child("expr-prefix &".to_string());
				let inner_type = self.expr(&mut *unary.expr, in_func, mut_self);
				if !inner_type.is_poisoned() {
					result = DataType {
						kind: TypeKind::Pointer(Box::new(inner_type)),
						qual: Default::default(),
					}
				}
			}
			syn::Prefix::Star => {
				self.tree_builder.begin_child("expr-prefix *".to_string());
				let inner_type = self.expr(&mut *unary.expr, in_func, mut_self);
				if !inner_type.is_poisoned() {
					result = DataType {
						kind: inner_type.kind,
						qual: Default::default(),
					}
				}
			}
			other => todo!("{other:?}"),
		}
		self.tree_builder.end_child();
		result
	}
	pub(super) fn expr_postfix(
		&mut self,
		unary: &mut syn::UnaryPostfix,
		in_func: bool,
		mut_self: bool,
	) -> DataType {
		let _ = match unary.op {
			syn::Postfix::Array(_) => self.tree_builder.begin_child("postfix `[ ]`".to_string()),
			syn::Postfix::ArgExprList(_) => {
				self.tree_builder.begin_child("postfix `( )`".to_string())
			}
			syn::Postfix::Dot(_) => self.tree_builder.begin_child("postfix `.`".to_string()),
			syn::Postfix::Arrow(_) => self.tree_builder.begin_child("postifx `->`".to_string()),
			syn::Postfix::Inc => self.tree_builder.begin_child("postfix `++`".to_string()),
			syn::Postfix::Dec => self.tree_builder.begin_child("postfix `--`".to_string()),
		};
		self.expr(&mut *unary.expr, in_func, mut_self);
		self.tree_builder.end_child();
		DataType::POISON
	}
	pub(super) fn expr_binary(
		&mut self,
		binary: &mut syn::ExprBinary,
		in_func: bool,
		mut_self: bool,
	) -> DataType {
		if self.print_ast {
			match &binary.op.kind {
				syn::BinOpKind::Mul => self.tree_builder.begin_child("*".to_string()),
				syn::BinOpKind::Div => self.tree_builder.begin_child("/".to_string()),
				syn::BinOpKind::Rem => self.tree_builder.begin_child("%".to_string()),
				syn::BinOpKind::Sub => self.tree_builder.begin_child("-".to_string()),
				syn::BinOpKind::Add => self.tree_builder.begin_child("+".to_string()),
				syn::BinOpKind::NotEqual => self.tree_builder.begin_child("!=".to_string()),
				syn::BinOpKind::Equal => self.tree_builder.begin_child("==".to_string()),
				syn::BinOpKind::And => self.tree_builder.begin_child("&".to_string()),
				syn::BinOpKind::XOr => self.tree_builder.begin_child("^".to_string()),
				syn::BinOpKind::Or => self.tree_builder.begin_child("|".to_string()),
				syn::BinOpKind::LogicalAnd => self.tree_builder.begin_child("&&".to_string()),
				syn::BinOpKind::LogicalOr => self.tree_builder.begin_child("||".to_string()),
				syn::BinOpKind::Assign => self.tree_builder.begin_child("=".to_string()),
				syn::BinOpKind::MulAssign => self.tree_builder.begin_child("*=".to_string()),
				syn::BinOpKind::DivAssign => self.tree_builder.begin_child("/=".to_string()),
				syn::BinOpKind::RemAssign => self.tree_builder.begin_child("%=".to_string()),
				syn::BinOpKind::AddAssign => self.tree_builder.begin_child("+=".to_string()),
				syn::BinOpKind::SubAssign => self.tree_builder.begin_child("-=".to_string()),
				syn::BinOpKind::LShiftAssign => self.tree_builder.begin_child("<<=".to_string()),
				syn::BinOpKind::RShiftAssign => self.tree_builder.begin_child(">>=".to_string()),
				syn::BinOpKind::AmpAssign => self.tree_builder.begin_child("&=".to_string()),
				syn::BinOpKind::XOrAssign => self.tree_builder.begin_child("^=".to_string()),
				syn::BinOpKind::OrAssign => self.tree_builder.begin_child("|=".to_string()),
				syn::BinOpKind::Comma => self.tree_builder.begin_child(",".to_string()),
				syn::BinOpKind::Shl => self.tree_builder.begin_child("<<".to_string()),
				syn::BinOpKind::Shr => self.tree_builder.begin_child(">>".to_string()),
				syn::BinOpKind::LessEqual => self.tree_builder.begin_child("<=".to_string()),
				syn::BinOpKind::GreatEqual => self.tree_builder.begin_child(">=".to_string()),
				syn::BinOpKind::Less => self.tree_builder.begin_child("<".to_string()),
				syn::BinOpKind::Great => self.tree_builder.begin_child(">".to_string()),
			};
		}
		let is_print = self.print_ast;
		self.print_ast = false;
		let mut l_type = self.expr(&mut *binary.left, in_func, true);
		let mut r_type = self.expr(&mut *binary.right, in_func, true);
		self.print_ast = is_print;

		// add implicit casts to the ast.
		let result = if mut_self {
			let l_score =
				self.convert_type(&mut binary.left, &l_type, &r_type, binary.op.to_span());
			let r_score =
				self.convert_type(&mut binary.right, &r_type, &l_type, binary.op.to_span());
			if l_score <= r_score {
				l_type
			} else {
				r_type
			}
		} else {
			DataType::POISON
		};

		if self.print_ast {
			self.expr(&mut *binary.left, in_func, false);
			self.expr(&mut *binary.right, in_func, false);
			self.tree_builder.end_child();
		}
		result
	}
	pub(super) fn expr_ternary(
		&mut self,
		ternary: &mut syn::ExprTernary,
		in_func: bool,
		mut_self: bool,
	) -> DataType {
		self.tree_builder.begin_child("ternary `?:`".to_string());
		self.expr(&mut *ternary.expr_cond, in_func, mut_self);
		self.expr(&mut *ternary.expr_then, in_func, mut_self);
		self.expr(&mut *ternary.expr_else, in_func, mut_self);
		self.tree_builder.end_child();
		DataType::POISON
	}
	pub(super) fn expr_const(&mut self, constant: &mut Const, mut_self: bool) -> DataType {
		match constant {
			Const::Integer(IntegerConstant::I32(inner)) => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'int'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::I32),
					qual: Default::default(),
				}
			}
			Const::Integer(IntegerConstant::U32(inner)) => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'unsigned int'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::U32),
					qual: Default::default(),
				}
			}
			Const::Integer(IntegerConstant::I64(inner)) => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'long'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::I64),
					qual: Default::default(),
				}
			}
			Const::Integer(IntegerConstant::U64(inner)) => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'unsigned long'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::U64),
					qual: Default::default(),
				}
			}
			Const::Integer(IntegerConstant::I128(inner)) => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'long long'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::I128),
					qual: Default::default(),
				}
			}
			Const::Integer(IntegerConstant::U128(inner)) => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'unsigned long long'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::U128),
					qual: Default::default(),
				}
			}
			other => DataType::POISON,
		}
	}
}
