// Copyright (c) 2024-2026 Jonathan A. Thomason

// use crate::data_types::DataType;

use std::path;

use crate::{
	analysis::{
		syn::StorageClass,
		tok,
	},
	data_type::DataType,
};

#[derive(Debug, Clone)]
pub enum DiagKind {
	Internal(&'static str),
	Trace(String),
	FileNotFound(path::PathBuf),
	ErrorDirective(String),
	UnexpectedEof,
	UnexpectedEscape,
	UnrecognizedToken {
		token: String,
		expected: Vec<String>,
	},
	InvalidToken,
	ExtraToken,
	//HeaderNameError,
	MultStorageClasses,
	DuplicateSpecifier(String),
	BothSpecifiers(String, String),
	InvalidRestrict,
	// TypeError { found: DataType, expected: DataType },
	CastError {
		from_type: DataType,
		to_type: DataType,
	},
	MultipleTypes,
	TooLong,
	ImplicitInt(Option<String>),
	ArrayOfFunctions {
		name: Option<String>,
		dtype: DataType,
	},
	FnRetFn(Option<String>),
	OmittedParamName,
	DeclIdentList,
	UnboundVLA,
	InvalidStar,
	IfAssign,
	OnlyVoid,
	ArrayOfVoid(Option<String>),
	IllegalStorage(StorageClass),
	BitfieldRange(Option<String>),
	BitfieldNonIntegral(Option<String>),
	NonIntConstExpr,
	InitializerNotConst,
	EnumRange,
	EnumNonIntegral(String),
	ArrayMaxRange,
	ArrayMinRange,
	DeclaratorLimit,
	ParameterLimit,
	UndefPredef,
	RedefPredef,
	DirectiveExtraTokens(tok::Directive),
	DirectiveLineNotSimple,
	DirectiveLineMinRange,
	DirectiveLineMaxRange,
	DirectiveLineFilename,
	DirectiveIncludeExtraTokens,
	DirectivePragma,
	StructNoNamedMembers,
	ArrayArgTooSmall,
	SymbolAlreadyExists(String, DataType),
	ArrayDeclIncomplete,
	ArrayExcessElements,
	VlaInitList,
	SymbolUndeclared {
		name: String,
		in_func: bool,
	},
	LabeledDeclaration,
	LabeledCompoundEnd,
	PragmaCxLimitedRange,
	PragmaIgnored,
}
