//! Declarations

use super::expr;
use crate::{analysis::tok, diagnostics as diag};

/// (6.9.1) declaration-list
pub struct DeclarationList(Vec<Declaration>);

/// (6.7) declaration
#[derive(Debug, Default)]
pub struct Declaration {
	/// (6.7) declaration-specifiers
	pub specifiers: DeclarationSpecifiers,
	/// (6.7) init-declarator-list
	pub init_declarator_list: Vec<InitDeclarator>,
}

#[derive(Debug, Default, Clone)]
pub struct DeclarationSpecifiers {
	pub storage_classes: Vec<StorageClassSpecifier>,
	pub type_specifiers: Vec<TypeSpecifier>,
	pub type_qualifiers: Vec<TypeQualifier>,
	pub func_specifiers: Vec<FunctionSpecifier>,
}

impl From<Vec<DeclarationSpecifierKind>> for DeclarationSpecifiers {
	fn from(value: Vec<DeclarationSpecifierKind>) -> Self {
		let mut specifiers = DeclarationSpecifiers::default();
		for kind in value {
			use DeclarationSpecifierKind::*;
			match kind {
				StorageClassSpecifier(inner) => specifiers.storage_classes.push(inner),
				TypeSpecifier(inner) => specifiers.type_specifiers.push(inner),
				TypeQualifier(inner) => specifiers.type_qualifiers.push(inner),
				FunctionSpecifier(inner) => specifiers.func_specifiers.push(inner),
			}
		}
		specifiers
	}
}

/// (6.7) declaration-specifiers
#[derive(Debug)]
pub enum DeclarationSpecifierKind {
	StorageClassSpecifier(StorageClassSpecifier),
	TypeSpecifier(TypeSpecifier),
	/// (6.7.3) type-qualifier
	TypeQualifier(TypeQualifier),
	/// (6.7.4) function-specifier
	FunctionSpecifier(FunctionSpecifier),
}

/// (6.7) init-declarator
#[derive(Debug)]
pub struct InitDeclarator {
	pub declarator: Declarator,
	pub initializer: Option<Initializer>,
}

/// (6.7.1) storage-class-specifier
#[derive(Debug, Clone)]
pub struct StorageClassSpecifier {
	pub span: diag::Span,
	pub keyword: tok::Keyword,
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
	TypedefName{
		span: diag::Span,
		name: tok::Ident
	},
}

/// (6.7.2.2) enum-specifier
#[derive(Debug, Clone)]
pub struct EnumSpecifier {
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
	pub specifier_qualifier_list: Vec<SpecifierQualifier>,
	pub struct_declaration_list: Vec<StructDeclarator>,
}

/// (6.7.2.1) struct-declarator
#[derive(Debug, Clone)]
pub struct StructDeclarator {
	pub declarator: Option<Declarator>,
	pub constant_expr: Option<expr::Expr>,
}

/// (6.7.8) initializer
#[derive(Debug, Clone)]
pub enum Initializer {
	Expr(expr::Expr),
	InitializerList(InitializerList),
}

#[derive(Debug, Clone, Copy)]
pub enum FunctionSpecifier {
	Inline,
}

/// (6.7.5) declarator
#[derive(Debug, Clone)]
pub struct Declarator {
	pub pointer: Vec<Pointer>,
	pub direct_declarator: Vec<DirectDeclarator>,
}

/// (6.7.5) direct-declarator
#[derive(Debug, Clone)]
pub enum DirectDeclarator {
	Identifier(tok::Ident),
	/// ( declarator )
	Declarator(Box<Declarator>),
	Array {
		/// (6.7.5) type-qualifier-list
		type_qualifier_list: Vec<TypeQualifier>,
		assignment_expr: Option<expr::Expr>,
		has_static: bool,
		has_ptr: bool,
	},
	/// ( parameter-type-list )
	ParameterTypeList(ParameterTypeList),
	/// ( identifier-list_opt )
	IdentifierList(Vec<tok::Ident>),
}

/// (6.7.5) parameter-type-list
#[derive(Debug, Clone)]
pub struct ParameterTypeList {
	/// (6.7.5) parameter-list
	pub parameter_list: Vec<ParameterDeclaration>,
	pub comma_ellipsis: bool,
}

/// (6.7.5) pointer
#[derive(Debug, Clone)]
pub struct Pointer {
	/// (6.7.5) type-qualifier-list
	pub type_qualifier_list: Vec<TypeQualifier>,
}

/// (6.7.5) parameter-declaration
#[derive(Debug, Clone)]
pub struct ParameterDeclaration {
	pub specifiers: DeclarationSpecifiers,
	pub parameter_declarator: ParameterDeclarator,
}

#[derive(Debug, Clone)]
pub enum ParameterDeclarator {
	Declarator(Declarator),
	AbstractDeclarator(Option<AbstractDeclarator>),
}

/// (6.7.6) abstract-declarator
#[derive(Debug, Clone)]
pub enum AbstractDeclarator {
	Pointer(Pointer),
	DirectAbstractDeclarator {
		pointer: Pointer,
		direct_abstract_declarator: Vec<DirectAbstractDeclarator>,
	},
}

/// (6.7.6) direct-abstract-declarator
#[derive(Debug, Clone)]
pub enum DirectAbstractDeclarator {
	/// ( abstract-declarator )
	AbstractDeclarator(AbstractDeclarator),
	Array {
		direct_abstract_declarator: Vec<TypeQualifier>,
		assignment_expr: Option<expr::Expr>,
		has_static: bool,
	},
	/// direct-abstract-declarator_opt [ * }
	ArrayPointer,
	/// direct-abstract-declarator_opt ( parameter-type-list_opt )
	ParameterTypeList(Option<ParameterTypeList>),
}

/// (6.7.3) type-qualifier
#[derive(Debug, Clone)]
pub enum TypeQualifier {
	Const,
	Restrict,
	Volatile,
}
/// (6.7.6) type-name
#[derive(Debug, Clone)]
pub struct TypeName {
	/// specifier-qualifier-list
	pub specifier_qualifier_list: Vec<SpecifierQualifier>,
	/// abstract-declarator_opt
	pub abstract_declarator: Option<AbstractDeclarator>,
}

#[derive(Debug, Clone)]
pub enum SpecifierQualifier {
	TypeSpecifier(TypeSpecifier),
	TypeQualifier(TypeQualifier),
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
