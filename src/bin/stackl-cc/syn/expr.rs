use super::decl;
use crate::tok;

/// (6.5.16) assignment-expression
pub struct AssignmentExpression {
	assignment_expression: Vec<(UnaryExpression, AssignmentOperator)>,
	conditional_expression: ConditionalExpression,
}

/// (6.5.3) unary-expression
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

/// (6.5.2) argument-expression-list
pub struct ArgumentExpressionList(Vec<AssignmentExpression>);

/// (6.5.1) primary-expression
pub enum PrimaryExpression {
	Identifier(tok::Identifier),
	Constant(tok::Constant),
	StringLiteral(tok::StringLiteral),
	Expression(Expression),
}

/// (6.5.17) expression
pub struct Expression(Vec<AssignmentExpression>);

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

/// (6.5.9) equality-expression
pub enum EqualityExpression {
	RelationalExpression(RelationalExpression),
	Equal(Box<EqualityExpression>, RelationalExpression),
	NotEqual(Box<EqualityExpression>, RelationalExpression),
}

/// (6.5.8) relational-expression
pub enum RelationalExpression {
	ShiftExpression(ShiftExpression),
	Less(Box<RelationalExpression>, ShiftExpression),
	Great(Box<RelationalExpression>, ShiftExpression),
	LessEqual(Box<RelationalExpression>, ShiftExpression),
	GreatEqual(Box<RelationalExpression>, ShiftExpression),
}

/// (6.5.7) shift-expression
pub enum ShiftExpression {
	AdditiveExpression(AdditiveExpression),
	LeftShift(Box<ShiftExpression>, AdditiveExpression),
	RightShift(Box<ShiftExpression>, AdditiveExpression),
}

/// (6.5.6) additive-expression
pub enum AdditiveExpression {
	MultiplicativeExpression(MultiplicativeExpression),
	Add(Box<AdditiveExpression>, MultiplicativeExpression),
	Sub(Box<AdditiveExpression>, MultiplicativeExpression),
}

/// (6.5.5) multiplicative-expression
pub enum MultiplicativeExpression {
	CastExpression(CastExpression),
	Mul(Box<MultiplicativeExpression>, CastExpression),
	Div(Box<MultiplicativeExpression>, CastExpression),
	Mod(Box<MultiplicativeExpression>, CastExpression),
}

/// (6.5.4) cast-expression
pub enum CastExpression {
	UnaryExpression(Box<UnaryExpression>),
	TypeName(TypeName, Box<CastExpression>),
}

/// (6.7.6) type-name
pub struct TypeName {
	specifier_qualifier_list: decl::SpecifierQualifierList,
	abstract_declarator: Option<decl::AbstractDeclarator>,
}

/// (6.6) constant-expression
pub struct ConstantExpression(ConditionalExpression);
