use super::{decl, expr};
use crate::tok;

pub struct CompoundStatement(Option<BlockItemList>);

pub struct BlockItemList(Vec<BlockItem>);

pub enum BlockItem {
	Declaration(decl::Declaration),
	Statement(Statement),
}

pub enum Statement {
	LabeledStatement(LabeledStatement),
	CompoundStatement(CompoundStatement),
	ExpressionStatement(ExpressionStatement),
	SelectionStatement(SelectionStatement),
	IterationStatement(IterationStatement),
	JumpStatement(JumpStatement),
}

pub enum LabeledStatement {
	Label(tok::Identifier, Box<Statement>),
	Case(expr::ConstantExpression, Box<Statement>),
	Default(Box<Statement>),
}

pub struct ExpressionStatement(Option<expr::Expression>);

pub enum SelectionStatement {
	If {
		condition: expr::Expression,
		statement_true: Box<Statement>,
		statement_false: Option<Box<Statement>>,
	},
	Switch {
		expression: expr::Expression,
		statement: Box<Statement>,
	},
}

pub enum IterationStatement {
	While {
		condition: expr::Expression,
		statement: Box<Statement>,
	},
	DoWhile {
		statement: Box<Statement>,
		condition: expr::Expression,
	},
	ForExpr {
		init_expr: Option<expr::Expression>,
		condition: Option<expr::Expression>,
		iteration: Option<expr::Expression>,
		statement: Box<Statement>,
	},
	ForDecl {
		init_decl: decl::Declaration,
		condition: Option<expr::Expression>,
		iteration: Option<expr::Expression>,
		statement: Box<Statement>,
	},
}

pub enum JumpStatement {
	Goto(tok::Identifier),
	Continue,
	Break,
	Return(Option<expr::Expression>),
}
