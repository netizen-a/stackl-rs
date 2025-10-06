use crate::analysis::syn;
use crate::data_types::*;
use crate::diagnostics as diag;
use crate::diagnostics::ToSpan;

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

impl super::SemanticParser {
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
	) -> Option<DataType> {
		let mut type_kind: Option<TypeKind> = None;

		let mut is_signed: Option<bool> = None;
		let mut long_count = 0;
		for type_spec in specifiers.type_specifiers.iter_mut() {
			match type_spec {
				syn::TypeSpecifier::Void(span) => {
					match type_kind {
						None => type_kind = Some(TypeKind::Void),
						Some(TypeKind::Poison) => {
							// do nothing
						}
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
						}
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								VOID_STR.to_owned(),
							),
							span.clone(),
						));
						type_kind = Some(TypeKind::Poison);
					}
				}
				syn::TypeSpecifier::Char(span) => {
					match type_kind {
						Some(TypeKind::Poison) => {
							// do nothing
						}
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
						}
						None => type_kind = Some(TypeKind::Scalar(ScalarType::I8)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								CHAR_STR.to_owned(),
							),
							span.clone(),
						));
						type_kind = Some(TypeKind::Poison);
					}
				}
				syn::TypeSpecifier::Short(span) => {
					match type_kind {
						Some(TypeKind::Poison) => {
							// do nothing
						}
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
						}
						None => type_kind = Some(TypeKind::Scalar(ScalarType::I16)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								SHORT_STR.to_owned(),
							),
							span.clone(),
						));
						type_kind = Some(TypeKind::Poison);
					}
				}
				syn::TypeSpecifier::Int(span) => match type_kind {
					Some(TypeKind::Poison) => {
						// do nothing
					}
					Some(_) => {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						));
						type_kind = Some(TypeKind::Poison);
					}
					None => type_kind = Some(TypeKind::Scalar(ScalarType::I32)),
				},
				syn::TypeSpecifier::Long(span) => {
					long_count += 1;
					if long_count > 2 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::TooLong,
							span.clone(),
						));
						type_kind = Some(TypeKind::Poison);
					}
					match &mut type_kind {
						Some(TypeKind::Struct(_) | TypeKind::Union(_) | TypeKind::Enum(_)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
						}
						Some(type_kind) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									LONG_STR.to_owned(),
									type_kind.to_string(),
								),
								span.clone(),
							));
							*type_kind = TypeKind::Poison;
						}
						None | Some(TypeKind::Scalar(ScalarType::I32)) => {
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
							type_kind = Some(TypeKind::Poison);
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									FLOAT_STR.to_owned(),
								),
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
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
							type_kind = Some(TypeKind::Poison);
						}
						None => type_kind = Some(TypeKind::Scalar(ScalarType::Float)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								FLOAT_STR.to_owned(),
							),
							span.clone(),
						));
						type_kind = Some(TypeKind::Poison);
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
							type_kind = Some(TypeKind::Poison);
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									DOUBLE_STR.to_owned(),
								),
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
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
							type_kind = Some(TypeKind::Poison);
						}
						None => type_kind = Some(TypeKind::Scalar(ScalarType::Double)),
					}
					if long_count > 1 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_LONG_STR.to_owned(),
								DOUBLE_STR.to_owned(),
							),
							span.clone(),
						));
						type_kind = Some(TypeKind::Poison);
					}
				}
				syn::TypeSpecifier::Signed(span) => {
					match is_signed {
						Some(true) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::DuplicateSpecifier(SIGNED_STR.to_owned()),
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									UNSIGNED_STR.to_owned(),
								),
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
						}
						None => is_signed = Some(true),
					}
					match &type_kind {
						Some(TypeKind::Scalar(ScalarType::Double)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									DOUBLE_STR.to_owned(),
								),
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
						}
						Some(TypeKind::Scalar(ScalarType::Float)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									FLOAT_STR.to_owned(),
								),
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
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
							type_kind = Some(TypeKind::Poison);
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
							type_kind = Some(TypeKind::Poison);
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::DuplicateSpecifier(UNSIGNED_STR.to_owned()),
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
						}
						None => is_signed = Some(false),
					}
					match &type_kind {
						Some(TypeKind::Scalar(ScalarType::Double)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									DOUBLE_STR.to_owned(),
								),
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
						}
						Some(TypeKind::Scalar(ScalarType::Float)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									FLOAT_STR.to_owned(),
								),
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
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
							type_kind = Some(TypeKind::Poison);
						}
					}
				}
				syn::TypeSpecifier::Bool(span) => {
					match type_kind {
						Some(TypeKind::Poison) => {
							// do nothing
						}
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
						}
						None => type_kind = Some(TypeKind::Scalar(ScalarType::Bool)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								BOOL_STR.to_owned(),
							),
							span.clone(),
						));
						type_kind = Some(TypeKind::Poison);
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
							type_kind = Some(TypeKind::Poison);
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									BOOL_STR.to_owned(),
								),
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
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
					ident,
					..
				}) => {
					let span = match ident {
						Some(ident) => ident.to_span(),
						None => struct_or_union.span.clone(),
					};
					if type_kind.is_some() || long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						));
						type_kind = Some(TypeKind::Poison);
						break;
					}
					let mut members = vec![];
					let mut member_is_named = false;
					for decl in struct_declaration_list.iter_mut() {
						let member_vec = self.struct_declaration(decl, &mut member_is_named);
						let Some(mut member_vec) = member_vec else {
							type_kind = Some(TypeKind::Poison);
							continue;
						};
						members.append(&mut member_vec);
					}
					if !member_is_named {
						let error = diag::Diagnostic::error(
							diag::DiagKind::StructNoNamedMembers,
							span.clone(),
						);
						self.diagnostics.push(error);
					}
					match struct_or_union.kind {
						syn::StructOrUnionKind::Struct => {
							let struct_type = StructType {
								name: ident.clone().map(|v| v.name),
								members,
								is_incomplete: *is_incomplete,
							};
							type_kind = Some(TypeKind::Struct(struct_type));
						}
						syn::StructOrUnionKind::Union => {
							let union_type = UnionType {
								name: ident.clone().map(|v| v.name),
								members,
								is_incomplete: *is_incomplete,
							};
							type_kind = Some(TypeKind::Union(union_type));
						}
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
							type_kind = Some(TypeKind::Poison);
						}
						Some(false) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									struct_or_union.kind.to_string(),
								),
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
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
						Some(TypeKind::Poison) => {
							// do nothing
						}
						Some(_) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::MultipleTypes,
								span.clone(),
							));
							type_kind = Some(TypeKind::Poison);
						}
						None => type_kind = Some(TypeKind::Scalar(ScalarType::I32)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								CHAR_STR.to_owned(),
							),
							span.clone(),
						));
						type_kind = Some(TypeKind::Poison);
					}
					for enumerator in enumerator_list {
						match enumerator.constant_expr.as_mut().map(|v| v.to_i32()) {
							Some(Err(syn::ConversionError::OutOfRange)) => {
								self.diagnostics.push(diag::Diagnostic::error(
									diag::DiagKind::EnumRange,
									enumerator.enumeration_constant.to_span(),
								));
								type_kind = Some(TypeKind::Poison);
							}
							Some(Err(syn::ConversionError::Expr(_))) => {
								self.diagnostics.push(diag::Diagnostic::error(
									diag::DiagKind::EnumNonIntegral(
										enumerator.enumeration_constant.name.clone(),
									),
									enumerator.enumeration_constant.to_span(),
								));
								type_kind = Some(TypeKind::Poison);
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

		if type_kind.is_none() {
			match long_count {
				1 => type_kind = Some(TypeKind::Scalar(ScalarType::I64)),
				2 => type_kind = Some(TypeKind::Scalar(ScalarType::I128)),
				_ => {}
			}
		}

		if let Some(TypeKind::Scalar(ref mut scalar)) = &mut type_kind {
			if let ScalarType::I32 = scalar {
				match long_count {
					1 => *scalar = ScalarType::I64,
					2 => *scalar = ScalarType::I128,
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
			Some(DataType {
				kind: type_kind,
				qual: type_qual,
			})
		} else {
			None
		}
	}
}
