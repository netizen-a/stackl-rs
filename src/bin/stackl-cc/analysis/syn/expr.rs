use std::f32;

use super::decl;
use crate::analysis::tok::{self, FloatingConstant, IntegerConstant};

pub enum ConversionError {
	OutOfRange,
	Expr(Expr),
}

/// (6.5.17) expression
#[derive(Debug, Clone)]
pub enum Expr {
	Ident(tok::Ident),
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
	pub fn with_binary(op: BinOp, left: Self, right: Self) -> Self {
		Self::Binary(ExprBinary {
			op,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	#[inline]
	pub fn with_ternary(cond_expr: Self, then_expr: Self, else_expr: Self) -> Self {
		Self::Ternary(ExprTernary {
			cond_expr: Box::new(cond_expr),
			then_expr: Box::new(then_expr),
			else_expr: Box::new(else_expr),
		})
	}

	fn reduce(&self) -> Expr {
		use tok::Const::{Floating, Integer};
		match self {
			Self::UnaryPrefix(unary) => {
				let op = &unary.op;
				let expr = unary.expr.reduce();
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
				let expr = unary.expr.reduce();
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
				let left = binary.left.reduce();
				let right = binary.right.reduce();
				let op = binary.op;
				match (&left, &right) {
					(Expr::Const(Integer(lhs_int)), Expr::Const(Integer(rhs_int))) => {
						op.reduce_int(lhs_int, rhs_int)
					}
					(Expr::Const(Floating(lhs_float)), Expr::Const(Floating(rhs_float))) => {
						op.reduce_float(lhs_float, rhs_float)
					}
					_ => Self::Binary(ExprBinary {
						op,
						left: Box::new(left),
						right: Box::new(right),
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
	pub fn to_u32(&mut self) -> Result<u32, ConversionError> {
		const U64_CAP: u64 = u32::MAX as u64;
		const I64_CAP: i64 = u32::MAX as i64;
		const U128_CAP: u128 = u32::MAX as u128;
		const I128_CAP: i128 = u32::MAX as i128;
		*self = self.reduce();
		match self {
			Self::Const(tok::Const::Integer(int_const)) => match int_const {
				IntegerConstant::U32(val) => Ok(*val),
				IntegerConstant::I32(val) => match val {
					0.. => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::U64(val) => match val {
					0..U64_CAP => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::I64(val) => match val {
					0..I64_CAP => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::U128(val) => match val {
					0..U128_CAP => Ok(*val as u32),
					_ => Err(ConversionError::OutOfRange),
				},
				IntegerConstant::I128(val) => match val {
					0..I128_CAP => Ok(*val as u32),
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
	pub cond_expr: Box<Expr>,
	pub then_expr: Box<Expr>,
	pub else_expr: Box<Expr>,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
	Mul,
	Div,
	Mod,
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
	ModAssign,
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

impl BinOp {
	fn reduce_int(&self, lhs: &IntegerConstant, rhs: &IntegerConstant) -> Expr {
		let int_const = match (self, lhs, rhs) {
			(BinOp::Mul, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_mul(*rval))
			}
			(BinOp::Mul, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(lval.wrapping_mul(*rval))
			}
			(BinOp::Div, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				if *rval != 0 {
					IntegerConstant::U32(lval.wrapping_sub(*rval))
				} else {
					// Undefined Behavior
					IntegerConstant::U32(u32::MAX)
				}
			}
			(BinOp::Div, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				if *rval != 0 {
					IntegerConstant::I32(lval.wrapping_sub(*rval))
				} else {
					// Undefined Behavior
					IntegerConstant::I32(i32::MAX)
				}
			}
			(BinOp::Mod, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_rem(*rval))
			}
			(BinOp::Mod, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(lval.wrapping_rem(*rval))
			}
			(BinOp::Sub, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_sub(*rval))
			}
			(BinOp::Sub, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(lval.wrapping_sub(*rval))
			}
			(BinOp::Add, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_add(*rval))
			}
			(BinOp::Add, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(lval.wrapping_add(*rval))
			}
			(BinOp::NotEqual, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval != *rval) as i32)
			}
			(BinOp::NotEqual, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32((*lval != *rval) as i32)
			}
			(BinOp::Equal, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval == *rval) as i32)
			}
			(BinOp::Equal, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32((*lval == *rval) as i32)
			}
			(BinOp::And, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(*lval & *rval)
			}
			(BinOp::And, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(*lval & *rval)
			}
			(BinOp::XOr, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(*lval ^ *rval)
			}
			(BinOp::XOr, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(*lval ^ *rval)
			}
			(BinOp::Or, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(*lval | *rval)
			}
			(BinOp::Or, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(*lval | *rval)
			}
			(BinOp::LogicalAnd, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32(((*lval != 0) && (*rval != 0)) as i32)
			}
			(BinOp::LogicalAnd, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(((*lval != 0) && (*rval != 0)) as i32)
			}
			(BinOp::LogicalOr, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32(((*lval != 0) || (*rval != 0)) as i32)
			}
			(BinOp::LogicalOr, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				IntegerConstant::I32(((*lval != 0) || (*rval != 0)) as i32)
			}
			(BinOp::Shl, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_shl(*rval))
			}
			(BinOp::Shl, IntegerConstant::I32(lval), IntegerConstant::I32(rval)) => {
				match (*rval).try_into() {
					Ok(rval) => IntegerConstant::I32(lval.wrapping_shl(rval)),
					Err(_) => IntegerConstant::I32(lval.wrapping_shr((-rval) as u32)),
				}
			}
			(BinOp::Shl, IntegerConstant::I32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32(lval.wrapping_shl(*rval))
			}
			(BinOp::Shr, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::U32(lval.wrapping_shr(*rval))
			}
			(BinOp::LessEqual, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval <= *rval) as i32)
			}
			(BinOp::GreatEqual, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval >= *rval) as i32)
			}
			(BinOp::Less, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval < *rval) as i32)
			}
			(BinOp::Great, IntegerConstant::U32(lval), IntegerConstant::U32(rval)) => {
				IntegerConstant::I32((*lval > *rval) as i32)
			}
			_ => {
				return Expr::Binary(ExprBinary {
					op: *self,
					left: Box::new(Expr::Const(tok::Const::Integer(lhs.clone()))),
					right: Box::new(Expr::Const(tok::Const::Integer(rhs.clone()))),
				})
			}
		};
		Expr::Const(tok::Const::Integer(int_const))
	}
	fn reduce_float(&self, lhs: &FloatingConstant, rhs: &FloatingConstant) -> Expr {
		// TODO
		return Expr::Binary(ExprBinary {
			op: *self,
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
			(Prefix::Neg, IntegerConstant::I32(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
			(Prefix::Neg, IntegerConstant::U32(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
			(Prefix::Neg, IntegerConstant::I64(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
			(Prefix::Neg, IntegerConstant::U64(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
			(Prefix::Neg, IntegerConstant::I128(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
			(Prefix::Neg, IntegerConstant::U128(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
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
	Dot(tok::Ident),
	Arrow(tok::Ident),
	Inc,
	Dec,
}
