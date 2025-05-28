use super::{decl, expr};
use crate::tok;

/// (6.8.2) compound-statement
pub struct CompoundStatement(Option<BlockItemList>);

/// (6.8.2) block-item-list
pub struct BlockItemList(Vec<BlockItem>);

/// (6.8.2) block-item
pub enum BlockItem {
	Declaration(decl::Declaration),
	Statement(Statement),
}

/// (6.8) statement
pub enum Statement {
	LabeledStatement(LabeledStatement),
	CompoundStatement(CompoundStatement),
	ExpressionStatement(ExpressionStatement),
	SelectionStatement(SelectionStatement),
	IterationStatement(IterationStatement),
	JumpStatement(JumpStatement),
}

/// (6.8.1) labeled-statement
pub enum LabeledStatement {
	Label(tok::Ident, Box<Statement>),
	Case(expr::ConstantExpression, Box<Statement>),
	Default(Box<Statement>),
}

/// (6.8.3) expression-statement
pub struct ExpressionStatement(Option<expr::Expression>);

/// (6.8.4) selection-statement
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

/// (6.8.5) iteration-statement
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

/// (6.8.6) jump-statement
pub enum JumpStatement {
	Goto(tok::Ident),
	Continue,
	Break,
	Return(Option<expr::Expression>),
}
