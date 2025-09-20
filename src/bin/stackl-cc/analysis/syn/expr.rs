use std::f32;

use super::decl;
use crate::analysis::tok::{self, FloatingConstant, IntegerConstant};

/// (6.5.17) expression
#[derive(Debug, Clone)]
pub enum Expr {
	Ident(tok::Ident),
	Const(tok::Const),
	StrLit(tok::StrLit),
	Unary(ExprUnary),
	Binary(ExprBinary),
	Ternary(ExprTernary),
	CompoundLiteral(decl::TypeName, decl::InitializerList),
	Sizeof(decl::TypeName),
}

impl Expr {
	pub fn with_binary(op: BinOp, left: Expr, right: Expr) -> Self {
		use tok::Const::{Floating, Integer};
		match (&left, &right) {
			(Expr::Const(Integer(lhs_int)), Expr::Const(Integer(rhs_int))) => {
				op.reduce_int(lhs_int, rhs_int)
			}
			(Expr::Const(Floating(lhs_int)), Expr::Const(Floating(rhs_int))) => {
				op.reduce_float(lhs_int, rhs_int)
			}
			_ => Self::Binary(ExprBinary {
				op,
				left: Box::new(left),
				right: Box::new(right),
			}),
		}
	}
	pub fn with_unary(op: UnOp, expr: Expr) -> Self {
		use tok::Const::{Floating, Integer};
		match &expr {
			Expr::Const(Integer(int_const)) => op.reduce_int(int_const),
			Expr::Const(Floating(float_const)) => op.reduce_float(float_const),
			_ => Self::Unary(ExprUnary {
				op,
				expr: Box::new(expr),
			}),
		}
	}
}

/// (6.5.3) unary-expression
#[derive(Debug, Clone)]
pub struct ExprUnary {
	pub op: UnOp,
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
	pub cond: Box<Expr>,
	pub then_branch: Box<Expr>,
	pub else_branch: Box<Expr>,
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
pub enum UnOp {
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
	Postfix(Postfix),
}

impl UnOp {
	fn reduce_int(&self, rhs: &IntegerConstant) -> Expr {
		let int_const = match (self, rhs) {
			(UnOp::Plus, rval) => rval.clone(),
			(UnOp::Minus, IntegerConstant::I32(rval)) => IntegerConstant::I32(-(*rval)),
			(UnOp::Minus, IntegerConstant::I64(rval)) => IntegerConstant::I64(-(*rval)),
			(UnOp::Minus, IntegerConstant::I128(rval)) => IntegerConstant::I128(-(*rval)),
			(UnOp::Neg, IntegerConstant::I32(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
			(UnOp::Neg, IntegerConstant::U32(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
			(UnOp::Neg, IntegerConstant::I64(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
			(UnOp::Neg, IntegerConstant::U64(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
			(UnOp::Neg, IntegerConstant::I128(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
			(UnOp::Neg, IntegerConstant::U128(rval)) => match *rval {
				0 => IntegerConstant::I32(1),
				_ => IntegerConstant::I32(0),
			},
			_ => {
				return Expr::Unary(ExprUnary {
					op: self.clone(),
					expr: Box::new(Expr::Const(tok::Const::Integer(rhs.clone()))),
				})
			}
		};
		Expr::Const(tok::Const::Integer(int_const))
	}
	fn reduce_float(&self, rhs: &FloatingConstant) -> Expr {
		// TODO
		return Expr::Unary(ExprUnary {
			op: self.clone(),
			expr: Box::new(Expr::Const(tok::Const::Floating(rhs.clone()))),
		});
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
