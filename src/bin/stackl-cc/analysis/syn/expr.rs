use super::decl;
use crate::analysis::tok;

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
		use tok::Const::Integer;
		use tok::IntegerConstant;

		let (Expr::Const(left_const), Expr::Const(right_const)) = (&left, &right) else {
			return Self::Binary(ExprBinary {
				op,
				left: Box::new(left),
				right: Box::new(right),
			});
		};

		match (&left_const, &right_const) {
			(Integer(IntegerConstant::U32(left_int)), Integer(IntegerConstant::U32(right_int))) => {
				let result: u32 = match op {
					BinOp::Add => left_int.wrapping_add(*right_int),
					BinOp::Sub => left_int.wrapping_sub(*right_int),
					BinOp::Mul => left_int.wrapping_mul(*right_int),
					BinOp::Mod => left_int.wrapping_rem(*right_int),
					BinOp::Div => if *right_int == 0 {
						u32::MAX
					} else {
						left_int.wrapping_div(*right_int)
					},
					BinOp::Shl => left_int.wrapping_shl(*right_int),
					BinOp::Shr => left_int.wrapping_shr(*right_int),
					BinOp::And => *left_int & *right_int,
					BinOp::Or => *left_int | *right_int,
					BinOp::XOr => *left_int ^ *right_int,
					_ => {
						return Self::Binary(ExprBinary {
							op,
							left: Box::new(left),
							right: Box::new(right),
						})
					}
				};
				Expr::Const(Integer(IntegerConstant::U32(result)))
			}
			(Integer(IntegerConstant::U32(left_int)), Integer(IntegerConstant::U64(right_int))) => {
				let result: u64 = match op {
					BinOp::Add => (*left_int as u64).wrapping_add(*right_int),
					BinOp::Sub => (*left_int as u64).wrapping_sub(*right_int),
					BinOp::Mul => (*left_int as u64).wrapping_mul(*right_int),
					BinOp::Mod => (*left_int as u64).wrapping_rem(*right_int),
					BinOp::Div => if *right_int == 0 {
						u64::MAX
					} else {
						(*left_int as u64).wrapping_div(*right_int)
					},
					BinOp::And => (*left_int as u64) & *right_int,
					BinOp::Or => (*left_int as u64) | *right_int,
					_ => {
						return Self::Binary(ExprBinary {
							op,
							left: Box::new(left),
							right: Box::new(right),
						})
					}
				};
				Expr::Const(Integer(IntegerConstant::U64(result)))
			}
			(Integer(IntegerConstant::U32(left_int)), Integer(IntegerConstant::U128(right_int))) => {
				let result: u128 = match op {
					BinOp::Add => (*left_int as u128).wrapping_add(*right_int),
					BinOp::Sub => (*left_int as u128).wrapping_sub(*right_int),
					BinOp::Mul => (*left_int as u128).wrapping_mul(*right_int),
					BinOp::Mod => (*left_int as u128).wrapping_rem(*right_int),
					BinOp::Div => if *right_int == 0 {
						u128::MAX
					} else {
						(*left_int as u128).wrapping_div(*right_int)
					},
					_ => {
						return Self::Binary(ExprBinary {
							op,
							left: Box::new(left),
							right: Box::new(right),
						})
					}
				};
				Expr::Const(Integer(IntegerConstant::U128(result)))
			}
			(Integer(IntegerConstant::I32(left_int)), Integer(IntegerConstant::I32(right_int))) => {
				let result: i32 = match op {
					BinOp::Add => left_int.wrapping_add(*right_int),
					BinOp::Sub => left_int.wrapping_sub(*right_int),
					BinOp::Mul => left_int.wrapping_mul(*right_int),
					BinOp::Mod => left_int.wrapping_rem(*right_int),
					BinOp::Div => if *right_int == 0 {
						i32::MAX
					} else {
						left_int.wrapping_div(*right_int)
					},
					BinOp::And => *left_int & *right_int,
					BinOp::Or => *left_int | *right_int,
					BinOp::XOr => *left_int ^ *right_int,
					_ => {
						return Self::Binary(ExprBinary {
							op,
							left: Box::new(left),
							right: Box::new(right),
						})
					}
				};
				Expr::Const(Integer(IntegerConstant::I32(result)))
			}
			(Integer(IntegerConstant::U64(left_int)), Integer(IntegerConstant::U32(right_int))) => {
				let result: u64 = match op {
					BinOp::Add => left_int.wrapping_add(*right_int as u64),
					BinOp::Sub => left_int.wrapping_sub(*right_int as u64),
					BinOp::Mul => left_int.wrapping_mul(*right_int as u64),
					BinOp::Mod => left_int.wrapping_rem(*right_int as u64),
					BinOp::Div => if *right_int == 0 {
						u64::MAX
					} else {
						left_int.wrapping_div(*right_int as u64)
					},
					BinOp::Shl => left_int.wrapping_shl(*right_int),
					BinOp::Shr => left_int.wrapping_shr(*right_int),
					_ => {
						return Self::Binary(ExprBinary {
							op,
							left: Box::new(left),
							right: Box::new(right),
						})
					}
				};
				Expr::Const(Integer(IntegerConstant::U64(result)))
			}
			(Integer(IntegerConstant::U64(left_int)), Integer(IntegerConstant::U64(right_int))) => {
				let result: u64 = match op {
					BinOp::Add => left_int.wrapping_add(*right_int),
					BinOp::Sub => left_int.wrapping_sub(*right_int),
					BinOp::Mul => left_int.wrapping_mul(*right_int),
					BinOp::Mod => left_int.wrapping_rem(*right_int),
					BinOp::Div => if *right_int == 0 {
						u64::MAX
					} else {
						left_int.wrapping_div(*right_int)
					},
					BinOp::And => *left_int & *right_int,
					BinOp::Or => *left_int | *right_int,
					BinOp::XOr => *left_int ^ *right_int,
					_ => {
						return Self::Binary(ExprBinary {
							op,
							left: Box::new(left),
							right: Box::new(right),
						})
					}
				};
				Expr::Const(Integer(IntegerConstant::U64(result)))
			}
			(Integer(IntegerConstant::I64(left_int)), Integer(IntegerConstant::I64(right_int))) => {
				let result: i64 = match op {
					BinOp::Add => left_int.wrapping_add(*right_int),
					BinOp::Sub => left_int.wrapping_sub(*right_int),
					BinOp::Mul => left_int.wrapping_mul(*right_int),
					BinOp::Mod => left_int.wrapping_rem(*right_int),
					BinOp::Div => if *right_int == 0 {
						i64::MAX
					} else {
						left_int.wrapping_div(*right_int)
					},
					BinOp::And => *left_int & *right_int,
					BinOp::Or => *left_int | *right_int,
					BinOp::XOr => *left_int ^ *right_int,
					_ => {
						return Self::Binary(ExprBinary {
							op,
							left: Box::new(left),
							right: Box::new(right),
						})
					}
				};
				Expr::Const(Integer(IntegerConstant::I64(result)))
			}
			(
				Integer(IntegerConstant::U128(left_int)),
				Integer(IntegerConstant::U32(right_int)),
			) => {
				let result: u128 = match op {
					BinOp::Add => left_int.wrapping_add(*right_int as u128),
					BinOp::Sub => left_int.wrapping_sub(*right_int as u128),
					BinOp::Mul => left_int.wrapping_mul(*right_int as u128),
					BinOp::Mod => left_int.wrapping_rem(*right_int as u128),
					BinOp::Div => if *right_int == 0 {
						u128::MAX
					} else {
						left_int.wrapping_div(*right_int as u128)
					},
					BinOp::Shl => left_int.wrapping_shl(*right_int),
					BinOp::Shr => left_int.wrapping_shr(*right_int),
					_ => {
						return Self::Binary(ExprBinary {
							op,
							left: Box::new(left),
							right: Box::new(right),
						})
					}
				};
				Expr::Const(Integer(IntegerConstant::U128(result)))
			}
			(
				Integer(IntegerConstant::U128(left_int)),
				Integer(IntegerConstant::U128(right_int)),
			) => {
				let result: u128 = match op {
					BinOp::Add => left_int.wrapping_add(*right_int),
					BinOp::Sub => left_int.wrapping_sub(*right_int),
					BinOp::Mul => left_int.wrapping_mul(*right_int),
					BinOp::Mod => left_int.wrapping_rem(*right_int),
					BinOp::Div => if *right_int == 0 {
						u128::MAX
					} else {
						left_int.wrapping_div(*right_int)
					},
					BinOp::And => *left_int & *right_int,
					BinOp::Or => *left_int | *right_int,
					BinOp::XOr => *left_int ^ *right_int,
					_ => {
						return Self::Binary(ExprBinary {
							op,
							left: Box::new(left),
							right: Box::new(right),
						})
					}
				};
				Expr::Const(Integer(IntegerConstant::U128(result)))
			}
			(
				Integer(IntegerConstant::I128(left_int)),
				Integer(IntegerConstant::I128(right_int)),
			) => {
				let result: i128 = match op {
					BinOp::Add => left_int.wrapping_add(*right_int),
					BinOp::Sub => left_int.wrapping_sub(*right_int),
					BinOp::Mul => left_int.wrapping_mul(*right_int),
					BinOp::Mod => left_int.wrapping_rem(*right_int),
					BinOp::Div => if *right_int == 0 {
						i128::MAX
					} else {
						left_int.wrapping_div(*right_int)
					},
					BinOp::And => *left_int & *right_int,
					BinOp::Or => *left_int | *right_int,
					_ => {
						return Self::Binary(ExprBinary {
							op,
							left: Box::new(left),
							right: Box::new(right),
						})
					}
				};
				Expr::Const(Integer(IntegerConstant::I128(result)))
			}
			_ => Self::Binary(ExprBinary {
				op,
				left: Box::new(left),
				right: Box::new(right),
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
	ExclusiveOr,
	InclusiveOr,
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
	XOr,
	Or,
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
