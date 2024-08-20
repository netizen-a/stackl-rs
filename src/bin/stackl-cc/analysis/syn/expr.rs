// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::f32;

use super::Identifier;
use super::decl;
use crate::analysis::syn::Constant;
use crate::analysis::syn::ConstantKind;
use crate::analysis::syn::FloatingKind;
use crate::analysis::syn::IntegerKind;
use crate::analysis::syn::StringLiteral;
use crate::analysis::tok;
use crate::data_type as dtype;
use crate::diagnostics as diag;
use crate::diagnostics::ToSpan;

pub enum ConversionError {
	OutOfRange,
	Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum CastKind {
	/// Cost: 0
	BitCast,
	/// Cost: 1
	FnToPtr,
	/// Cost: 1
	Trunc(Box<dtype::TypeKind>),
	/// Cost: 1
	ZExt(Box<dtype::TypeKind>),
	/// Cost: 1
	SExt(Box<dtype::TypeKind>),
	/// Cost: 1
	FpTrunc(Box<dtype::TypeKind>),
	/// Cost: 1
	FpExt(Box<dtype::TypeKind>),
	/// Cost: 1
	PtrToInt,
	/// Cost: 1
	IntToPtr,
	/// Cost: 1
	LValueToRValue,
	/// Cost: 1
	IntToBool,
	/// Cost: 2
	UIToFP(Box<dtype::TypeKind>),
	/// Cost: 2
	SIToFP(Box<dtype::TypeKind>),
	/// Cost: 3
	FPToUI(Box<dtype::TypeKind>),
	/// Cost: 3
	FPToSI(Box<dtype::TypeKind>),
	/// Cost: N/A
	Explicit(decl::TypeName),
}

#[derive(Debug, Clone)]
pub struct ExprCast {
	pub span: diag::Span,
	pub kind: CastKind,
	pub expr: Box<Expr>,
}

impl ToSpan for ExprCast {
	fn to_span(&self) -> diag::Span {
		self.span.clone()
	}
}

/// (6.5.17) expression
#[derive(Debug, Clone)]
pub enum Expr {
	// Paren variant is required for the AssignIf warning to work.
	Paren(Box<Expr>),
	Ident(Identifier),
	Const(Constant),
	StrLit(StringLiteral),
	UnaryPrefix(UnaryPrefix),
	UnaryPostfix(UnaryPostfix),
	Binary(ExprBinary),
	Ternary(ExprTernary),
	CompoundLiteral(decl::TypeName, decl::InitializerList),
	Sizeof(decl::TypeName),
	/// ( type-name ) expression
	Cast(ExprCast),
}

impl ToSpan for Expr {
	fn to_span(&self) -> diag::Span {
		match self {
			Self::Paren(inner) => inner.to_span(),
			Self::Ident(inner) => inner.to_span(),
			Self::Const(inner) => inner.to_span(),
			Self::StrLit(inner) => inner.to_span(),
			Self::UnaryPrefix(inner) => inner.to_span(),
			Self::UnaryPostfix(inner) => inner.to_span(),
			Self::Binary(inner) => inner.to_span(),
			Self::Ternary(inner) => inner.to_span(),
			Self::CompoundLiteral(inner, _) => inner.specifiers.to_span(),
			Self::Sizeof(inner) => inner.to_span(),
			Self::Cast(inner) => inner.to_span(),
		}
	}
}

impl Expr {
	#[inline]
	pub fn with_prefix(op: Prefix, expr: Self) -> Self {
		Self::UnaryPrefix(UnaryPrefix {
			op,
			expr: Box::new(expr),
		})
	}
	#[inline]
	pub fn with_postfix(op: Postfix, expr: Self) -> Self {
		Self::UnaryPostfix(UnaryPostfix {
			op,
			expr: Box::new(expr),
		})
	}
	#[inline]
	pub fn with_binary(
		op: BinOp,
		left: Self,
		right: Self,
		contract_int: bool,
		contract_float: bool,
	) -> Self {
		use tok::Const::{
			Floating,
			Integer,
		};
		let result = Self::Binary(ExprBinary {
			op,
			left: Box::new(left),
			right: Box::new(right),
		});
		if contract_int || contract_float {
			result.constant_fold(contract_int, contract_float)
		} else {
			result
		}
	}
	#[inline]
	pub fn with_ternary(
		cond_expr: Self,
		cond_span: diag::Span,
		then_expr: Self,
		then_span: diag::Span,
		else_expr: Self,
	) -> Self {
		Self::Ternary(ExprTernary {
			expr_cond: Box::new(cond_expr),
			cond_span,
			expr_then: Box::new(then_expr),
			then_span,
			expr_else: Box::new(else_expr),
		})
	}

	pub fn constant_fold(&self, contract_int: bool, contract_float: bool) -> Expr {
		use ConstantKind::*;
		match self {
			Self::UnaryPrefix(unary) => {
				let op = &unary.op;
				let expr = unary.expr.constant_fold(contract_int, contract_float);
				match &expr {
					Expr::Const(Constant {
						kind: Integer(rhs_int),
						span,
					}) => op.reduce_int(rhs_int, span.to_span()),
					_ => Self::UnaryPrefix(UnaryPrefix {
						op: op.clone(),
						expr: Box::new(expr),
					}),
				}
			}
			Self::UnaryPostfix(unary) => {
				let op = &unary.op;
				let expr = unary.expr.constant_fold(contract_int, contract_float);
				match &expr {
					Expr::Const(Constant {
						kind: Integer(rhs_int),
						..
					}) => {
						//op.reduce_int(rhs_int)
						todo!("postfix reduce_int")
					}
					_ => Self::UnaryPostfix(UnaryPostfix {
						op: op.clone(),
						expr: Box::new(expr),
					}),
				}
			}
			Self::Binary(binary) => {
				let left = binary.left.constant_fold(contract_int, contract_float);
				let right = binary.right.constant_fold(contract_int, contract_float);
				let op = binary.op.clone();
				match (contract_int, contract_float, &left, &right) {
					(
						true,
						_,
						Expr::Const(Constant {
							kind: Integer(lhs_int),
							span: l_span,
						}),
						Expr::Const(Constant {
							kind: Integer(rhs_int),
							span: r_span,
						}),
					) => op.constant_fold_int(
						(lhs_int, l_span.to_span()),
						(rhs_int, r_span.to_span()),
					),
					(
						true,
						_,
						Expr::Paren(expr),
						Expr::Const(Constant {
							kind: Integer(rhs_int),
							span: r_span,
						}),
					) => {
						if let Expr::Const(Constant {
							kind: Integer(lhs_int),
							span: l_span,
						}) = expr.as_ref()
						{
							op.constant_fold_int(
								(lhs_int, l_span.to_span()),
								(rhs_int, r_span.to_span()),
							)
						} else {
							self.clone()
						}
					}
					(
						true,
						_,
						Expr::Const(Constant {
							kind: Integer(lhs_int),
							span: l_span,
						}),
						Expr::Paren(expr),
					) => {
						if let Expr::Const(Constant {
							kind: Integer(rhs_int),
							span: r_span,
						}) = expr.as_ref()
						{
							op.constant_fold_int(
								(lhs_int, l_span.to_span()),
								(rhs_int, r_span.to_span()),
							)
						} else {
							self.clone()
						}
					}
					(_, _, Expr::Paren(lhs_expr), Expr::Paren(rhs_expr)) => {
						if let (
							Expr::Const(Constant {
								kind: Integer(lhs_int),
								span: l_span,
							}),
							Expr::Const(Constant {
								kind: Integer(rhs_int),
								span: r_span,
							}),
						) = (lhs_expr.as_ref(), rhs_expr.as_ref())
						{
							op.constant_fold_int(
								(lhs_int, l_span.to_span()),
								(rhs_int, r_span.to_span()),
							)
						} else {
							self.clone()
						}
					}
					(
						_,
						true,
						Expr::Const(Constant {
							kind: Floating(lhs_float),
							span: l_span,
						}),
						Expr::Const(Constant {
							kind: Floating(rhs_float),
							span: r_span,
						}),
					) => op.constant_fold_float(
						(lhs_float, l_span.to_span()),
						(rhs_float, r_span.to_span()),
					),
					_ => Self::Binary(ExprBinary {
						op: op.clone(),
						left: Box::new(left.clone()),
						right: Box::new(right.clone()),
					}),
				}
			}
			Self::Ternary(ternary) => todo!(),
			Self::Sizeof(_) => todo!(),
			_ => {
				// cannot reduce
				self.clone()
			}
		}
	}
	// TODO: get sizeof working with this
	pub fn to_u32(&mut self) -> Result<u32, ConversionError> {
		use ConstantKind::*;
		const U64_CAP: u64 = u32::MAX as u64;
		const I64_CAP: i64 = u32::MAX as i64;
		const U128_CAP: u128 = u32::MAX as u128;
		const I128_CAP: i128 = u32::MAX as i128;
		*self = self.constant_fold(true, false);
		match self {
			Self::Const(Constant {
				kind: Integer(int_const),
				..
			}) => match int_const {
				IntegerKind::U32(val) => Ok(*val),
				IntegerKind::I32(val) => match val {
					0.. => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerKind::U64(val) => match val {
					0..=U64_CAP => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerKind::I64(val) => match val {
					0..=I64_CAP => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerKind::U128(val) => match val {
					0..=U128_CAP => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerKind::I128(val) => match val {
					0..=I128_CAP => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
			},
			expr => Err(ConversionError::Expr(expr.clone())),
		}
	}
	pub fn to_i32(&mut self) -> Result<i32, ConversionError> {
		use ConstantKind::*;
		const U32_CAP: u32 = i32::MAX as u32;
		const U64_CAP: u64 = i32::MAX as u64;
		const I64_CAP: i64 = i32::MAX as i64;
		const U128_CAP: u128 = i32::MAX as u128;
		const I128_CAP: i128 = i32::MAX as i128;
		*self = self.constant_fold(true, false);
		match self {
			Self::Const(Constant {
				kind: Integer(int_const),
				..
			}) => match int_const {
				IntegerKind::U32(val) => match val {
					0..=U32_CAP => Ok(*val as i32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerKind::I32(val) => Ok(*val),
				IntegerKind::U64(val) => match val {
					0..=U64_CAP => Ok(*val as i32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerKind::I64(val) => match val {
					0..=I64_CAP => Ok(*val as i32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerKind::U128(val) => match val {
					0..=U128_CAP => Ok(*val as i32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerKind::I128(val) => match val {
					0..=I128_CAP => Ok(*val as i32),
					_ => Err(ConversionError::OutOfRange),
				},
			},
			expr => Err(ConversionError::Expr(expr.clone())),
		}
	}
}

/// (6.5.3) unary-expression
#[derive(Debug, Clone)]
pub struct UnaryPrefix {
	pub op: Prefix,
	pub expr: Box<Expr>,
}

impl ToSpan for UnaryPrefix {
	fn to_span(&self) -> diag::Span {
		self.op.to_span()
	}
}

/// (6.5.3) unary-expression
#[derive(Debug, Clone)]
pub struct UnaryPostfix {
	pub op: Postfix,
	pub expr: Box<Expr>,
}

impl ToSpan for UnaryPostfix {
	fn to_span(&self) -> diag::Span {
		self.op.to_span()
	}
}

#[derive(Debug, Clone)]
pub struct ExprBinary {
	pub op: BinOp,
	pub left: Box<Expr>,
	pub right: Box<Expr>,
}

impl ToSpan for ExprBinary {
	fn to_span(&self) -> diag::Span {
		self.op.to_span()
	}
}

#[derive(Debug, Clone)]
pub struct ExprTernary {
	pub expr_cond: Box<Expr>,
	pub cond_span: diag::Span,
	pub expr_then: Box<Expr>,
	pub then_span: diag::Span,
	pub expr_else: Box<Expr>,
}

impl ToSpan for ExprTernary {
	fn to_span(&self) -> diag::Span {
		self.cond_span.clone()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum BinOpKind {
	Mul,
	Div,
	Rem,
	Sub,
	Add,
	NotEqual,
	Equal,
	And,
	XOr,
	Or,
	LogicalAnd,
	LogicalOr,
	Assign,
	MulAssign,
	DivAssign,
	RemAssign,
	AddAssign,
	SubAssign,
	LShiftAssign,
	RShiftAssign,
	AmpAssign,
	XOrAssign,
	OrAssign,
	Comma,
	Shl,
	Shr,
	LessEqual,
	GreatEqual,
	Less,
	Great,
}

#[derive(Debug, Clone)]
pub struct BinOp {
	pub span: diag::Span,
	pub kind: BinOpKind,
}

impl ToSpan for BinOp {
	fn to_span(&self) -> diag::Span {
		self.span.clone()
	}
}

impl BinOp {
	fn constant_fold_int(
		&self,
		lhs: (&IntegerKind, diag::Span),
		rhs: (&IntegerKind, diag::Span),
	) -> Expr {
		use ConstantKind::*;
		let int_const = match (self.kind, lhs.0, rhs.0) {
			(BinOpKind::Mul, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::U32(lval.wrapping_mul(*rval))
			}
			(BinOpKind::Mul, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				IntegerKind::I32(lval.wrapping_mul(*rval))
			}
			(BinOpKind::Div, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				if *rval != 0 {
					IntegerKind::U32(lval.wrapping_sub(*rval))
				} else {
					// Undefined Behavior
					IntegerKind::U32(u32::MAX)
				}
			}
			(BinOpKind::Div, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				if *rval != 0 {
					IntegerKind::I32(lval.wrapping_sub(*rval))
				} else {
					// Undefined Behavior
					IntegerKind::I32(i32::MAX)
				}
			}
			(BinOpKind::Rem, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::U32(lval.wrapping_rem(*rval))
			}
			(BinOpKind::Rem, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				IntegerKind::I32(lval.wrapping_rem(*rval))
			}
			(BinOpKind::Sub, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::U32(lval.wrapping_sub(*rval))
			}
			(BinOpKind::Sub, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				IntegerKind::I32(lval.wrapping_sub(*rval))
			}
			(BinOpKind::Add, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::U32(lval.wrapping_add(*rval))
			}
			(BinOpKind::Add, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				IntegerKind::I32(lval.wrapping_add(*rval))
			}
			(BinOpKind::NotEqual, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::I32((*lval != *rval) as i32)
			}
			(BinOpKind::NotEqual, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				IntegerKind::I32((*lval != *rval) as i32)
			}
			(BinOpKind::Equal, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::I32((*lval == *rval) as i32)
			}
			(BinOpKind::Equal, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				IntegerKind::I32((*lval == *rval) as i32)
			}
			(BinOpKind::And, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::U32(*lval & *rval)
			}
			(BinOpKind::And, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				IntegerKind::I32(*lval & *rval)
			}
			(BinOpKind::XOr, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::U32(*lval ^ *rval)
			}
			(BinOpKind::XOr, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				IntegerKind::I32(*lval ^ *rval)
			}
			(BinOpKind::Or, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::U32(*lval | *rval)
			}
			(BinOpKind::Or, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				IntegerKind::I32(*lval | *rval)
			}
			(BinOpKind::LogicalAnd, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::I32(((*lval != 0) && (*rval != 0)) as i32)
			}
			(BinOpKind::LogicalAnd, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				IntegerKind::I32(((*lval != 0) && (*rval != 0)) as i32)
			}
			(BinOpKind::LogicalOr, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::I32(((*lval != 0) || (*rval != 0)) as i32)
			}
			(BinOpKind::LogicalOr, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				IntegerKind::I32(((*lval != 0) || (*rval != 0)) as i32)
			}
			(BinOpKind::Shl, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::U32(lval.wrapping_shl(*rval))
			}
			(BinOpKind::Shl, IntegerKind::I32(lval), IntegerKind::I32(rval)) => {
				match (*rval).try_into() {
					Ok(rval) => IntegerKind::I32(lval.wrapping_shl(rval)),
					Err(_) => IntegerKind::I32(lval.wrapping_shr((-rval) as u32)),
				}
			}
			(BinOpKind::Shl, IntegerKind::I32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::I32(lval.wrapping_shl(*rval))
			}
			(BinOpKind::Shr, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::U32(lval.wrapping_shr(*rval))
			}
			(BinOpKind::LessEqual, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::I32((*lval <= *rval) as i32)
			}
			(BinOpKind::GreatEqual, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::I32((*lval >= *rval) as i32)
			}
			(BinOpKind::Less, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::I32((*lval < *rval) as i32)
			}
			(BinOpKind::Great, IntegerKind::U32(lval), IntegerKind::U32(rval)) => {
				IntegerKind::I32((*lval > *rval) as i32)
			}
			_ => {
				return Expr::Binary(ExprBinary {
					op: self.clone(),
					left: Box::new(Expr::Const(Constant {
						kind: Integer(lhs.0.clone()),
						span: lhs.1,
					})),
					right: Box::new(Expr::Const(Constant {
						kind: Integer(rhs.0.clone()),
						span: rhs.1,
					})),
				});
			}
		};
		// default span to left-most
		Expr::Const(Constant {
			kind: Integer(int_const),
			span: lhs.1,
		})
	}
	fn constant_fold_float(
		&self,
		lhs: (&FloatingKind, diag::Span),
		rhs: (&FloatingKind, diag::Span),
	) -> Expr {
		use ConstantKind::*;
		// TODO
		return Expr::Binary(ExprBinary {
			op: self.clone(),
			left: Box::new(Expr::Const(Constant {
				kind: Floating(lhs.0.clone()),
				span: lhs.1,
			})),
			right: Box::new(Expr::Const(Constant {
				kind: Floating(rhs.0.clone()),
				span: rhs.1,
			})),
		});
	}
}

/// (6.5.3) unary-operator
#[derive(Debug, Clone)]
pub enum PrefixKind {
	/// `&`
	Amp,
	/// `*`
	Star,
	/// `+`
	Plus,
	/// `-`
	Minus,
	/// `~`
	Comp,
	/// `!`
	Neg,
	/// `++`
	Inc,
	/// `--`
	Dec,
	/// `sizeof`
	Sizeof,
}

#[derive(Debug, Clone)]
pub struct Prefix {
	pub span: diag::Span,
	pub kind: PrefixKind,
}

impl ToSpan for Prefix {
	fn to_span(&self) -> diag::Span {
		self.span.clone()
	}
}

impl Prefix {
	fn reduce_int(&self, rhs: &IntegerKind, span: diag::Span) -> Expr {
		use ConstantKind::*;
		let int_const = match (&self.kind, rhs) {
			(PrefixKind::Plus, rval) => rval.clone(),
			(PrefixKind::Minus, IntegerKind::I32(rval)) => IntegerKind::I32(-(*rval)),
			(PrefixKind::Minus, IntegerKind::I64(rval)) => IntegerKind::I64(-(*rval)),
			(PrefixKind::Minus, IntegerKind::I128(rval)) => IntegerKind::I128(-(*rval)),
			(PrefixKind::Neg, IntegerKind::I32(rval)) => IntegerKind::I32((*rval == 0) as i32),
			(PrefixKind::Neg, IntegerKind::U32(rval)) => IntegerKind::I32((*rval == 0) as i32),
			(PrefixKind::Neg, IntegerKind::I64(rval)) => IntegerKind::I32((*rval == 0) as i32),
			(PrefixKind::Neg, IntegerKind::U64(rval)) => IntegerKind::I32((*rval == 0) as i32),
			(PrefixKind::Neg, IntegerKind::I128(rval)) => IntegerKind::I32((*rval == 0) as i32),
			(PrefixKind::Neg, IntegerKind::U128(rval)) => IntegerKind::I32((*rval == 0) as i32),
			_ => {
				return Expr::UnaryPrefix(UnaryPrefix {
					op: self.clone(),
					expr: Box::new(Expr::Const(Constant {
						kind: Integer(rhs.clone()),
						span,
					})),
				});
			}
		};
		Expr::Const(Constant {
			kind: Integer(int_const),
			span,
		})
	}
}

/// (6.5.2) postfix-expression
#[derive(Debug, Clone)]
pub enum PostfixKind {
	Array(Box<Expr>),
	/// (6.5.2) argument-expression-list
	ArgExprList(Vec<Expr>),
	Dot(Identifier),
	Arrow(Identifier),
	Inc,
	Dec,
}

/// (6.5.2) postfix-expression
#[derive(Debug, Clone)]
pub struct Postfix {
	pub span: diag::Span,
	pub kind: PostfixKind,
}

impl ToSpan for Postfix {
	fn to_span(&self) -> diag::Span {
		self.span.clone()
	}
}
