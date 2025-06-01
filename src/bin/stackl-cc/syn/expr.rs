use super::decl;
use crate::tok;

/// (6.5.17) expression
#[derive(Debug)]
pub enum Expr {
	Ident(tok::Ident),
	Const(tok::Const),
	StrLit(tok::StrLit),
	Paren(Box<Expr>),
	Unary(ExprUnary),
	Binary(ExprBinary),
	Ternary(ExprTernary),
}

/// (6.5.3) unary-expression
#[derive(Debug)]
pub struct ExprUnary {
	pub op: UnOp,
	pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct ExprBinary {
	pub left: Box<Expr>,
	pub op: BinOp,
	pub right: Box<Expr>,
}

#[derive(Debug)]
pub struct ExprTernary {
	pub cond: Box<Expr>,
	pub then_branch: Box<Expr>,
	pub else_branch: Box<Expr>,
}

#[derive(Debug)]
pub enum BinOp {
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
}

/// (6.5.3) unary-operator
#[derive(Debug)]
pub enum UnOp {
	Prefix(UnOpPrefix),
	Postfix(UnOpPostfix),
}

#[derive(Debug)]
pub enum UnOpPrefix {
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
}

/// (6.5.2) postfix-expression
#[derive(Debug)]
pub enum UnOpPostfix {
	Array(Box<Expr>),
	/// (6.5.2) argument-expression-list
	ArgExprList(Vec<Expr>),
	Dot(tok::Ident),
	Arrow(tok::Ident),
	Increment,
	Decrement,
	TypeNameInitializerList(decl::TypeName, decl::InitializerList),
}
