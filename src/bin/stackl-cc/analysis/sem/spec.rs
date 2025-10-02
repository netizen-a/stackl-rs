use crate::analysis::syn;
use crate::data_types as dtype;
use crate::diagnostics::ToSpan;
use crate::diagnostics as diag;

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
    pub(super) fn specifiers_storage(&mut self, specifiers: &mut syn::Specifiers) -> Option<syn::StorageClassSpecifier> {
		let mut storage_class = None;
		let mut is_valid = true;
		for (i, storage_class_specifier) in specifiers.storage_classes.iter().enumerate() {
			if i > 0 {
				let diag = diag::Diagnostic::error(
					diag::DiagKind::MultStorageClasses,
					storage_class_specifier.span.clone(),
				);
				self.diagnostics.push(diag);
				storage_class = None;
				is_valid = false;
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
			is_valid = false;
		}
		if is_valid {
			storage_class
		} else {
			None
		}
	}
    pub(super) fn specifiers_dtype(&mut self, specifiers: &mut syn::Specifiers) -> Result<dtype::DataType, bool> {
		let mut is_valid = true;
		let mut type_kind: Option<dtype::TypeKind> = None;

		let mut is_signed: Option<bool> = None;
		let mut long_count = 0;
		for type_spec in specifiers.type_specifiers.iter_mut() {
			match type_spec {
				syn::TypeSpecifier::Void(span) => {
					match type_kind {
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							is_valid = false;
						}
						None => type_kind = Some(dtype::TypeKind::Void),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								VOID_STR.to_owned(),
							),
							span.clone(),
						));
						is_valid = false;
					}
				}
				syn::TypeSpecifier::Char(span) => {
					match type_kind {
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							is_valid = false;
						}
						None => type_kind = Some(dtype::TypeKind::Scalar(dtype::ScalarType::I8)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								CHAR_STR.to_owned(),
							),
							span.clone(),
						));
						is_valid = false;
					}
				}
				syn::TypeSpecifier::Short(span) => {
					match type_kind {
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							is_valid = false;
						}
						None => type_kind = Some(dtype::TypeKind::Scalar(dtype::ScalarType::I16)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								SHORT_STR.to_owned(),
							),
							span.clone(),
						));
						is_valid = false;
					}
				}
				syn::TypeSpecifier::Int(span) => match type_kind {
					Some(_) => {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						));
						is_valid = false;
					}
					None => type_kind = Some(dtype::TypeKind::Scalar(dtype::ScalarType::I32)),
				},
				syn::TypeSpecifier::Long(span) => {
					long_count += 1;
					if long_count > 2 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::TooLong,
							span.clone(),
						));
						is_valid = false;
					}
					match &type_kind {
						Some(
							dtype::TypeKind::Struct(_)
							| dtype::TypeKind::Union(_)
							| dtype::TypeKind::Enum,
						) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							is_valid = false;
						}
						Some(type_kind) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									LONG_STR.to_owned(),
									type_kind.to_string(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						None | Some(dtype::TypeKind::Scalar(dtype::ScalarType::I32)) => {
							// do nothing
						}
					}
				}
				syn::TypeSpecifier::Float(span) => {
					match is_signed {
						Some(true) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									FLOAT_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									FLOAT_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						None => {
							// do nothing
						}
					}
					match type_kind {
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							is_valid = false;
						}
						None => type_kind = Some(dtype::TypeKind::Scalar(dtype::ScalarType::Float)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								FLOAT_STR.to_owned(),
							),
							span.clone(),
						));
						is_valid = false;
					}
				}
				syn::TypeSpecifier::Double(span) => {
					match is_signed {
						Some(true) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									DOUBLE_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									DOUBLE_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						None => {
							// do nothing
						}
					}
					match type_kind {
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							is_valid = false;
						}
						None => {
							type_kind = Some(dtype::TypeKind::Scalar(dtype::ScalarType::Double))
						}
					}
					if long_count > 1 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_LONG_STR.to_owned(),
								DOUBLE_STR.to_owned(),
							),
							span.clone(),
						));
						is_valid = false;
					}
				}
				syn::TypeSpecifier::Signed(span) => {
					match is_signed {
						Some(true) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::DuplicateSpecifier(SIGNED_STR.to_owned()),
								span.clone(),
							));
							is_valid = false;
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									UNSIGNED_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						None => is_signed = Some(true),
					}
					match &type_kind {
						Some(dtype::TypeKind::Scalar(dtype::ScalarType::Double)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									DOUBLE_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						Some(dtype::TypeKind::Scalar(dtype::ScalarType::Float)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									FLOAT_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						Some(_) => {
							let expected = vec![
								"identifier".to_string(),
								"\";\"".to_string(),
								"\"(\"".to_string(),
							];
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::UnrecognizedToken { expected },
								span.clone(),
							));
							is_valid = false;
						}
						Some(dtype::TypeKind::Scalar(_)) | None => {
							// do nothing
						}
					}
				}
				syn::TypeSpecifier::Unsigned(span) => {
					match is_signed {
						Some(true) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									UNSIGNED_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::DuplicateSpecifier(UNSIGNED_STR.to_owned()),
								span.clone(),
							));
							is_valid = false;
						}
						None => is_signed = Some(false),
					}
					match &type_kind {
						Some(dtype::TypeKind::Scalar(dtype::ScalarType::Double)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									DOUBLE_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						Some(dtype::TypeKind::Scalar(dtype::ScalarType::Float)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									FLOAT_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						Some(_) => {
							let expected = vec![
								"identifier".to_string(),
								"\";\"".to_string(),
								"\"(\"".to_string(),
							];
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::UnrecognizedToken { expected },
								span.clone(),
							));
							is_valid = false;
						}
						Some(dtype::TypeKind::Scalar(_)) | None => {
							// do nothing
						}
					}
				}
				syn::TypeSpecifier::Bool(span) => {
					match type_kind {
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							is_valid = false;
						}
						None => type_kind = Some(dtype::TypeKind::Scalar(dtype::ScalarType::Bool)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								BOOL_STR.to_owned(),
							),
							span.clone(),
						));
						is_valid = false;
					}
					match is_signed {
						Some(true) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									BOOL_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									BOOL_STR.to_owned(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						None => {
							// do nothing
						}
					}
				}
				syn::TypeSpecifier::StructOrUnionSpecifier(syn::StructOrUnionSpecifier {
					struct_or_union,
					struct_declaration_list,
					is_incomplete,
					..
				}) => {
					let span = struct_or_union.span.clone();
					let mut members = vec![];
					for decl in struct_declaration_list.iter_mut() {
						let mut member_vec = self.struct_declaration(decl);
						let Some(mut member_vec) = member_vec else {
							is_valid = false;
							continue;
						};
						members.append(&mut member_vec);
					}
					match struct_or_union.kind {
						syn::StructOrUnionKind::Struct => {
							let struct_type = dtype::StructType {
								members,
								is_incomplete: *is_incomplete,
							};
							type_kind = Some(dtype::TypeKind::Struct(struct_type));
						}
						syn::StructOrUnionKind::Union => {
							let union_type = dtype::UnionType {
								members,
								is_incomplete: *is_incomplete,
							};
							type_kind = Some(dtype::TypeKind::Union(union_type));
						}
					}

					if type_kind.is_some() || long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						));
						is_valid = false;
					}
					match is_signed {
						Some(true) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									struct_or_union.kind.to_string(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									struct_or_union.kind.to_string(),
								),
								span.clone(),
							));
							is_valid = false;
						}
						None => {
							// do nothing
						}
					}
				}
				syn::TypeSpecifier::EnumSpecifier(syn::EnumSpecifier {
					tag_span,
					enumerator_list,
					..
				}) => {
					let span = tag_span.clone();
					match type_kind {
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							is_valid = false;
						}
						None => type_kind = Some(dtype::TypeKind::Scalar(dtype::ScalarType::I32)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								CHAR_STR.to_owned(),
							),
							span.clone(),
						));
						is_valid = false;
					}
					for enumerator in enumerator_list {
						match enumerator.constant_expr.as_mut().map(|v| v.to_i32()) {
							Some(Err(syn::ConversionError::OutOfRange)) => {
								self.diagnostics.push(diag::Diagnostic::error(
									diag::DiagKind::EnumRange,
									enumerator.enumeration_constant.to_span(),
								));
								is_valid = false;
							}
							Some(Err(syn::ConversionError::Expr(_))) => {
								self.diagnostics.push(diag::Diagnostic::error(
									diag::DiagKind::EnumNonIntegral(
										enumerator.enumeration_constant.name.clone(),
									),
									enumerator.enumeration_constant.to_span(),
								));
								is_valid = false;
							}
							_ => {
								// do nothing
							}
						}
					}
				}
				syn::TypeSpecifier::TypedefName { .. } => todo!("typedef"),
			}
		}
		if let Some(dtype::TypeKind::Scalar(ref mut scalar)) = &mut type_kind {
			if let dtype::ScalarType::I32 = scalar {
				match long_count {
					1 => *scalar = dtype::ScalarType::I64,
					2 => *scalar = dtype::ScalarType::I128,
					_ => {}
				}
			}
			if let Some(is_signed) = is_signed {
				scalar.set_signedness(is_signed);
			}
		}

		let data_type = if let (Some(type_kind), true) = (type_kind, is_valid) {
			let type_qual = dtype::TypeQual {
				is_const: specifiers.is_const,
				is_restrict: !specifiers.restrict_list.is_empty(),
				is_volatile: specifiers.is_volatile,
			};
			Some(dtype::DataType {
				kind: type_kind,
				qual: type_qual,
			})
		} else {
			None
		};

		match (data_type, is_valid) {
			(Some(data_type), true) => Ok(data_type),
			(maybe, false) => Err(maybe.is_some()),
			(None, true) => Err(false),
		}
	}
}
