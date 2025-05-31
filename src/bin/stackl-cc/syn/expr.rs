use super::decl;
use crate::tok;

/// (6.5.16) assignment-expression
pub struct AssignmentExpr {
	assignment_expr: Vec<(UnaryExpr, AssignmentOperator)>,
	conditional_expr: ConditionalExpr,
}

/// (6.5.3) unary-expression
pub enum UnaryExpr {
	PostfixExpr(Vec<PostfixExpr>),
	Increment(Box<UnaryExpr>),
	Decrement(Box<UnaryExpr>),
	/// unary-operator cast-expression
	UnaryOperator(UnaryOperator, CastExpr),
	/// sizeof unary-expression
	SizeofUnary(Box<UnaryExpr>),
	/// sizeof ( type-name )
	Sizeof(decl::TypeName),
}

/// (6.5.3) unary-operator
pub enum UnaryOperator {
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
}

/// (6.5.2) postfix-expression
pub enum PostfixExpr {
	PrimaryExpr(PrimaryExpr),
	Array(Expr),
	/// (6.5.2) argument-expression-list
	ArgumentExprList(Vec<AssignmentExpr>),
	Dot(tok::Ident),
	Arrow(tok::Ident),
	Increment,
	Decrement,
	TypeNameInitializerList(decl::TypeName, decl::InitializerList),
}

/// (6.5.1) primary-expression
pub enum PrimaryExpr {
	Identifier(tok::Ident),
	Constant(tok::Const),
	StrLit(tok::StrLit),
	Expr(Expr),
}

/// (6.5.17) expression
pub struct Expr(Vec<AssignmentExpr>);

/// (6.5.16) assignment-operator
pub enum AssignmentOperator {
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
}

/// (6.5.15) conditional-expression
pub enum ConditionalExpr {
	LogicalORExpression(LogicalOrExpr),
	Ternary(LogicalOrExpr, Expr, Box<ConditionalExpr>),
}

/// (6.5.14) logical-OR-expression
pub enum LogicalOrExpr {
	LogicalAndExpr(LogicalAndExpr),
	LogicalOrExpr(Box<LogicalOrExpr>, LogicalAndExpr),
}

/// (6.5.13) logical-AND-expression
pub enum LogicalAndExpr {
	InclusiveOrExpr(InclusiveOrExpr),
	LogicalAndExpr(Box<LogicalAndExpr>, InclusiveOrExpr),
}

/// (6.5.12) inclusive-OR-expression
pub enum InclusiveOrExpr {
	ExclusiveOrExpr(ExclusiveOrExpr),
	InclusiveOrExpr(Box<InclusiveOrExpr>, ExclusiveOrExpr),
}

/// (6.5.11) exclusive-OR-expression
pub enum ExclusiveOrExpr {
	AndExpr(AndExpr),
	ExclusiveOrExpr(Box<ExclusiveOrExpr>, AndExpr),
}

/// (6.5.10) AND-expression
pub enum AndExpr {
	EqualityExpr(EqualityExpr),
	AndExpr(Box<AndExpr>, EqualityExpr),
}

/// (6.5.9) equality-expression
pub enum EqualityExpr {
	RelationalExpr(RelationalExpr),
	Equal(Box<EqualityExpr>, RelationalExpr),
	NotEqual(Box<EqualityExpr>, RelationalExpr),
}

/// (6.5.8) relational-expression
pub enum RelationalExpr {
	ShiftExpr(ShiftExpr),
	Less(Box<RelationalExpr>, ShiftExpr),
	Great(Box<RelationalExpr>, ShiftExpr),
	LessEqual(Box<RelationalExpr>, ShiftExpr),
	GreatEqual(Box<RelationalExpr>, ShiftExpr),
}

/// (6.5.7) shift-expression
pub enum ShiftExpr {
	AdditiveExpr(AdditiveExpr),
	LeftShift(Box<ShiftExpr>, AdditiveExpr),
	RightShift(Box<ShiftExpr>, AdditiveExpr),
}

/// (6.5.6) additive-expression
pub enum AdditiveExpr {
	MultiplicativeExpr(MultiplicativeExpr),
	Add(Box<AdditiveExpr>, MultiplicativeExpr),
	Sub(Box<AdditiveExpr>, MultiplicativeExpr),
}

/// (6.5.5) multiplicative-expression
pub enum MultiplicativeExpr {
	CastExpr(CastExpr),
	Mul(Box<MultiplicativeExpr>, CastExpr),
	Div(Box<MultiplicativeExpr>, CastExpr),
	Mod(Box<MultiplicativeExpr>, CastExpr),
}

/// (6.5.4) cast-expression
pub enum CastExpr {
	UnaryExpr(Box<UnaryExpr>),
	TypeName(decl::TypeName, Box<CastExpr>),
}

/// (6.6) constant-expression
pub struct ConstantExpr(ConditionalExpr);
