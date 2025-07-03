use super::{decl, expr};
use crate::analysis::tok;

/// (6.8.2) compound-statement
#[derive(Debug)]
pub struct CompoundStmt(pub Vec<BlockItem>);

/// (6.8.2) block-item
#[derive(Debug)]
pub enum BlockItem {
	Declaration(decl::Declaration),
	Statement(Stmt),
}

/// (6.8) statement
#[derive(Debug)]
pub enum Stmt {
	LabeledStatement(LabeledStmt),
	CompoundStatement(CompoundStmt),
	ExpressionStatement(ExprStmt),
	SelectionStatement(SelectionStmt),
	IterationStatement(IterationStmt),
	JumpStatement(JumpStmt),
}

/// (6.8.1) labeled-statement
#[derive(Debug)]
pub enum LabeledStmt {
	Label(tok::Ident, Box<Stmt>),
	Case(expr::Expr, Box<Stmt>),
	Default(Box<Stmt>),
}

/// (6.8.3) expression-statement
#[derive(Debug)]
pub struct ExprStmt(Option<expr::Expr>);

/// (6.8.4) selection-statement
#[derive(Debug)]
pub enum SelectionStmt {
	If {
		condition: expr::Expr,
		stmt_true: Box<Stmt>,
		stmt_false: Option<Box<Stmt>>,
	},
	Switch {
		expr: expr::Expr,
		statement: Box<Stmt>,
	},
}

/// (6.8.5) iteration-statement
#[derive(Debug)]
pub enum IterationStmt {
	While {
		cond: expr::Expr,
		stmt: Box<Stmt>,
	},
	DoWhile {
		stmt: Box<Stmt>,
		cond: expr::Expr,
	},
	ForExpr {
		init_expr: Option<expr::Expr>,
		cond: Option<expr::Expr>,
		iteration: Option<expr::Expr>,
		stmt: Box<Stmt>,
	},
	ForDecl {
		init_decl: decl::Declaration,
		cond: Option<expr::Expr>,
		iteration: Option<expr::Expr>,
		stmt: Box<Stmt>,
	},
}

/// (6.8.6) jump-statement
#[derive(Debug)]
pub enum JumpStmt {
	Goto(tok::Ident),
	Continue,
	Break,
	Return(Option<expr::Expr>),
}
