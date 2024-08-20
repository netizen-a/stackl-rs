// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::analysis::syn::Constant;
use crate::diagnostics::*;
use crate::symtab as sym;
use crate::{
	analysis::syn,
	data_type::*,
};

pub struct ExprContext {
	pub in_func: bool,
	pub is_mut: bool,
	pub enabled_diag: bool,
}

impl super::SemanticParser<'_> {
	pub(super) fn expr(&mut self, expr: &mut syn::Expr, context: &ExprContext) -> DataType {
		match expr {
			syn::Expr::Paren(inner) => {
				if self.print_ast {
					self.tree_builder.begin_child("( expression )".to_string());
				}
				let result = self.expr(inner, context);
				if self.print_ast {
					self.tree_builder.end_child();
				}
				result
			}
			syn::Expr::Ident(inner) => self.expr_identifier(inner, context),
			syn::Expr::Const(inner) => self.expr_const(inner, context),
			syn::Expr::StrLit(inner) => self.expr_string_literal(inner),
			syn::Expr::UnaryPrefix(unary) => self.expr_prefix(unary, context),
			syn::Expr::UnaryPostfix(unary) => self.expr_postfix(unary, context),
			syn::Expr::Binary(binary) => self.expr_binary(binary, context),
			syn::Expr::Ternary(ternary) => self.expr_ternary(ternary, context),
			syn::Expr::CompoundLiteral(_, _) => DataType::POISON,
			syn::Expr::Sizeof(_) => DataType::POISON,
			syn::Expr::Cast(inner) => self.expr_cast(inner, context),
		}
	}

	#[inline]
	pub fn expr_no_print(&mut self, expr: &mut syn::Expr, context: &ExprContext) -> DataType {
		let is_print = self.print_ast;
		self.print_ast = false;
		let result = self.expr(expr, context);
		self.print_ast = is_print;
		result
	}

	fn expr_string_literal(&mut self, literal: &mut syn::StringLiteral) -> DataType {
		let result = DataType {
			kind: TypeKind::Array(ArrayType {
				component: Box::new(DataType {
					kind: TypeKind::Scalar(ScalarType::SChar),
					qual: Default::default(),
				}),
				length: ArrayLength::Fixed((literal.seq.len() + 1) as u32),
				is_decayed: false,
				has_static: false,
			}),
			qual: Default::default(),
		};
		if self.print_ast {
			let seq = literal.seq.replace("\n", "\\n").replace('"', "\\\"");
			self.tree_builder
				.add_empty_child(format!("string-literal \"{seq}\" '{result}'"));
		}
		result
	}

	fn expr_cast(&mut self, cast: &mut syn::ExprCast, context: &ExprContext) -> DataType {
		let from_type = self.expr_no_print(&mut cast.expr, context);

		let to_type: DataType = match &mut cast.kind {
			syn::CastKind::BitCast => {
				todo!("cast bit-cast")
			}
			syn::CastKind::FnToPtr => {
				todo!("cast fn-to-ptr")
			}
			syn::CastKind::Trunc(inner) => DataType {
				kind: *inner.clone(),
				qual: Default::default(),
			},
			syn::CastKind::ZExt(inner) => DataType {
				kind: *inner.clone(),
				qual: Default::default(),
			},
			syn::CastKind::SExt(inner) => DataType {
				kind: *inner.clone(),
				qual: Default::default(),
			},
			syn::CastKind::FpTrunc(inner) => DataType {
				kind: *inner.clone(),
				qual: Default::default(),
			},
			syn::CastKind::FpExt(inner) => DataType {
				kind: *inner.clone(),
				qual: Default::default(),
			},
			syn::CastKind::PtrToInt => {
				todo!("cast ptr-to-int")
			}
			syn::CastKind::IntToPtr => {
				todo!("cast int-to-ptr")
			}
			syn::CastKind::LValueToRValue => from_type.clone(),
			syn::CastKind::IntToBool => DataType {
				kind: TypeKind::Scalar(ScalarType::Bool),
				qual: Default::default(),
			},
			syn::CastKind::UIToFP(inner) => DataType {
				kind: *inner.clone(),
				qual: Default::default(),
			},
			syn::CastKind::SIToFP(inner) => DataType {
				kind: *inner.clone(),
				qual: Default::default(),
			},
			syn::CastKind::FPToUI(inner) => DataType {
				kind: *inner.clone(),
				qual: Default::default(),
			},
			syn::CastKind::FPToSI(inner) => DataType {
				kind: *inner.clone(),
				qual: Default::default(),
			},
			syn::CastKind::Explicit(type_name) => {
				let maybe = self.specifiers_dtype(&mut type_name.specifiers, context.in_func);
				self.unwrap_or_poison(maybe, None, cast.to_span())
			}
		};

		if self.print_ast {
			match &cast.kind {
				syn::CastKind::BitCast => self
					.tree_builder
					.begin_child(format!("cast bit-cast '{from_type}' -> '{to_type}'")),
				syn::CastKind::FnToPtr => self
					.tree_builder
					.begin_child(format!("cast fn-to-ptr '{from_type}' -> '{to_type}'")),
				syn::CastKind::Trunc(_) => self
					.tree_builder
					.begin_child(format!("cast trunc '{from_type}' -> '{to_type}'")),
				syn::CastKind::ZExt(_) => self
					.tree_builder
					.begin_child(format!("cast z-ext '{from_type}' -> '{to_type}'")),
				syn::CastKind::SExt(_) => self
					.tree_builder
					.begin_child(format!("cast s-ext '{from_type}' -> '{to_type}'")),
				syn::CastKind::FpTrunc(_) => self
					.tree_builder
					.begin_child(format!("cast fp-trunc '{from_type}' -> '{to_type}'")),
				syn::CastKind::FpExt(_) => self
					.tree_builder
					.begin_child(format!("cast fp-ext '{from_type}' -> '{to_type}'")),
				syn::CastKind::PtrToInt => self
					.tree_builder
					.begin_child(format!("cast ptr-to-int '{from_type}' -> '{to_type}'")),
				syn::CastKind::IntToPtr => self
					.tree_builder
					.begin_child(format!("cast int-to-ptr '{from_type}' -> '{to_type}'")),
				syn::CastKind::LValueToRValue => {
					self.tree_builder.begin_child(format!("cast lval-to-rval"))
				}
				syn::CastKind::IntToBool => self
					.tree_builder
					.begin_child(format!("cast int-to-bool '{from_type}' -> '{to_type}'")),
				syn::CastKind::UIToFP(_) => self
					.tree_builder
					.begin_child(format!("cast ui-to-fp '{from_type}' -> '{to_type}'")),
				syn::CastKind::SIToFP(_) => self
					.tree_builder
					.begin_child(format!("cast si-to-fp '{from_type}' -> '{to_type}'")),
				syn::CastKind::FPToUI(_) => self
					.tree_builder
					.begin_child(format!("cast fp-to-ui '{from_type}' -> '{to_type}'")),
				syn::CastKind::FPToSI(_) => self
					.tree_builder
					.begin_child(format!("cast fp-to-si '{from_type}' -> '{to_type}'")),
				syn::CastKind::Explicit(type_name) => self
					.tree_builder
					.begin_child(format!("cast explicit '{from_type}' -> '{to_type}'")),
			};
		}

		if self.print_ast {
			self.expr(&mut cast.expr, context);
			self.tree_builder.end_child();
		}
		to_type
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
			syn::Expr::StrLit(_) => false,
			syn::Expr::UnaryPrefix(unary) => matches!(unary.op.kind, syn::PrefixKind::Star),
			_ => false,
		}
	}

	fn expr_identifier(&mut self, ident: &mut syn::Identifier, context: &ExprContext) -> DataType {
		let span = ident.to_span();
		let (_, reported_line, col) = self.diagnostics.get_location(&span).unwrap();
		let maybe = self.ordinary_table.global_lookup(&ident.name);
		if let Some(entry) = maybe {
			if self.print_ast {
				self.tree_builder.add_empty_child(format!(
					"identifier <line:{reported_line}, col:{col}> `{}` '{}'",
					ident.name, entry.data_type
				));
			}
			if context.enabled_diag && !context.in_func && !entry.is_constant() {
				let error = Diagnostic::error(DiagKind::InitializerNotConst, span);
				self.diagnostics.push(error);
			}
		} else {
			let kind = DiagKind::SymbolUndeclared {
				name: ident.name.clone(),
				in_func: context.in_func,
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
		context: &ExprContext,
	) -> DataType {
		let mut result = DataType::POISON;
		match &unary.op.kind {
			syn::PrefixKind::Amp => {
				self.tree_builder.begin_child("expr-prefix &".to_string());
				let inner_type = self.expr(&mut *unary.expr, context);
				if !inner_type.is_poisoned() {
					result = DataType {
						kind: TypeKind::Pointer(Box::new(inner_type)),
						qual: Default::default(),
					}
				}
			}
			syn::PrefixKind::Star => {
				self.tree_builder.begin_child("expr-prefix *".to_string());
				let inner_type = self.expr(&mut *unary.expr, context);
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
		context: &ExprContext,
	) -> DataType {
		let _ = match unary.op.kind {
			syn::PostfixKind::Array(_) => {
				self.tree_builder.begin_child("postfix `[ ]`".to_string())
			}
			syn::PostfixKind::ArgExprList(_) => {
				self.tree_builder.begin_child("postfix `( )`".to_string())
			}
			syn::PostfixKind::Dot(_) => self.tree_builder.begin_child("postfix `.`".to_string()),
			syn::PostfixKind::Arrow(_) => self.tree_builder.begin_child("postifx `->`".to_string()),
			syn::PostfixKind::Inc => self.tree_builder.begin_child("postfix `++`".to_string()),
			syn::PostfixKind::Dec => self.tree_builder.begin_child("postfix `--`".to_string()),
		};
		self.expr(&mut *unary.expr, context);
		self.tree_builder.end_child();
		DataType::POISON
	}
	pub(super) fn expr_binary(
		&mut self,
		binary: &mut syn::ExprBinary,
		context: &ExprContext,
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
		let mut l_type = self.expr_no_print(&mut *binary.left, context);
		let mut r_type = self.expr_no_print(&mut *binary.right, context);

		// add implicit casts to the ast.
		let result = if context.is_mut {
			let l_score =
				self.convert_type(&mut binary.left, &l_type, &r_type, binary.op.to_span());
			let r_score =
				self.convert_type(&mut binary.right, &r_type, &l_type, binary.op.to_span());
			if l_score <= r_score { l_type } else { r_type }
		} else {
			DataType::POISON
		};

		if self.print_ast {
			let print_ast_context = ExprContext {
				in_func: context.in_func,
				is_mut: false,
				enabled_diag: context.enabled_diag,
			};
			self.expr(&mut *binary.left, &print_ast_context);
			self.expr(&mut *binary.right, &print_ast_context);
			self.tree_builder.end_child();
		}
		result
	}
	pub(super) fn expr_ternary(
		&mut self,
		ternary: &mut syn::ExprTernary,
		context: &ExprContext,
	) -> DataType {
		self.tree_builder.begin_child("ternary `?:`".to_string());
		self.expr(&mut *ternary.expr_cond, context);
		self.expr(&mut *ternary.expr_then, context);
		self.expr(&mut *ternary.expr_else, context);
		self.tree_builder.end_child();
		DataType::POISON
	}
	pub(super) fn expr_const(
		&mut self,
		constant: &mut Constant,
		context: &ExprContext,
	) -> DataType {
		use syn::ConstantKind::*;
		use syn::FloatingKind::*;
		use syn::IntegerKind::*;
		match constant {
			Constant {
				kind: Integer(I32(inner)),
				..
			} => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'int'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::SInt),
					qual: Default::default(),
				}
			}
			Constant {
				kind: Integer(U32(inner)),
				..
			} => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'unsigned int'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::UInt),
					qual: Default::default(),
				}
			}
			Constant {
				kind: Integer(I64(inner)),
				..
			} => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'long'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::SLong),
					qual: Default::default(),
				}
			}
			Constant {
				kind: Integer(U64(inner)),
				..
			} => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'unsigned long'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::ULong),
					qual: Default::default(),
				}
			}
			Constant {
				kind: Integer(I128(inner)),
				..
			} => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'long long'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::SLong2),
					qual: Default::default(),
				}
			}
			Constant {
				kind: Integer(U128(inner)),
				..
			} => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'unsigned long long'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::ULong2),
					qual: Default::default(),
				}
			}
			Constant {
				kind: Floating(Float(inner)),
				..
			} => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'float'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::Float),
					qual: Default::default(),
				}
			}
			Constant {
				kind: Floating(Double(inner)),
				..
			} => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'double'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::Double),
					qual: Default::default(),
				}
			}
			Constant {
				kind: Floating(LongDouble(inner)),
				..
			} => {
				if self.print_ast {
					self.tree_builder
						.add_empty_child(format!("constant `{inner}` 'long double'"));
				}
				DataType {
					kind: TypeKind::Scalar(ScalarType::LongDouble),
					qual: Default::default(),
				}
			}
			other => DataType::POISON,
		}
	}
}
