use crate::analysis::sem::Linkage;
use crate::analysis::sem::Namespace;
use crate::analysis::sem::StorageClass;
use crate::analysis::sem::SymbolTableEntry;
use crate::analysis::syn::*;
use crate::analysis::tok;
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeclType {
	Proto,
	FnDef,
	Decl,
}

impl super::SemanticParser<'_> {
	pub(super) fn function_definition(&mut self, decl: &mut FunctionDefinition) -> bool {
		let (maybe_sc, data_type) = self.specifiers(&mut decl.specifiers);

		let (storage, linkage) = match &maybe_sc {
			None | Some(StorageClassSpecifier{kind:StorageClass::Extern,..}) => (StorageClass::Extern, Linkage::External),
			Some(StorageClassSpecifier{kind:StorageClass::Static,..}) => (StorageClass::Static, Linkage::Internal),
			Some(storage) => {
				let kind = diag::DiagKind::IllegalStorage(storage.kind);
				let diag = diag::Diagnostic::error(kind, storage.to_span());
				self.diagnostics.push(diag);
				return false
			},
		};
		let mut ret_type = data_type.unwrap();
		if !matches!(
			decl.declarators.first_mut(),
			None | Some(Declarator::Pointer(_))
		) {
			self.declarator_list(
				decl.ident.span.clone(),
				&mut decl.declarators[1..],
				&mut ret_type,
				false,
				DeclType::FnDef,
				Some(decl.ident.name.clone()),
			);
		}
		match decl.declarators.first_mut() {
			Some(Declarator::IdentList(ident_list)) => {
				let func_type = dtype::FuncType {
					params: vec![],
					ret: Box::new(ret_type),
					is_variadic: false,
					is_inline: !decl.specifiers.inline_list.is_empty(),
				};
				let entry = SymbolTableEntry {
					data_type: dtype::DataType::Function(func_type),
					linkage,
					storage,
					is_incomplete: false,
				};
				let key = Namespace::Ordinary(decl.ident.name.clone());
				self.symtab.insert(key, entry);
			}
			Some(Declarator::ParamList(param_list)) => {
				let Some(mut params) = self.param_list(param_list, DeclType::FnDef) else {
					// failed to get param types
					return false;
				};
				let is_variadic = param_list.is_variadic;
				let func_type = dtype::FuncType {
					params,
					ret: Box::new(ret_type),
					is_variadic,
					is_inline: !decl.specifiers.inline_list.is_empty(),
				};
				let entry = SymbolTableEntry {
					data_type: dtype::DataType::Function(func_type),
					linkage,
					storage,
					is_incomplete: false,
				};
				let key = Namespace::Ordinary(decl.ident.name.clone());
				self.symtab.insert(key, entry);
			}
			Some(Declarator::Array(array)) => {
				let kind = diag::DiagKind::ArrayOfFunctions(decl.ident.name.clone());
				let diag = diag::Diagnostic::error(kind, array.span.clone());
				self.diagnostics.push(diag);
				return false;
			}
			None | Some(Declarator::Pointer(_)) => {
				let kind = diag::DiagKind::UnrecognizedToken {
					expected: vec![
						"\"=\"".to_string(),
						"\",\"".to_string(),
						"\";\"".to_string(),
						"\"asm\"".to_string(),
					],
				};
				let diag = diag::Diagnostic::error(kind, decl.compound_stmt.lcurly.clone());
				self.diagnostics.push(diag);
				return false;
			}
		}
		self.symtab.increase_scope();
		{
			for declaration in decl.declaration_list.iter_mut() {
				self.declaration(declaration, StorageClass::Auto);
			}
			for item in decl.compound_stmt.blocks.iter_mut() {
				self.block_item(item);
			}
		}
		self.decrease_scope();
		return true;
	}

	fn param_list(
		&mut self,
		param_list: &mut ParamList,
		decl_type: DeclType,
	) -> Option<Vec<dtype::DataType>> {
		let param_count = param_list.param_list.len();
		let mut result = vec![];
		let mut is_valid = true;
		for (index, param) in param_list.param_list.iter_mut().enumerate() {
			let (_, data_type) = self.specifiers(&mut param.specifiers);
			let (param_name, param_span): (Option<String>, diag::Span) =
				match (param.name.as_ref(), data_type.unwrap()) {
					(None, dtype::DataType::Void) => {
						let span: diag::Span = match param.declarators.front() {
							Some(Declarator::Array(ArrayDecl { span, .. })) => {
								let kind = diag::DiagKind::ArrayOfVoid(None);
								let diag = diag::Diagnostic::error(kind, span.clone());
								self.diagnostics.push(diag);
								is_valid = false;
								span.clone()
							}
							Some(Declarator::Pointer(_)) => {
								if decl_type == DeclType::FnDef {
									let kind = diag::DiagKind::OmittedParamName;
									let diag = diag::Diagnostic::error(
										kind,
										param.specifiers.first_span.clone(),
									);
									self.diagnostics.push(diag);
									is_valid = false;
								}
								param.specifiers.first_span.clone()
							}
							Some(Declarator::ParamList(_)) => {
								if decl_type == DeclType::FnDef {
									let kind = diag::DiagKind::OmittedParamName;
									let diag = diag::Diagnostic::error(
										kind,
										param.specifiers.first_span.clone(),
									);
									self.diagnostics.push(diag);
									is_valid = false;
								}
								let implicit = Declarator::Pointer(PtrDecl {
									is_const: false,
									is_volatile: false,
									is_restrict: false,
								});
								param.declarators.push_front(implicit);
								param.specifiers.first_span.clone()
							}
							Some(Declarator::IdentList(_)) => {
								let kind = diag::DiagKind::DeclIdentList;
								let diag = diag::Diagnostic::error(
									kind,
									param.specifiers.first_span.clone(),
								);
								self.diagnostics.push(diag);
								is_valid = false;
								param.specifiers.first_span.clone()
							}
							None => {
								if param_count > 1 {
									let kind = diag::DiagKind::OnlyVoid;
									let diag = diag::Diagnostic::error(
										kind,
										param.specifiers.first_span.clone(),
									);
									self.diagnostics.push(diag);
									is_valid = false;
								}
								param.specifiers.first_span.clone()
							}
						};
						(None, span)
					}
					(Some(ident), dtype::DataType::Void) => {
						match param.declarators.front() {
							Some(Declarator::Array(ArrayDecl { span, .. })) => {
								let kind = diag::DiagKind::ArrayOfVoid(Some(ident.name.clone()));
								let diag = diag::Diagnostic::error(kind, ident.span.clone());
								self.diagnostics.push(diag);
								is_valid = false;
							}
							Some(Declarator::IdentList(_)) => {
								let kind = diag::DiagKind::DeclIdentList;
								let diag = diag::Diagnostic::error(kind, ident.span.clone());
								self.diagnostics.push(diag);
								is_valid = false;
							}
							Some(Declarator::ParamList(_)) => {
								let implicit = Declarator::Pointer(PtrDecl {
									is_const: false,
									is_volatile: false,
									is_restrict: false,
								});
								param.declarators.push_front(implicit);
							}
							Some(Declarator::Pointer(_)) => {}
							None => {
								let kind = diag::DiagKind::OnlyVoid;
								let diag = diag::Diagnostic::error(kind, ident.span.clone());
								self.diagnostics.push(diag);
								is_valid = false;
							}
						}
						(Some(ident.name.clone()), ident.span.clone())
					}
					(None, _) => {
						if decl_type == DeclType::FnDef {
							let kind = diag::DiagKind::OmittedParamName;
							let diag =
								diag::Diagnostic::error(kind, param.specifiers.first_span.clone());
							self.diagnostics.push(diag);
							is_valid = false;
						}
						(None, param.specifiers.first_span.clone())
					}
					(Some(ident), _) => (Some(ident.name.clone()), ident.span.clone()),
				};
			let (_, param_type) = self.specifiers(&mut param.specifiers);
			let mut param_type = param_type.unwrap();
			is_valid &= self.declarator_list(
				param_span,
				param.declarators.make_contiguous(),
				&mut param_type,
				true,
				decl_type,
				param_name,
			);
			result.push(param_type)
		}
		match is_valid {
			true => Some(result),
			false => None
		}
	}

	pub(super) fn declaration(&mut self, decl: &mut Declaration, default_sc: StorageClass) -> bool {
		let mut is_valid = true;
		let (maybe_sc, maybe_ty) = self.specifiers(&mut decl.specifiers);
		let (storage, linkage) = match maybe_sc.map(|v| v.kind).unwrap_or(default_sc) {
			StorageClass::Extern => (StorageClass::Extern, Linkage::External),
			StorageClass::Static => (StorageClass::Static, Linkage::Internal),
			storage => (storage, Linkage::None),
		};

		for init_decl in decl.init_declarator_list.iter_mut() {
			let ident = &init_decl.identifier;
			let Some(data_type) = &maybe_ty else {
				let diag = diag::Diagnostic::error(
					diag::DiagKind::ImplicitInt(ident.name.clone()),
					ident.span.clone(),
				);
				self.diagnostics.push(diag);
				continue;
			};
			let mut var_dtype = data_type.clone();
			is_valid &= self.declarator_list(
				ident.span.clone(),
				&mut init_decl.declarator,
				&mut var_dtype,
				false,
				DeclType::Decl,
				Some(ident.name.clone()),
			);
			if !is_valid {
				return false;
			}
			if let Some(ref mut init) = init_decl.initializer {
				self.initializer(init);
			}
			let entry = SymbolTableEntry {
				data_type: var_dtype,
				is_incomplete: false,
				linkage,
				storage,
			};
			let key = Namespace::Ordinary(ident.name.clone());
			self.symtab.insert(key, entry);
		}
		is_valid
	}
	fn specifiers(
		&mut self,
		specifiers: &mut Specifiers,
	) -> (Option<StorageClassSpecifier>, Option<dtype::DataType>) {
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
		}

		let mut is_signed: Option<bool> = None;
		let mut data_type: Option<dtype::DataType> = None;
		let mut long_count = 0;
		for type_spec in specifiers.type_specifiers.iter_mut() {
			match type_spec {
				TypeSpecifier::Void(span) => {
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(dtype::DataType::Void),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								VOID_STR.to_owned(),
							),
							span.clone(),
						));
					}
				}
				TypeSpecifier::Char(span) => {
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(dtype::DataType::Scalar(dtype::ScalarType::I8)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								CHAR_STR.to_owned(),
							),
							span.clone(),
						));
					}
				}
				TypeSpecifier::Short(span) => {
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(dtype::DataType::Scalar(dtype::ScalarType::I16)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								SHORT_STR.to_owned(),
							),
							span.clone(),
						));
					}
				}
				TypeSpecifier::Int(span) => match data_type {
					Some(_) => self.diagnostics.push(diag::Diagnostic::error(
						diag::DiagKind::MultipleTypes,
						span.clone(),
					)),
					None => data_type = Some(dtype::DataType::Scalar(dtype::ScalarType::I32)),
				},
				TypeSpecifier::Long(span) => {
					long_count += 1;
					if long_count > 2 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::TooLong,
							span.clone(),
						));
					}
					match &data_type {
						Some(data_type) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								data_type.to_string(),
							),
							span.clone(),
						)),
						None | Some(dtype::DataType::Scalar(dtype::ScalarType::I32)) => {
							// do nothing
						}
					}
				}
				TypeSpecifier::Float(span) => {
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								FLOAT_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								FLOAT_STR.to_owned(),
							),
							span.clone(),
						)),
						None => {
							// do nothing
						}
					}
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(dtype::DataType::Scalar(dtype::ScalarType::Float)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								FLOAT_STR.to_owned(),
							),
							span.clone(),
						));
					}
				}
				TypeSpecifier::Double(span) => {
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								DOUBLE_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								DOUBLE_STR.to_owned(),
							),
							span.clone(),
						)),
						None => {
							// do nothing
						}
					}
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => {
							data_type = Some(dtype::DataType::Scalar(dtype::ScalarType::Double))
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
					}
				}
				TypeSpecifier::Signed(span) => {
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::DuplicateSpecifier(SIGNED_STR.to_owned()),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								UNSIGNED_STR.to_owned(),
							),
							span.clone(),
						)),
						None => is_signed = Some(true),
					}
					match &data_type {
						Some(dtype::DataType::Scalar(dtype::ScalarType::Double)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									DOUBLE_STR.to_owned(),
								),
								span.clone(),
							))
						}
						Some(dtype::DataType::Scalar(dtype::ScalarType::Float)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									FLOAT_STR.to_owned(),
								),
								span.clone(),
							))
						}
						Some(name) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(SIGNED_STR.to_owned(), name.to_string()),
							span.clone(),
						)),
						Some(dtype::DataType::Scalar(_)) | None => {
							// do nothing
						}
					}
				}
				TypeSpecifier::Unsigned(span) => {
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								UNSIGNED_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::DuplicateSpecifier(UNSIGNED_STR.to_owned()),
							span.clone(),
						)),
						None => is_signed = Some(false),
					}
					match &data_type {
						Some(dtype::DataType::Scalar(dtype::ScalarType::Double)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									DOUBLE_STR.to_owned(),
								),
								span.clone(),
							))
						}
						Some(dtype::DataType::Scalar(dtype::ScalarType::Float)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									FLOAT_STR.to_owned(),
								),
								span.clone(),
							))
						}
						Some(name) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								name.to_string(),
							),
							span.clone(),
						)),
						Some(dtype::DataType::Scalar(_)) | None => {
							// do nothing
						}
					}
				}
				TypeSpecifier::Bool(span) => {
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(dtype::DataType::Scalar(dtype::ScalarType::Bool)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								BOOL_STR.to_owned(),
							),
							span.clone(),
						));
					}
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								BOOL_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								BOOL_STR.to_owned(),
							),
							span.clone(),
						)),
						None => {
							// do nothing
						}
					}
				}
				TypeSpecifier::StructOrUnionSpecifier(StructOrUnionSpecifier {
					struct_or_union,
					struct_declaration_list,
					..
				}) => {
					let span = struct_or_union.span.clone();
					let mut members = vec![];
					for decl in struct_declaration_list.iter_mut() {
						let mut member_vec = self.struct_declaration(decl);
						members.append(&mut member_vec);
					}
					if data_type.is_some() {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						));
					} else if let tok::Keyword::Struct = struct_or_union.keyword {
						data_type = Some(dtype::DataType::Struct(members));
					} else if let tok::Keyword::Union = struct_or_union.keyword {
						data_type = Some(dtype::DataType::Union(members));
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								CHAR_STR.to_owned(),
							),
							span.clone(),
						));
					}
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								STRUCT_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								STRUCT_STR.to_owned(),
							),
							span.clone(),
						)),
						None => {
							// do nothing
						}
					}
				}
				TypeSpecifier::EnumSpecifier(EnumSpecifier { tag_span, .. }) => {
					let span = tag_span.clone();
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(dtype::DataType::Scalar(dtype::ScalarType::I8)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								CHAR_STR.to_owned(),
							),
							span.clone(),
						));
					}
				}
				TypeSpecifier::TypedefName { .. } => todo!("typedef"),
			}
		}
		if let Some(dtype::DataType::Scalar(ref mut scalar)) = &mut data_type {
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
		(storage_class, data_type)
	}
	fn init_declarator(&mut self, decl: &mut InitDeclarator) {
		let _ = decl.identifier;
		let _ = decl.declarator;
		if let Some(ref mut init) = decl.initializer {
			self.initializer(init);
		}
	}
	fn enum_specifier(&mut self, _spec: &mut EnumSpecifier) {
		todo!("enum-specifier")
	}
	fn enumerator(&mut self, enumerator: &mut Enumerator) {
		if let Some(ref mut expr) = enumerator.constant_expr {
			self.expr(expr);
		}
	}

	// FIXME: symbol table infrastructure required to parse this AST.
	fn struct_declaration(
		&mut self,
		struct_decl: &mut StructDeclaration,
	) -> Vec<dtype::MemberType> {
		// let mut result = vec![];
		// // only type-specifier and type-qualifier is syntactically allowed here.
		// let (_, ty_opt) = self.specifiers(&mut struct_decl.specifiers);
		// for decl in struct_decl.struct_declaration_list.iter_mut() {
		// 	//self.struct_declarator(decl);
		// 	let ident = decl.identifier.and_then(|v| Some(v.name));
		// 	result.push(dtype::MemberType { ident, dtype: () });
		// }
		// result
		todo!("struct-decl")
	}

	fn struct_declarator(&mut self, struct_decl: &mut StructDeclarator) {
		// if let Some(ref mut decl) = struct_decl.declarator {
		// 	self.declarator(decl)
		// }
		if let Some(ref mut expr) = struct_decl.const_expr {
			self.expr(expr);
		}
	}
	fn struct_or_union_specifier(&mut self, _spec: &mut StructOrUnionSpecifier) {
		todo!("struct-or-union-specifier")
	}
	fn initializer(&mut self, init: &mut Initializer) {
		use Initializer::*;
		match init {
			Expr(expr) => self.expr(expr),
			InitializerList(list) => self.initializer_list(list),
		}
	}

	fn declarator_list(
		&mut self,
		span: diag::Span,
		decl_list: &mut [Declarator],
		data_type: &mut dtype::DataType,
		mut is_param: bool,
		mut decl_type: DeclType,
		name: Option<String>,
	) -> bool {
		let mut last_is_ptr = decl_type != DeclType::FnDef;
		// first iteration is for type checking
		for declarator in decl_list.iter() {
			match declarator {
				Declarator::Array(array) => {
					if array.has_star && decl_type != DeclType::Proto {
						if is_param && decl_type == DeclType::FnDef {
							let kind = diag::DiagKind::UnboundVLA;
							let diag = diag::Diagnostic::error(kind, span.clone());
							self.diagnostics.push(diag);
						} else if !is_param {
							let kind = diag::DiagKind::InvalidStar;
							let diag = diag::Diagnostic::error(kind, array.span.clone());
							self.diagnostics.push(diag);
						}
					}
					last_is_ptr = false;
				}
				Declarator::Pointer(pointer) => {
					last_is_ptr = true;
				}
				Declarator::IdentList(_) => {
					if !last_is_ptr {
						let kind = diag::DiagKind::FnRetFn(name.clone());
						let diag = diag::Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
					}
					last_is_ptr = false;
				}
				Declarator::ParamList(type_list) => {
					if !last_is_ptr {
						let kind = diag::DiagKind::FnRetFn(name.clone());
						let diag = diag::Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
					}
					last_is_ptr = false;
				}
			};
		}
		// reversed iterator because recursive type construction has
		// data type at the end
		for declarator in decl_list.iter_mut().rev() {
			*data_type = match declarator {
				Declarator::Array(array) => {
					let array_type: dtype::ArrayType = if !is_param {
						let length = if let Some(assign_expr) = &mut array.assignment_expr {
							match assign_expr.to_u32() {
								Ok(val) => dtype::ArrayLength::Fixed(val),
								Err(ConversionError::OutOfRange) => todo!("error"),
								Err(ConversionError::Expr(_)) => dtype::ArrayLength::Variable,
							}
						} else {
							//TODO
							continue;
						};

						dtype::ArrayType {
							component: Box::new(data_type.clone()),
							length,
						}
					} else {
						//TODO
						continue;
					};

					dtype::DataType::Array(array_type)
				}
				Declarator::Pointer(pointer) => {
					let ptr_type = dtype::PtrType {
						inner: Box::new(data_type.clone()),
						is_const: pointer.is_const,
						is_restrict: pointer.is_restrict,
						is_volatile: pointer.is_volatile,
					};
					dtype::DataType::Pointer(ptr_type)
				}
				Declarator::IdentList(ident_list) => {
					let kind = diag::DiagKind::DeclIdentList;
					let diag = diag::Diagnostic::error(kind, ident_list.span.clone());
					self.diagnostics.push(diag);
					let func_type = dtype::FuncType {
						params: vec![],
						ret: Box::new(data_type.clone()),
						is_variadic: false,
						is_inline: false,
					};
					dtype::DataType::Function(func_type)
				}
				Declarator::ParamList(type_list) => {
					if DeclType::FnDef == decl_type && is_param {
						decl_type = DeclType::Proto;
					}
					let Some(mut params) = self.param_list(type_list, decl_type) else {
						return false;
					};

					let func_type = dtype::FuncType {
						params,
						ret: Box::new(data_type.clone()),
						is_variadic: type_list.is_variadic,
						is_inline: false,
					};
					dtype::DataType::Function(func_type)
				}
			};
		}
		return true;
	}
	fn type_qualifier(&mut self, qual: &mut TypeQualifier) {
		match qual.kind {
			TypeQualifierKind::Const => (),
			TypeQualifierKind::Restrict => (),
			TypeQualifierKind::Volatile => (),
		}
	}
	fn initializer_list(&mut self, list: &mut InitializerList) {
		for (desig, ref mut init) in list.0.iter_mut() {
			if let Some(ref mut desig) = desig {
				self.designation(desig);
			}
			self.initializer(init);
		}
	}
	fn designation(&mut self, desig: &mut Designation) {
		for ref mut desig in desig.0.iter_mut() {
			self.designator(desig)
		}
	}
	fn designator(&mut self, desig: &mut Designator) {
		use Designator::*;
		match desig {
			ConstantExpr(expr) => self.expr(expr),
			Dot(_) => (),
		}
	}
}
