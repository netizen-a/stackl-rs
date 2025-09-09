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
	Error,
}

/// (6.8) statement
#[derive(Debug)]
pub enum Stmt {
	Labeled(LabeledStmt),
	Compound(CompoundStmt),
	Expr(ExprStmt),
	Selection(SelectionStmt),
	Iter(IterStmt),
	Jump(JumpStmt),
	Asm(AsmStmt),
}

#[derive(Debug)]
pub enum AsmQualifier {
	Volatile,
	Inline,
	Goto,
}

#[derive(Debug)]
pub struct AsmStmt (pub tok::StrLit);

#[derive(Debug)]
pub struct AsmConstraints {
	pub output_operands: Vec<OutputOperand>,
	pub input_operands: Vec<InputOperand>,
	pub clobber_operands: Vec<tok::StrLit>,
	pub goto_labels: Vec<tok::Ident>,
}

#[derive(Debug)]
pub struct OutputOperand {
	pub prefix: tok::StrLit,
	pub ident: tok::Ident,
}

#[derive(Debug)]
pub struct InputOperand {
	pub prefix: tok::StrLit,
	pub expr: expr::Expr,
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
pub struct ExprStmt(pub Option<expr::Expr>);

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
pub enum IterStmt {
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
