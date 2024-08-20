// Copyright (c) 2024-2026 Jonathan A. Thomason

//! Declarations

use std::{
	collections::VecDeque,
	fmt,
};

use super::Identifier;
use super::expr;
use crate::data_type::DataType;
use crate::diagnostics::Span;
use crate::synthesis::icg;
use crate::{
	analysis::tok,
	diagnostics::{
		self as diag,
		ToSpan,
	},
};
use stackl::ssa::data as ssa;

/// (6.7) declaration
#[derive(Debug, Default)]
pub struct Declaration {
	/// (6.7) declaration-specifiers
	pub specifiers: Specifiers,
	/// (6.7) init-declarator-list
	pub init_declarator_list: Box<[InitDeclarator]>,
}

#[derive(Debug, Default, Clone)]
pub struct Specifiers {
	first_span: diag::Span,
	pub storage_classes: Box<[StorageClassSpecifier]>,
	pub type_specifiers: Box<[TypeSpecifier]>,
	pub is_const: bool,
	pub is_volatile: bool,
	pub restrict_list: Box<[diag::Span]>,
	pub inline_list: Box<[diag::Span]>,
	pub storage: Option<ssa::StorageClass>,
	pub layout: Option<icg::DataLayout>,
}

impl From<Vec<SpecifierKind>> for Specifiers {
	fn from(value: Vec<SpecifierKind>) -> Self {
		// grammar should gaurentee that vector is not empty
		assert!(!value.is_empty());
		let mut specifiers = Specifiers::default();
		let mut storage_class_list = vec![];
		let mut type_specifier_list = vec![];
		let mut restrict_list = vec![];
		let mut inline_list = vec![];
		for (i, kind) in value.iter().enumerate() {
			match kind {
				SpecifierKind::StorageClassSpecifier(inner) => {
					if i == 0 {
						specifiers.first_span = inner.span.clone();
					}
					storage_class_list.push(inner.clone())
				}
				SpecifierKind::TypeSpecifier(inner) => {
					if i == 0 {
						specifiers.first_span = inner.span();
					}
					type_specifier_list.push(inner.clone())
				}
				SpecifierKind::TypeQualifier(inner) => {
					if i == 0 {
						specifiers.first_span = inner.span.clone();
					}
					match inner.kind {
						TypeQualifierKind::Const => specifiers.is_const = true,
						TypeQualifierKind::Volatile => specifiers.is_volatile = true,
						TypeQualifierKind::Restrict => restrict_list.push(inner.span.clone()),
					}
				}
				SpecifierKind::Inline(span) => {
					if i == 0 {
						specifiers.first_span = span.clone();
					}
					inline_list.push(span.clone())
				}
			}
		}
		specifiers.storage_classes = storage_class_list.into_boxed_slice();
		specifiers.type_specifiers = type_specifier_list.into_boxed_slice();
		specifiers.inline_list = inline_list.into_boxed_slice();
		specifiers.restrict_list = restrict_list.into_boxed_slice();
		specifiers
	}
}

impl ToSpan for Specifiers {
	fn to_span(&self) -> Span {
		self.first_span.clone()
	}
}

/// (6.7) declaration-specifiers
#[derive(Debug)]
pub enum SpecifierKind {
	StorageClassSpecifier(StorageClassSpecifier),
	TypeSpecifier(TypeSpecifier),
	/// (6.7.3) type-qualifier
	TypeQualifier(TypeQualifier),
	/// (6.7.4) function-specifier
	Inline(diag::Span),
}

/// (6.7) init-declarator
#[derive(Debug)]
pub struct InitDeclarator {
	pub identifier: Identifier,
	pub declarator: Box<[Declarator]>,
	pub initializer: Option<Initializer>,
}

#[derive(Debug, Clone, Copy)]
pub enum StorageClass {
	Static,
	Auto,
	Typedef,
	Register,
	Extern,
}

impl fmt::Display for StorageClass {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let sc = match self {
			Self::Static => "static",
			Self::Auto => "auto",
			Self::Typedef => "typedef",
			Self::Register => "register",
			Self::Extern => "extern",
		};
		write!(f, "{sc}")
	}
}

/// (6.7.1) storage-class-specifier
#[derive(Debug, Clone)]
pub struct StorageClassSpecifier {
	pub span: diag::Span,
	pub kind: StorageClass,
}

impl diag::ToSpan for StorageClassSpecifier {
	fn to_span(&self) -> diag::Span {
		self.span.clone()
	}
}

/// (6.7.2) type-specifier
#[derive(Debug, Clone)]
pub enum TypeSpecifier {
	Void(diag::Span),
	Char(diag::Span),
	Short(diag::Span),
	Int(diag::Span),
	Long(diag::Span),
	Float(diag::Span),
	Double(diag::Span),
	Signed(diag::Span),
	Unsigned(diag::Span),
	Bool(diag::Span),
	StructOrUnionSpecifier(StructOrUnionSpecifier),
	EnumSpecifier(EnumSpecifier),
	/// (6.7.7) typedef-name
	TypedefName(Identifier),
}

impl TypeSpecifier {
	fn span(&self) -> diag::Span {
		match self {
			Self::Void(span) => span.clone(),
			Self::Char(span) => span.clone(),
			Self::Short(span) => span.clone(),
			Self::Int(span) => span.clone(),
			Self::Long(span) => span.clone(),
			Self::Float(span) => span.clone(),
			Self::Double(span) => span.clone(),
			Self::Signed(span) => span.clone(),
			Self::Unsigned(span) => span.clone(),
			Self::Bool(span) => span.clone(),
			Self::StructOrUnionSpecifier(spec) => match spec.ident.as_ref() {
				Some(ident) => ident.span.clone(),
				None => spec.struct_or_union.span.clone(),
			},
			Self::EnumSpecifier(spec) => match spec.identifier.as_ref() {
				Some(ident) => ident.span.clone(),
				None => spec.tag_span.clone(),
			},
			Self::TypedefName(ident) => ident.to_span(),
		}
	}
}

/// (6.7.2.2) enum-specifier
#[derive(Debug, Clone)]
pub struct EnumSpecifier {
	pub tag_span: diag::Span,
	pub identifier: Option<Identifier>,
	/// (6.7.2.2) enumerator-list
	pub enumerator_list: Box<[Enumerator]>,
}

/// (6.7.2.2) enumerator
#[derive(Debug, Clone)]
pub struct Enumerator {
	/// (6.4.4.3) enumeration-constant
	pub enumeration_constant: Identifier,
	pub constant_expr: Option<expr::Expr>,
}

/// (6.7.2.1) struct-or-union-specifier
#[derive(Debug, Clone)]
pub struct StructOrUnionSpecifier {
	/// (6.7.2.1) struct-or-union
	pub struct_or_union: StructOrUnion,
	pub ident: Option<Identifier>,
	/// (6.7.2.1) struct-declaration-list
	pub struct_declaration_list: Box<[StructDeclaration]>,
}

#[derive(Debug, Clone, Copy)]
pub enum StructOrUnionKind {
	Struct,
	Union,
}

impl fmt::Display for StructOrUnionKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			Self::Struct => "struct",
			Self::Union => "union",
		};
		write!(f, "{s}")
	}
}

/// (6.7.2.1) struct-or-union
#[derive(Debug, Clone)]
pub struct StructOrUnion {
	pub span: diag::Span,
	pub kind: StructOrUnionKind,
}

/// (6.7.2.1) struct-declaration
#[derive(Debug, Clone)]
pub struct StructDeclaration {
	pub specifiers: Specifiers,
	pub struct_declarator_list: Box<[StructDeclarator]>,
}

/// (6.7.2.1) struct-declarator
#[derive(Debug, Clone)]
pub struct StructDeclarator {
	pub ident: Option<Identifier>,
	pub declarators: Box<[Declarator]>,
	pub const_expr: Option<expr::Expr>,
}

/// (6.7.8) initializer
#[derive(Debug, Clone)]
pub enum Initializer {
	Expr(expr::Expr),
	InitializerList(InitializerList),
}

#[derive(Debug, Clone)]
pub struct ArrayDecl {
	pub span: diag::Span,
	/// (6.7.5) type-qualifier-list
	pub type_qualifiers: Vec<TypeQualifier>,
	pub assignment_expr: Option<expr::Expr>,
	pub has_static: bool,
	pub has_star: bool,
}

/// (6.7.5) direct-declarator
#[derive(Debug, Clone)]
pub enum Declarator {
	Pointer(PtrDecl),
	Array(ArrayDecl),
	/// ( parameter-type-list )
	ParamList(ParamList),
	/// ( identifier-list_opt )
	IdentList(IdentList),
}

#[derive(Debug, Clone)]
pub struct IdentList {
	pub span: diag::Span,
	pub ident_list: Vec<Identifier>,
}

/// (6.7.5) parameter-type-list
#[derive(Debug, Clone)]
pub struct ParamList {
	/// (6.7.5) parameter-list
	pub param_list: Vec<ParameterDeclaration>,
	pub is_variadic: bool,
}

/// (6.7.5) pointer
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PtrDecl {
	pub is_const: bool,
	pub is_volatile: bool,
	pub is_restrict: bool,
}

impl From<&[TypeQualifier]> for PtrDecl {
	fn from(value: &[TypeQualifier]) -> Self {
		Self {
			is_const: value
				.iter()
				.find(|q| matches!(q.kind, TypeQualifierKind::Const))
				.is_some(),
			is_volatile: value
				.iter()
				.find(|q| matches!(q.kind, TypeQualifierKind::Volatile))
				.is_some(),
			is_restrict: value
				.iter()
				.find(|q| matches!(q.kind, TypeQualifierKind::Restrict))
				.is_some(),
		}
	}
}

/// (6.7.5) parameter-declaration
#[derive(Debug, Clone)]
pub struct ParameterDeclaration {
	pub ident: Option<Identifier>,
	pub specifiers: Specifiers,
	pub declarators: VecDeque<Declarator>,
}

/// (6.7.3) type-qualifier
#[derive(Debug, Clone)]
pub enum TypeQualifierKind {
	Const,
	Restrict,
	Volatile,
}
#[derive(Debug, Clone)]
pub struct TypeQualifier {
	pub span: diag::Span,
	pub kind: TypeQualifierKind,
}

/// (6.7.6) type-name
#[derive(Debug, Clone)]
pub struct TypeName {
	/// specifier-qualifier-list
	pub specifiers: Specifiers,
	/// abstract-declarator_opt
	pub declarators: Vec<Declarator>,
}

impl ToSpan for TypeName {
	fn to_span(&self) -> Span {
		self.specifiers.to_span()
	}
}

/// (6.7.8) initializer-list
#[derive(Debug, Clone)]
pub struct InitializerList {
	pub span: Span,
	pub list: Box<[(Box<[Designator]>, Initializer)]>,
}

/// (6.7.8) designator
#[derive(Debug, Clone)]
pub enum Designator {
	ConstExpr(expr::Expr),
	Dot(Identifier),
}
