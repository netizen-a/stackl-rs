// Copyright (c) 2024-2026 Jonathan A. Thomason

use super::Identifier;
use super::{
	decl,
	expr,
};
use crate::analysis::syn::StringLiteral;
use crate::analysis::tok;
use crate::diagnostics as diag;

/// (6.8.2) compound-statement
#[derive(Debug)]
pub struct CompoundStmt {
	pub lcurly: diag::Span,
	pub blocks: Vec<BlockItem>,
	pub rcurly: diag::Span,
}

/// (6.8.2) block-item
#[derive(Debug)]
pub enum BlockItem {
	Declaration(decl::Declaration),
	Statement(Stmt),
	Pragma(tok::Pragma),
	Error,
}

/// (6.8) statement
#[derive(Debug)]
pub enum Stmt {
	Label(LabeledStmt),
	Compound(CompoundStmt),
	Expr(ExprStmt),
	Select(SelectStmt),
	Iter(IterStmt),
	Jump(JumpStmt),
	Asm(AsmStmt),
	Error,
}

#[derive(Debug)]
pub enum AsmQualifier {
	Volatile,
	Inline,
	Goto,
}

#[derive(Debug)]
pub struct AsmStmt(pub StringLiteral);

/// (6.8.1) labeled-statement
#[derive(Debug)]
pub enum LabeledStmt {
	Label(Identifier, Box<Stmt>),
	Case(expr::Expr, Box<Stmt>),
	Default(Box<Stmt>),
}

/// (6.8.3) expression-statement
#[derive(Debug)]
pub struct ExprStmt(pub Option<expr::Expr>);

/// (6.8.4) selection-statement
#[derive(Debug)]
pub enum SelectStmt {
	If {
		stmt_cond: expr::Expr,
		stmt_then: Box<Stmt>,
		stmt_else: Option<Box<Stmt>>,
	},
	Switch {
		expr: expr::Expr,
		stmt: Box<Stmt>,
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
	Goto(Identifier),
	Continue,
	Break,
	Return(Option<expr::Expr>),
}
