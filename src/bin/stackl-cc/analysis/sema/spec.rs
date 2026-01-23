// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::analysis::syn;
use crate::data_type::*;
use crate::diagnostics as diag;
use crate::diagnostics::ToSpan;
use crate::diagnostics::{
	DiagKind,
	Diagnostic,
};
use crate::symtab as sym;
use crate::synthesis::icg;

const SIGNED_STR: &str = "signed";
const UNSIGNED_STR: &str = "unsigned";
const FLOAT_STR: &str = "float";
const DOUBLE_STR: &str = "double";
const LONG_STR: &str = "long";
const CHAR_STR: &str = "char";
const VOID_STR: &str = "void";
const SHORT_STR: &str = "void";
const BOOL_STR: &str = "_Bool";
const LONG_LONG_STR: &str = "long long";
const STRUCT_STR: &str = "struct";

impl super::SemanticParser<'_> {
	pub(super) fn specifiers_storage(
		&mut self,
		specifiers: &mut syn::Specifiers,
	) -> Option<syn::StorageClassSpecifier> {
		let mut storage_class = None;
		for (i, storage_class_specifier) in specifiers.storage_classes.iter().enumerate() {
			if i > 0 {
				let diag = diag::Diagnostic::error(
					diag::DiagKind::MultStorageClasses,
					storage_class_specifier.span.clone(),
				);
				self.diagnostics.push(diag);
				storage_class = None;
			} else {
				storage_class = Some(storage_class_specifier.clone());
			}
		}

		for (i, restrict_span) in specifiers.restrict_list.iter().enumerate() {
			let diag = if i == 0 {
				diag::Diagnostic::error(diag::DiagKind::InvalidRestrict, restrict_span.clone())
			} else {
				diag::Diagnostic::warn(
					diag::DiagKind::DuplicateSpecifier("restrict".to_owned()),
					restrict_span.clone(),
				)
			};
			self.diagnostics.push(diag);
			storage_class = None;
		}
		if let Some(specifier) = &storage_class {
			self.tree_builder
				.add_empty_child(format!("storage-class {}", specifier.kind));
			storage_class
		} else {
			None
		}
	}

	pub(super) fn specifiers_dtype(
		&mut self,
		specifiers: &mut syn::Specifiers,
		in_func: bool,
	) -> Option<DataType> {
		let mut type_kind: Option<TypeKind> = None;
		let mut is_signed: Option<bool> = None;
		let mut long_count = 0;
		for type_spec in specifiers.type_specifiers.iter_mut() {
			match type_spec {
				syn::TypeSpecifier::Void(span) => {
					self.specifier_void(span.to_span(), &mut type_kind, is_signed, long_count)
				}
				syn::TypeSpecifier::Char(span) => {
					self.specifier_char(span.to_span(), &mut type_kind, is_signed, long_count)
				}
				syn::TypeSpecifier::Short(span) => {
					self.specifier_short(span.to_span(), &mut type_kind, is_signed, long_count)
				}
				syn::TypeSpecifier::Int(span) => {
					self.specifier_int(span.to_span(), &mut type_kind, is_signed, long_count)
				}
				syn::TypeSpecifier::Long(span) => {
					self.specifier_long(span.to_span(), &mut type_kind, is_signed, &mut long_count)
				}
				syn::TypeSpecifier::Float(span) => {
					self.specifier_float(span.to_span(), &mut type_kind, is_signed, long_count)
				}
				syn::TypeSpecifier::Double(span) => {
					self.specifier_double(span.to_span(), &mut type_kind, is_signed, long_count)
				}
				syn::TypeSpecifier::Signed(span) => self.specifier_signed(
					span.to_span(),
					&mut type_kind,
					&mut is_signed,
					long_count,
				),
				syn::TypeSpecifier::Unsigned(span) => self.specifier_unsigned(
					span.to_span(),
					&mut type_kind,
					&mut is_signed,
					long_count,
				),
				syn::TypeSpecifier::Bool(span) => {
					self.specifier_bool(span.to_span(), &mut type_kind, is_signed, long_count)
				}
				syn::TypeSpecifier::StructOrUnionSpecifier(specifier) => {
					self.specifier_struct_or_union(
						specifier,
						&mut type_kind,
						is_signed,
						long_count,
						in_func,
					);
					specifiers.layout = type_kind
						.clone()
						.map(|kind| icg::DataLayout::try_from(kind).ok())?;
				}
				syn::TypeSpecifier::EnumSpecifier(specifier) => {
					self.specifier_enum(specifier, &mut type_kind, is_signed, long_count, in_func)
				}
				syn::TypeSpecifier::TypedefName { .. } => todo!("typedef"),
			}
		}

		if type_kind.is_none() {
			match long_count {
				1 => type_kind = Some(TypeKind::Scalar(ScalarType::SLong)),
				2 => type_kind = Some(TypeKind::Scalar(ScalarType::SLong2)),
				_ => {}
			}
		}

		if let Some(TypeKind::Scalar(scalar)) = &mut type_kind {
			if let ScalarType::SInt = scalar {
				match long_count {
					1 => *scalar = ScalarType::SLong,
					2 => *scalar = ScalarType::SLong2,
					_ => {}
				}
			}
			if let Some(is_signed) = is_signed {
				scalar.set_signedness(is_signed);
			}
		}

		if let Some(type_kind) = type_kind {
			let type_qual = TypeQual {
				is_const: specifiers.is_const,
				is_restrict: !specifiers.restrict_list.is_empty(),
				is_volatile: specifiers.is_volatile,
			};
			let data_type = DataType {
				kind: type_kind,
				qual: type_qual,
			};
			Some(data_type)
		} else {
			None
		}
	}
	fn specifier_void(
		&mut self,
		span: diag::Span,
		type_kind: &mut Option<TypeKind>,
		is_signed: Option<bool>,
		long_count: i32,
	) {
		match type_kind {
			None => *type_kind = Some(TypeKind::Void),
			Some(TypeKind::Poison) => {
				// do nothing
			}
			Some(_) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::MultipleTypes,
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
		}
		if long_count > 0 {
			self.diagnostics.push(diag::Diagnostic::error(
				diag::DiagKind::BothSpecifiers(LONG_STR.to_owned(), VOID_STR.to_owned()),
				span.clone(),
			));
			*type_kind = Some(TypeKind::Poison);
		}
	}
	fn specifier_char(
		&mut self,
		span: diag::Span,
		type_kind: &mut Option<TypeKind>,
		is_signed: Option<bool>,
		long_count: i32,
	) {
		match type_kind {
			Some(TypeKind::Poison) => {
				// do nothing
			}
			Some(_) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::MultipleTypes,
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => *type_kind = Some(TypeKind::Scalar(ScalarType::SChar)),
		}
		if long_count > 0 {
			self.diagnostics.push(diag::Diagnostic::error(
				diag::DiagKind::BothSpecifiers(LONG_STR.to_owned(), CHAR_STR.to_owned()),
				span.clone(),
			));
			*type_kind = Some(TypeKind::Poison);
		}
	}

	fn specifier_short(
		&mut self,
		span: diag::Span,
		type_kind: &mut Option<TypeKind>,
		is_signed: Option<bool>,
		long_count: i32,
	) {
		match type_kind {
			Some(TypeKind::Poison) => {
				// do nothing
			}
			Some(_) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::MultipleTypes,
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => *type_kind = Some(TypeKind::Scalar(ScalarType::SShort)),
		}
		if long_count > 0 {
			self.diagnostics.push(diag::Diagnostic::error(
				diag::DiagKind::BothSpecifiers(LONG_STR.to_owned(), SHORT_STR.to_owned()),
				span.clone(),
			));
			*type_kind = Some(TypeKind::Poison);
		}
	}

	fn specifier_int(
		&mut self,
		span: diag::Span,
		type_kind: &mut Option<TypeKind>,
		is_signed: Option<bool>,
		long_count: i32,
	) {
		match type_kind {
			Some(TypeKind::Poison) => {
				// do nothing
			}
			Some(_) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::MultipleTypes,
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => *type_kind = Some(TypeKind::Scalar(ScalarType::SInt)),
		}
	}

	fn specifier_long(
		&mut self,
		span: diag::Span,
		type_kind: &mut Option<TypeKind>,
		is_signed: Option<bool>,
		long_count: &mut i32,
	) {
		*long_count += 1;
		if *long_count > 2 {
			self.diagnostics.push(diag::Diagnostic::error(
				diag::DiagKind::TooLong,
				span.clone(),
			));
			*type_kind = Some(TypeKind::Poison);
		}
		match type_kind {
			Some(TypeKind::Tag(_)) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::MultipleTypes,
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			Some(type_kind) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(LONG_STR.to_owned(), type_kind.to_string()),
					span.clone(),
				));
				*type_kind = TypeKind::Poison;
			}
			None | Some(TypeKind::Scalar(ScalarType::SInt)) => {
				// do nothing
			}
		}
	}
	fn specifier_float(
		&mut self,
		span: diag::Span,
		type_kind: &mut Option<TypeKind>,
		is_signed: Option<bool>,
		long_count: i32,
	) {
		match is_signed {
			Some(true) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(SIGNED_STR.to_owned(), FLOAT_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			Some(false) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(UNSIGNED_STR.to_owned(), FLOAT_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => {
				// do nothing
			}
		}
		match type_kind {
			Some(TypeKind::Poison) => {
				// do nothing
			}
			Some(_) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::MultipleTypes,
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => *type_kind = Some(TypeKind::Scalar(ScalarType::Float)),
		}
		if long_count > 0 {
			self.diagnostics.push(diag::Diagnostic::error(
				diag::DiagKind::BothSpecifiers(LONG_STR.to_owned(), FLOAT_STR.to_owned()),
				span.clone(),
			));
			*type_kind = Some(TypeKind::Poison);
		}
	}
	fn specifier_double(
		&mut self,
		span: diag::Span,
		type_kind: &mut Option<TypeKind>,
		is_signed: Option<bool>,
		long_count: i32,
	) {
		match is_signed {
			Some(true) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(SIGNED_STR.to_owned(), DOUBLE_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			Some(false) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(UNSIGNED_STR.to_owned(), DOUBLE_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => {
				// do nothing
			}
		}
		match type_kind {
			Some(TypeKind::Poison) => {
				// do nothing
			}
			Some(_) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::MultipleTypes,
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => *type_kind = Some(TypeKind::Scalar(ScalarType::Double)),
		}
		if long_count > 1 {
			self.diagnostics.push(diag::Diagnostic::error(
				diag::DiagKind::BothSpecifiers(LONG_LONG_STR.to_owned(), DOUBLE_STR.to_owned()),
				span.clone(),
			));
			*type_kind = Some(TypeKind::Poison);
		}
	}
	fn specifier_signed(
		&mut self,
		span: diag::Span,
		type_kind: &mut Option<TypeKind>,
		is_signed: &mut Option<bool>,
		long_count: i32,
	) {
		match is_signed {
			Some(true) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::DuplicateSpecifier(SIGNED_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			Some(false) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(SIGNED_STR.to_owned(), UNSIGNED_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => *is_signed = Some(true),
		}
		match &type_kind {
			Some(TypeKind::Scalar(ScalarType::Double)) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(SIGNED_STR.to_owned(), DOUBLE_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			Some(TypeKind::Scalar(ScalarType::Float)) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(SIGNED_STR.to_owned(), FLOAT_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			Some(TypeKind::Scalar(_) | TypeKind::Poison) | None => {
				// do nothing
			}
			Some(token) => {
				let expected = vec![
					"identifier".to_string(),
					"\";\"".to_string(),
					"\"(\"".to_string(),
				];
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::UnrecognizedToken {
						token: format!("{token:?}"),
						expected,
					},
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
		}
	}
	fn specifier_unsigned(
		&mut self,
		span: diag::Span,
		type_kind: &mut Option<TypeKind>,
		is_signed: &mut Option<bool>,
		long_count: i32,
	) {
		match is_signed {
			Some(true) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(SIGNED_STR.to_owned(), UNSIGNED_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			Some(false) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::DuplicateSpecifier(UNSIGNED_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => *is_signed = Some(false),
		}
		match type_kind {
			Some(TypeKind::Scalar(ScalarType::Double)) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(UNSIGNED_STR.to_owned(), DOUBLE_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			Some(TypeKind::Scalar(ScalarType::Float)) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(UNSIGNED_STR.to_owned(), FLOAT_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			Some(TypeKind::Scalar(_) | TypeKind::Poison) | None => {
				// do nothing
			}
			Some(token) => {
				let expected = vec![
					"identifier".to_string(),
					"\";\"".to_string(),
					"\"(\"".to_string(),
				];
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::UnrecognizedToken {
						token: format!("{token:?}"),
						expected,
					},
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
		}
	}
	fn specifier_bool(
		&mut self,
		span: diag::Span,
		type_kind: &mut Option<TypeKind>,
		is_signed: Option<bool>,
		long_count: i32,
	) {
		match type_kind {
			Some(TypeKind::Poison) => {
				// do nothing
			}
			Some(_) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::MultipleTypes,
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => *type_kind = Some(TypeKind::Scalar(ScalarType::Bool)),
		}
		if long_count > 0 {
			self.diagnostics.push(diag::Diagnostic::error(
				diag::DiagKind::BothSpecifiers(LONG_STR.to_owned(), BOOL_STR.to_owned()),
				span.clone(),
			));
			*type_kind = Some(TypeKind::Poison);
		}
		match is_signed {
			Some(true) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(SIGNED_STR.to_owned(), BOOL_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			Some(false) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(UNSIGNED_STR.to_owned(), BOOL_STR.to_owned()),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => {
				// do nothing
			}
		}
	}
	fn specifier_struct_or_union(
		&mut self,
		spec: &mut syn::StructOrUnionSpecifier,
		type_kind: &mut Option<TypeKind>,
		is_signed: Option<bool>,
		long_count: i32,
		in_func: bool,
	) {
		let span = match &spec.ident {
			Some(ident) => ident.to_span(),
			None => spec.struct_or_union.span.clone(),
		};
		if type_kind.is_some() || long_count > 0 {
			self.diagnostics.push(diag::Diagnostic::error(
				diag::DiagKind::MultipleTypes,
				span.clone(),
			));
			*type_kind = Some(TypeKind::Poison);
			return;
		}
		let mut members = vec![];
		let mut member_is_named = false;
		for decl in spec.struct_declaration_list.iter_mut() {
			let member_vec = self.struct_declaration(decl, &mut member_is_named, in_func);
			let Some(mut member_vec) = member_vec else {
				*type_kind = Some(TypeKind::Poison);
				return;
			};
			members.append(&mut member_vec);
		}
		let is_incomplete = spec.struct_declaration_list.is_empty();
		if !member_is_named && !is_incomplete {
			let error = diag::Diagnostic::error(diag::DiagKind::StructNoNamedMembers, span.clone());
			self.diagnostics.push(error);
			*type_kind = Some(TypeKind::Poison);
			return;
		}

		let tmp_type_kind: TypeKind;
		match spec.struct_or_union.kind {
			syn::StructOrUnionKind::Struct => {
				tmp_type_kind = TypeKind::Tag(TagKind::Struct(
					spec.ident.as_ref().map(|id| id.name.clone()),
					members,
				))
			}
			syn::StructOrUnionKind::Union => {
				tmp_type_kind = TypeKind::Tag(TagKind::Union(
					spec.ident.as_ref().map(|id| id.name.clone()),
					members,
				))
			}
		}

		if let Some(ident) = &spec.ident {
			if let Some(entry) = self.tag_table.global_lookup(&ident.name.clone()) {
				if is_incomplete {
					if let (TypeKind::Tag(entry_tag), TypeKind::Tag(decl_tag)) =
						(&entry.data_type.kind, tmp_type_kind)
					{
						match (entry_tag, decl_tag) {
							(
								TagKind::Struct(Some(_), decl_body),
								TagKind::Struct(Some(_), stub_body),
							) if stub_body.is_empty() && !decl_body.is_empty() => {
								*type_kind = Some(entry.data_type.kind.clone());
							}
							(
								TagKind::Union(Some(_), decl_body),
								TagKind::Union(Some(_), stub_body),
							) if stub_body.is_empty() && !decl_body.is_empty() => {
								*type_kind = Some(entry.data_type.kind.clone());
							}
							_ => {
								todo!()
							}
						}
					}
				} else {
					let kind =
						DiagKind::SymbolAlreadyExists(ident.name.clone(), entry.data_type.clone());
					let mut error = Diagnostic::error(kind, entry.span.clone());
					error.push_span(
						span.clone(),
						&format!("`{}` redefined here", ident.name.clone()),
					);
					self.diagnostics.push(error);
				}
			} else {
				*type_kind = Some(tmp_type_kind);
			}
		} else if is_incomplete {
			// error
			todo!()
		} else {
			*type_kind = Some(tmp_type_kind);
		}

		match is_signed {
			Some(true) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(
						SIGNED_STR.to_owned(),
						spec.struct_or_union.kind.to_string(),
					),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			Some(false) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::BothSpecifiers(
						UNSIGNED_STR.to_owned(),
						spec.struct_or_union.kind.to_string(),
					),
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => {
				// do nothing
			}
		}
	}
	fn specifier_enum(
		&mut self,
		spec: &mut syn::EnumSpecifier,
		type_kind: &mut Option<TypeKind>,
		is_signed: Option<bool>,
		long_count: i32,
		in_func: bool,
	) {
		let span = spec.tag_span.clone();
		match type_kind {
			Some(TypeKind::Poison) => {
				// do nothing
			}
			Some(_) => {
				self.diagnostics.push(diag::Diagnostic::error(
					diag::DiagKind::MultipleTypes,
					span.clone(),
				));
				*type_kind = Some(TypeKind::Poison);
			}
			None => *type_kind = Some(TypeKind::Scalar(ScalarType::SInt)),
		}
		if long_count > 0 {
			self.diagnostics.push(diag::Diagnostic::error(
				diag::DiagKind::BothSpecifiers(LONG_STR.to_owned(), CHAR_STR.to_owned()),
				span.clone(),
			));
			*type_kind = Some(TypeKind::Poison);
		}
		let is_incomplete = spec.enumerator_list.is_empty();

		let mut enumerator_list: Vec<(syn::Identifier, i32)> = vec![];
		let mut index: i32 = 0;
		for enumerator in spec.enumerator_list.iter_mut() {
			let enumerator_name = enumerator.enumeration_constant.clone();
			match enumerator.constant_expr.as_mut().map(|v| v.to_i32()) {
				Some(Err(syn::ConversionError::OutOfRange)) => {
					self.diagnostics.push(diag::Diagnostic::error(
						diag::DiagKind::EnumRange,
						enumerator_name.to_span(),
					));
					*type_kind = Some(TypeKind::Poison);
				}
				Some(Err(syn::ConversionError::Expr(_))) => {
					self.diagnostics.push(diag::Diagnostic::error(
						diag::DiagKind::EnumNonIntegral(enumerator_name.name.clone()),
						enumerator_name.to_span(),
					));
					*type_kind = Some(TypeKind::Poison);
				}
				Some(Ok(value)) => {
					enumerator_list.push((enumerator_name.clone(), value));
					index = value;
				}
				None => {
					enumerator_list.push((enumerator_name.clone(), index));
				}
			}
			index += 1;
		}
		let tmp_type_kind: TypeKind;
		tmp_type_kind = TypeKind::Tag(TagKind::Enum(
			spec.identifier.as_ref().map(|id| id.name.clone()),
			enumerator_list,
		));
		*type_kind = Some(tmp_type_kind);
	}
}
