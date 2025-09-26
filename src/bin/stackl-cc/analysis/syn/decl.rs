//! Declarations

use super::expr;
use crate::{analysis::tok, diagnostics as diag};

/// (6.9.1) declaration-list
pub struct DeclarationList(Vec<Declaration>);

/// (6.7) declaration
#[derive(Debug, Default)]
pub struct Declaration {
	/// (6.7) declaration-specifiers
	pub specifiers: Specifiers,
	/// (6.7) init-declarator-list
	pub init_declarator_list: Vec<InitDeclarator>,
}

#[derive(Debug, Default, Clone)]
pub struct Specifiers {
	pub first_span: Option<diag::Span>,
	pub storage_classes: Vec<StorageClassSpecifier>,
	pub type_specifiers: Vec<TypeSpecifier>,
	pub is_const: bool,
	pub is_volatile: bool,
	pub restrict_list: Vec<diag::Span>,
	pub inline_list: Vec<diag::Span>,
}

impl From<Vec<SpecifierKind>> for Specifiers {
	fn from(value: Vec<SpecifierKind>) -> Self {
		let mut specifiers = Specifiers::default();
		let mut is_first = true;
		for kind in value {
			match kind {
				SpecifierKind::StorageClassSpecifier(inner) => {
					if is_first {
						is_first = false;
						specifiers.first_span = Some(inner.span.clone());
					}
					specifiers.storage_classes.push(inner)
				}
				SpecifierKind::TypeSpecifier(inner) => {
					if is_first {
						is_first = false;
						specifiers.first_span = Some(inner.span());
					}
					specifiers.type_specifiers.push(inner)
				}
				SpecifierKind::TypeQualifier(inner) => {
					if is_first {
						is_first = false;
						specifiers.first_span = Some(inner.span.clone());
					}
					match inner.kind {
						TypeQualifierKind::Const => specifiers.is_const = true,
						TypeQualifierKind::Volatile => specifiers.is_volatile = true,
						TypeQualifierKind::Restrict => specifiers.restrict_list.push(inner.span),
					}
				}
				SpecifierKind::Inline(span) => {
					if is_first {
						is_first = false;
						specifiers.first_span = Some(span.clone());
					}
					specifiers.inline_list.push(span)
				}
			}
		}
		specifiers
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
	pub identifier: tok::Ident,
	pub declarator: Vec<Declarator>,
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

/// (6.7.1) storage-class-specifier
#[derive(Debug, Clone)]
pub struct StorageClassSpecifier {
	pub span: diag::Span,
	pub storage_class: StorageClass,
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
	TypedefName {
		span: diag::Span,
		name: tok::Ident,
	},
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
			Self::TypedefName { span, .. } => span.clone(),
		}
	}
}

/// (6.7.2.2) enum-specifier
#[derive(Debug, Clone)]
pub struct EnumSpecifier {
	pub tag_span: diag::Span,
	pub identifier: Option<tok::Ident>,
	/// (6.7.2.2) enumerator-list
	pub enumerator_list: Vec<Enumerator>,
}

/// (6.7.2.2) enumerator
#[derive(Debug, Clone)]
pub struct Enumerator {
	/// (6.4.4.3) enumeration-constant
	pub enumeration_constant: tok::Ident,
	pub constant_expr: Option<expr::Expr>,
}

/// (6.7.2.1) struct-or-union-specifier
#[derive(Debug, Clone)]
pub struct StructOrUnionSpecifier {
	/// (6.7.2.1) struct-or-union
	pub struct_or_union: StructOrUnion,
	pub ident: Option<tok::Ident>,
	/// (6.7.2.1) struct-declaration-list
	pub struct_declaration_list: Vec<StructDeclaration>,
}

/// (6.7.2.1) struct-or-union
#[derive(Debug, Clone)]
pub struct StructOrUnion {
	pub span: diag::Span,
	pub keyword: tok::Keyword,
}

/// (6.7.2.1) struct-declaration
#[derive(Debug, Clone)]
pub struct StructDeclaration {
	pub specifiers: Specifiers,
	pub struct_declaration_list: Vec<StructDeclarator>,
}

/// (6.7.2.1) struct-declarator
#[derive(Debug, Clone)]
pub struct StructDeclarator {
	pub identifier: Option<tok::Ident>,
	pub declarator: Vec<Declarator>,
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
	pub ident_list: Vec<tok::Ident>,
}

/// (6.7.5) parameter-type-list
#[derive(Debug, Clone)]
pub struct ParamList {
	pub span: diag::Span,
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
	pub span: diag::Span,
	pub name: Option<tok::Ident>,
	pub specifiers: Specifiers,
	pub declarators: Vec<Declarator>,
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

/// (6.7.8) initializer-list
#[derive(Debug, Clone)]
pub struct InitializerList(pub Vec<(Option<Designation>, Initializer)>);

/// (6.7.8) designation
#[derive(Debug, Clone)]
pub struct Designation(pub Vec<Designator>);

/// (6.7.8) designator
#[derive(Debug, Clone)]
pub enum Designator {
	ConstantExpr(expr::Expr),
	Dot(tok::Ident),
}
