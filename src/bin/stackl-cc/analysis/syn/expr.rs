use std::f32;

use super::decl;
use super::Identifier;
use crate::analysis::tok::{self, FloatingConstant, IntegerConstant};
use crate::diagnostics as diag;
use crate::diagnostics::ToSpan;

pub enum ConversionError {
	OutOfRange,
	Expr(Expr),
}

/// (6.5.17) expression
#[derive(Debug, Clone)]
pub enum Expr {
	// Paren variant is required for the AssignIf warning to work.
	Paren(Box<Expr>),
	Ident(Identifier),
	Const(tok::Const),
	StrLit(tok::StrLit),
	UnaryPrefix(UnaryPrefix),
	UnaryPostfix(UnaryPostfix),
	Binary(ExprBinary),
	Ternary(ExprTernary),
	CompoundLiteral(decl::TypeName, decl::InitializerList),
	Sizeof(decl::TypeName),
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
		use tok::Const::{Floating, Integer};
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
	pub fn with_ternary(cond_expr: Self, then_expr: Self, else_expr: Self) -> Self {
		Self::Ternary(ExprTernary {
			expr_cond: Box::new(cond_expr),
			expr_then: Box::new(then_expr),
			expr_else: Box::new(else_expr),
		})
	}

	fn constant_fold(&self, contract_int: bool, contract_float: bool) -> Expr {
		use tok::Const::{Floating, Integer};
		match self {
			Self::UnaryPrefix(unary) => {
				let op = &unary.op;
				let expr = unary.expr.constant_fold(contract_int, contract_float);
				match &expr {
					Expr::Const(Integer(rhs_int)) => op.reduce_int(rhs_int),
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
					Expr::Const(Integer(rhs_int)) => {
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
					(true, _, Expr::Const(Integer(lhs_int)), Expr::Const(Integer(rhs_int))) => {
						op.constant_fold_int(lhs_int, rhs_int)
					}
					(true, _, Expr::Paren(expr), Expr::Const(Integer(rhs_int))) => {
						if let Expr::Const(Integer(lhs_int)) = expr.as_ref() {
							op.constant_fold_int(lhs_int, rhs_int)
						} else {
							self.clone()
						}
					}
					(true, _, Expr::Const(Integer(lhs_int)), Expr::Paren(expr)) => {
						if let Expr::Const(Integer(rhs_int)) = expr.as_ref() {
							op.constant_fold_int(lhs_int, rhs_int)
						} else {
							self.clone()
						}
					}
					(_, _, Expr::Paren(lhs_expr), Expr::Paren(rhs_expr)) => {
						if let (Expr::Const(Integer(lhs_int)), Expr::Const(Integer(rhs_int))) =
							(lhs_expr.as_ref(), rhs_expr.as_ref())
						{
							op.constant_fold_int(lhs_int, rhs_int)
						} else {
							self.clone()
						}
					}
					(
						_,
						true,
						Expr::Const(Floating(lhs_float)),
						Expr::Const(Floating(rhs_float)),
					) => op.constant_fold_float(lhs_float, rhs_float),
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
		const U64_CAP: u64 = u32::MAX as u64;
		const I64_CAP: i64 = u32::MAX as i64;
		const U128_CAP: u128 = u32::MAX as u128;
		const I128_CAP: i128 = u32::MAX as i128;
		*self = self.constant_fold(true, false);
		match self {
			Self::Const(tok::Const::Integer(int_const)) => match int_const {
				IntegerConstant::U32(val) => Ok(*val),
				IntegerConstant::I32(val) => match val {
					0.. => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::U64(val) => match val {
					0..=U64_CAP => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::I64(val) => match val {
					0..=I64_CAP => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::U128(val) => match val {
					0..=U128_CAP => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::I128(val) => match val {
					0..=I128_CAP => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
			},
			expr => Err(ConversionError::Expr(expr.clone())),
		}
	}
	pub fn to_i32(&mut self) -> Result<i32, ConversionError> {
		const U32_CAP: u32 = i32::MAX as u32;
		const U64_CAP: u64 = i32::MAX as u64;
		const I64_CAP: i64 = i32::MAX as i64;
		const U128_CAP: u128 = i32::MAX as u128;
		const I128_CAP: i128 = i32::MAX as i128;
		*self = self.constant_fold(true, false);
		match self {
			Self::Const(tok::Const::Integer(int_const)) => match int_const {
				IntegerConstant::U32(val) => match val {
					0..=U32_CAP => Ok(*val as i32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::I32(val) => Ok(*val),
				IntegerConstant::U64(val) => match val {
					0..=U64_CAP => Ok(*val as i32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::I64(val) => match val {
					0..=I64_CAP => Ok(*val as i32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::U128(val) => match val {
					0..=U128_CAP => Ok(*val as i32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::I128(val) => match val {
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

/// (6.5.3) unary-expression
#[derive(Debug, Clone)]
pub struct UnaryPostfix {
	pub op: Postfix,
	pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprBinary {
	pub op: BinOp,
	pub left: Box<Expr>,
	pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprTernary {
	pub expr_cond: Box<Expr>,
	pub expr_then: Box<Expr>,
	pub expr_else: Box<Expr>,
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
	fn constant_fold_int(&self, lhs: &IntegerConstant, rhs: &IntegerConstant) -> Expr {
		let int_const = match (self.kind, lhs, rhs) {
			(BinOpKind::Mul, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_mul(*rval))
			}
			(BinOpKind::Mul, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(lval.wrapping_mul(*rval))
			}
			(BinOpKind::Div, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				if *rval != 0 {
					IntegerConstant::U32(lval.wrapping_sub(*rval))
				} else {
					// Undefined Behavior
					IntegerConstant::U32(u32::MAX)
				}
			}
			(BinOpKind::Div, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				if *rval != 0 {
					IntegerConstant::I32(lval.wrapping_sub(*rval))
				} else {
					// Undefined Behavior
					IntegerConstant::I32(i32::MAX)
				}
			}
			(BinOpKind::Rem, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_rem(*rval))
			}
			(BinOpKind::Rem, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(lval.wrapping_rem(*rval))
			}
			(BinOpKind::Sub, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_sub(*rval))
			}
			(BinOpKind::Sub, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(lval.wrapping_sub(*rval))
			}
			(BinOpKind::Add, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_add(*rval))
			}
			(BinOpKind::Add, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(lval.wrapping_add(*rval))
			}
			(BinOpKind::NotEqual, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval != *rval) as i32)
			}
			(BinOpKind::NotEqual, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32((*lval != *rval) as i32)
			}
			(BinOpKind::Equal, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval == *rval) as i32)
			}
			(BinOpKind::Equal, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32((*lval == *rval) as i32)
			}
			(BinOpKind::And, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(*lval & *rval)
			}
			(BinOpKind::And, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(*lval & *rval)
			}
			(BinOpKind::XOr, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(*lval ^ *rval)
			}
			(BinOpKind::XOr, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(*lval ^ *rval)
			}
			(BinOpKind::Or, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(*lval | *rval)
			}
			(BinOpKind::Or, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(*lval | *rval)
			}
			(BinOpKind::LogicalAnd, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32(((*lval != 0) && (*rval != 0)) as i32)
			}
			(BinOpKind::LogicalAnd, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(((*lval != 0) && (*rval != 0)) as i32)
			}
			(BinOpKind::LogicalOr, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32(((*lval != 0) || (*rval != 0)) as i32)
			}
			(BinOpKind::LogicalOr, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(((*lval != 0) || (*rval != 0)) as i32)
			}
			(BinOpKind::Shl, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_shl(*rval))
			}
			(BinOpKind::Shl, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				match (*rval).try_into() {
					Ok(rval) => IntegerConstant::I32(lval.wrapping_shl(rval)),
					Err(_) => IntegerConstant::I32(lval.wrapping_shr((-rval) as u32)),
				}
			}
			(BinOpKind::Shl, IntegerConstant::I32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32(lval.wrapping_shl(*rval))
			}
			(BinOpKind::Shr, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_shr(*rval))
			}
			(BinOpKind::LessEqual, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval <= *rval) as i32)
			}
			(BinOpKind::GreatEqual, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval >= *rval) as i32)
			}
			(BinOpKind::Less, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval < *rval) as i32)
			}
			(BinOpKind::Great, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval > *rval) as i32)
			}
			_ => {
				return Expr::Binary(ExprBinary {
					op: self.clone(),
					left: Box::new(Expr::Const(tok::Const::Integer(lhs.clone()))),
					right: Box::new(Expr::Const(tok::Const::Integer(rhs.clone()))),
				})
			}
		};
		Expr::Const(tok::Const::Integer(int_const))
	}
	fn constant_fold_float(&self, lhs: &FloatingConstant, rhs: &FloatingConstant) -> Expr {
		// TODO
		return Expr::Binary(ExprBinary {
			op: self.clone(),
			left: Box::new(Expr::Const(tok::Const::Floating(lhs.clone()))),
			right: Box::new(Expr::Const(tok::Const::Floating(rhs.clone()))),
		});
	}
}

/// (6.5.3) unary-operator
#[derive(Debug, Clone)]
pub enum Prefix {
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
	/// ( type-name )
	Cast(decl::TypeName),
	/// ++
	Inc,
	/// --
	Dec,
	/// sizeof
	Sizeof,
}

impl Prefix {
	fn reduce_int(&self, rhs: &IntegerConstant) -> Expr {
		let int_const = match (self, rhs) {
			(Prefix::Plus, rval) => rval.clone(),
			(Prefix::Minus, IntegerConstant::I32(rval)) => IntegerConstant::I32(-(*rval)),
			(Prefix::Minus, IntegerConstant::I64(rval)) => IntegerConstant::I64(-(*rval)),
			(Prefix::Minus, IntegerConstant::I128(rval)) => IntegerConstant::I128(-(*rval)),
			(Prefix::Neg, IntegerConstant::I32(rval)) => IntegerConstant::I32((*rval == 0) as i32),
			(Prefix::Neg, IntegerConstant::U32(rval)) => IntegerConstant::I32((*rval == 0) as i32),
			(Prefix::Neg, IntegerConstant::I64(rval)) => IntegerConstant::I32((*rval == 0) as i32),
			(Prefix::Neg, IntegerConstant::U64(rval)) => IntegerConstant::I32((*rval == 0) as i32),
			(Prefix::Neg, IntegerConstant::I128(rval)) => IntegerConstant::I32((*rval == 0) as i32),
			(Prefix::Neg, IntegerConstant::U128(rval)) => IntegerConstant::I32((*rval == 0) as i32),
			_ => {
				return Expr::UnaryPrefix(UnaryPrefix {
					op: self.clone(),
					expr: Box::new(Expr::Const(tok::Const::Integer(rhs.clone()))),
				})
			}
		};
		Expr::Const(tok::Const::Integer(int_const))
	}
}

/// (6.5.2) postfix-expression
#[derive(Debug, Clone)]
pub enum Postfix {
	Array(Box<Expr>),
	/// (6.5.2) argument-expression-list
	ArgExprList(Vec<Expr>),
	Dot(Identifier),
	Arrow(Identifier),
	Inc,
	Dec,
}
