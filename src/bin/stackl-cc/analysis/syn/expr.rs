use super::decl;
use crate::analysis::tok;

/// (6.5.17) expression
#[derive(Debug, Clone)]
pub enum Expr {
	Ident(tok::Ident),
	Const(tok::Const),
	StrLit(tok::StrLit),
	Paren(Box<Expr>),
	Unary(ExprUnary),
	Binary(ExprBinary),
	Ternary(ExprTernary),
	CompoundLiteral(decl::TypeName, decl::InitializerList),
	Sizeof(decl::TypeName),
}

/// (6.5.3) unary-expression
#[derive(Debug, Clone)]
pub struct ExprUnary {
	pub op: UnOp,
	pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprBinary {
	pub left: Box<Expr>,
	pub op: BinOp,
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
	LShift,
	RShift,
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
