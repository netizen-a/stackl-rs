use super::decl;
use crate::tok;

pub struct AssignmentExpression {
	assignment_expression: Vec<(UnaryExpression, AssignmentOperator)>,
	conditional_expression: ConditionalExpression,
}

pub enum UnaryExpression {
	PostfixExpression(Vec<PostfixExpression>),
	Increment(Box<UnaryExpression>),
	Decrement(Box<UnaryExpression>),
	/// unary-operator cast-expression
	UnaryOperator(UnaryOperator, CastExpression),
	/// sizeof unary-expression
	SizeofUnary(Box<UnaryExpression>),
	/// sizeof ( type-name )
	Sizeof(decl::TypeName),
}

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

pub enum PostfixExpression {
	PrimaryExpression(PrimaryExpression),
	Array(Expression),
	ArgumentExpressionList(Option<ArgumentExpressionList>),
	Dot(tok::Identifier),
	Arrow(tok::Identifier),
	Increment,
	Decrement,
	TypeNameInitializerList(TypeName, decl::InitializerList),
}

pub struct ArgumentExpressionList(Vec<AssignmentExpression>);

pub enum PrimaryExpression {
	Identifier(tok::Identifier),
	Constant(tok::Constant),
	StringLiteral(tok::StringLiteral),
	Expression(Expression),
}

pub struct Expression(Vec<AssignmentExpression>);

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

pub enum ConditionalExpression {
	LogicalORExpression(LogicalOrExpression),
	Ternary(LogicalOrExpression, Expression, Box<ConditionalExpression>),
}

/// (6.5.14) logical-OR-expression
pub enum LogicalOrExpression {
	LogicalAndExpression(LogicalAndExpression),
	LogicalOrExpression(Box<LogicalOrExpression>, LogicalAndExpression),
}

/// (6.5.13) logical-AND-expression
pub enum LogicalAndExpression {
	InclusiveOrExpression(InclusiveOrExpression),
	LogicalAndExpression(Box<LogicalAndExpression>, InclusiveOrExpression),
}

/// (6.5.12) inclusive-OR-expression
pub enum InclusiveOrExpression {
	ExclusiveOrExpression(ExclusiveOrExpression),
	InclusiveOrExpression(Box<InclusiveOrExpression>, ExclusiveOrExpression),
}

/// (6.5.11) exclusive-OR-expression
pub enum ExclusiveOrExpression {
	AndExpressionExpression(AndExpression),
	ExclusiveOrExpression(Box<ExclusiveOrExpression>, AndExpression),
}

/// (6.5.10) AND-expression
pub enum AndExpression {
	EqualityExpression(EqualityExpression),
	AndExpression(Box<AndExpression>, EqualityExpression),
}

pub enum EqualityExpression {
	RelationalExpression(RelationalExpression),
	Equal(Box<EqualityExpression>, RelationalExpression),
	NotEqual(Box<EqualityExpression>, RelationalExpression),
}

pub enum RelationalExpression {
	ShiftExpression(ShiftExpression),
	Less(Box<RelationalExpression>, ShiftExpression),
	Great(Box<RelationalExpression>, ShiftExpression),
	LessEqual(Box<RelationalExpression>, ShiftExpression),
	GreatEqual(Box<RelationalExpression>, ShiftExpression),
}

pub enum ShiftExpression {
	AdditiveExpression(AdditiveExpression),
	LeftShift(Box<ShiftExpression>, AdditiveExpression),
	RightShift(Box<ShiftExpression>, AdditiveExpression),
}

pub enum AdditiveExpression {
	MultiplicativeExpression(MultiplicativeExpression),
	Add(Box<AdditiveExpression>, MultiplicativeExpression),
	Sub(Box<AdditiveExpression>, MultiplicativeExpression),
}

pub enum MultiplicativeExpression {
	CastExpression(CastExpression),
	Mul(Box<MultiplicativeExpression>, CastExpression),
	Div(Box<MultiplicativeExpression>, CastExpression),
	Mod(Box<MultiplicativeExpression>, CastExpression),
}

pub enum CastExpression {
	UnaryExpression(Box<UnaryExpression>),
	TypeName(TypeName, Box<CastExpression>),
}

pub struct TypeName {
	specifier_qualifier_list: decl::SpecifierQualifierList,
	abstract_declarator: Option<decl::AbstractDeclarator>,
}

pub struct ConstantExpression(ConditionalExpression);
