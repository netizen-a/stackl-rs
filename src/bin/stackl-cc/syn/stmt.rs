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
	ExpressionStatement(ExprStatement),
	SelectionStatement(SelectionStatement),
	IterationStatement(IterationStatement),
	JumpStatement(JumpStatement),
}

/// (6.8.1) labeled-statement
pub enum LabeledStatement {
	Label(tok::Ident, Box<Statement>),
	Case(expr::ConstantExpr, Box<Statement>),
	Default(Box<Statement>),
}

/// (6.8.3) expression-statement
pub struct ExprStatement(Option<expr::Expr>);

/// (6.8.4) selection-statement
pub enum SelectionStatement {
	If {
		condition: expr::Expr,
		statement_true: Box<Statement>,
		statement_false: Option<Box<Statement>>,
	},
	Switch {
		expr: expr::Expr,
		statement: Box<Statement>,
	},
}

/// (6.8.5) iteration-statement
pub enum IterationStatement {
	While {
		condition: expr::Expr,
		statement: Box<Statement>,
	},
	DoWhile {
		statement: Box<Statement>,
		condition: expr::Expr,
	},
	ForExpr {
		init_expr: Option<expr::Expr>,
		condition: Option<expr::Expr>,
		iteration: Option<expr::Expr>,
		statement: Box<Statement>,
	},
	ForDecl {
		init_decl: decl::Declaration,
		condition: Option<expr::Expr>,
		iteration: Option<expr::Expr>,
		statement: Box<Statement>,
	},
}

/// (6.8.6) jump-statement
pub enum JumpStatement {
	Goto(tok::Ident),
	Continue,
	Break,
	Return(Option<expr::Expr>),
}
