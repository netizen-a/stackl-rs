//! Declarations

use super::expr;
use crate::tok;

/// (6.9.1) declaration-list
pub struct DeclarationList(Vec<Declaration>);

/// (6.7) declaration
#[derive(Debug)]
pub struct Declaration {
	/// (6.7) declaration-specifiers
	pub declaration_specifiers: Vec<DeclarationSpecifier>,
	/// (6.7) init-declarator-list
	pub init_declarator_list: Vec<InitDeclarator>,
}

/// (6.7) declaration-specifiers
#[derive(Debug)]
pub enum DeclarationSpecifier {
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
#[derive(Debug)]
pub enum StorageClassSpecifier {
	Typedef,
	Extern,
	Static,
	Auto,
	Register(Option<tok::Ident>),
}

/// (6.7.2) type-specifier
#[derive(Debug)]
pub enum TypeSpecifier {
	Void,
	Char,
	Short,
	Int,
	Long,
	Float,
	Double,
	Signed,
	Unsigned,
	Bool,
	StructOrUnionSpecifier(StructOrUnionSpecifier),
	EnumSpecifier(EnumSpecifier),
	/// (6.7.7) typedef-name
	TypedefName(tok::Ident),
}

/// (6.7.2.2) enum-specifier
#[derive(Debug)]
pub struct EnumSpecifier {
	pub identifier: Option<tok::Ident>,
	/// (6.7.2.2) enumerator-list
	pub enumerator_list: Vec<Enumerator>,
}

/// (6.7.2.2) enumerator
#[derive(Debug)]
pub struct Enumerator {
	/// (6.4.4.3) enumeration-constant
	pub enumeration_constant: tok::Ident,
	pub constant_expr: Option<expr::Expr>,
}

/// (6.7.2.1) struct-or-union-specifier
#[derive(Debug)]
pub struct StructOrUnionSpecifier {
	/// (6.7.2.1) struct-or-union
	pub struct_or_union: tok::Keyword,
	pub identifier: Option<tok::Ident>,
	pub struct_declaration_list: Option<StructDeclarationList>,
}

/// (6.7.2.1) struct-declaration-list
#[derive(Debug)]
pub struct StructDeclarationList(pub Vec<StructDeclaration>);

/// (6.7.2.1) struct-declaration
#[derive(Debug)]
pub struct StructDeclaration {
	pub specifier_qualifier_list: Vec<SpecifierQualifier>,
	pub struct_declaration_list: Vec<StructDeclarator>,
}

/// (6.7.2.1) struct-declarator
#[derive(Debug)]
pub struct StructDeclarator {
	pub declarator: Option<Declarator>,
	pub constant_expr: Option<expr::Expr>,
}

/// (6.7.8) initializer
#[derive(Debug)]
pub enum Initializer {
	Expr(expr::Expr),
	InitializerList(InitializerList),
}

#[derive(Debug)]
pub enum FunctionSpecifier {
	Inline,
}

/// (6.7.5) declarator
#[derive(Debug)]
pub struct Declarator {
	pub pointer: Vec<Pointer>,
	pub direct_declarator: Vec<DirectDeclarator>,
}

/// (6.7.5) direct-declarator
#[derive(Debug)]
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
#[derive(Debug)]
pub struct ParameterTypeList {
	/// (6.7.5) parameter-list
	pub parameter_list: Vec<ParameterDeclaration>,
	pub comma_ellipsis: bool,
}

/// (6.7.5) pointer
#[derive(Debug)]
pub struct Pointer {
	/// (6.7.5) type-qualifier-list
	pub type_qualifier_list: Vec<TypeQualifier>,
}

/// (6.7.5) parameter-declaration
#[derive(Debug)]
pub enum ParameterDeclaration {
	Declarator {
		/// (6.7) declaration-specifiers
		declaration_specifiers: Vec<DeclarationSpecifier>,
		/// (6.7.5) declarator
		declarator: Declarator,
	},
	AbstractDeclarator {
		/// (6.7) declaration-specifiers
		declaration_specifiers: Vec<DeclarationSpecifier>,
		abstract_declarator: Option<AbstractDeclarator>,
	},
}

/// (6.7.6) abstract-declarator
#[derive(Debug)]
pub enum AbstractDeclarator {
	Pointer(Pointer),
	DirectAbstractDeclarator {
		pointer: Pointer,
		direct_abstract_declarator: Vec<DirectAbstractDeclarator>,
	},
}

/// (6.7.6) direct-abstract-declarator
#[derive(Debug)]
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
#[derive(Debug)]
pub enum TypeQualifier {
	Const,
	Restrict,
	Volatile,
}
/// (6.7.6) type-name
#[derive(Debug)]
pub struct TypeName {
	/// specifier-qualifier-list
	pub specifier_qualifier_list: Vec<SpecifierQualifier>,
	/// abstract-declarator_opt
	pub abstract_declarator: Option<AbstractDeclarator>,
}

#[derive(Debug)]
pub enum SpecifierQualifier {
	TypeSpecifier(TypeSpecifier),
	TypeQualifier(TypeQualifier),
}

/// (6.7.8) initializer-list
#[derive(Debug)]
pub struct InitializerList(Vec<(Option<Designation>, Initializer)>);

/// (6.7.8) designation
#[derive(Debug)]
pub struct Designation(Vec<Designator>);

/// (6.7.8) designator
#[derive(Debug)]
pub enum Designator {
	ConstantExpr(expr::Expr),
	Dot(tok::Ident),
}
