use crate::analysis::sem::Linkage;
use crate::analysis::sem::Namespace;
use crate::analysis::sem::StorageClass;
use crate::analysis::sem::SymbolTableEntry;
use crate::analysis::syn::*;
use crate::analysis::tok;
use crate::data_types as dtype;
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
	pub(super) fn function_definition(&mut self, decl: &mut FunctionDefinition) {
		let (storage, data_type) = self.specifiers(&mut decl.specifiers);

		let storage = storage.unwrap_or(StorageClass::Extern);
		let linkage = match storage {
			StorageClass::Extern => Linkage::External,
			StorageClass::Static => Linkage::Internal,
			_ => todo!("invalid storage class"),
		};
		let mut ret_type = data_type.unwrap();
		self.declarator_list(&mut decl.declarators[1..], &mut ret_type, false);
		match decl.declarators.first_mut() {
			Some(Declarator::IdentList(ident_list)) => {
				let func_type = dtype::FuncType {
					params: vec![],
					ret: Box::new(ret_type),
					is_variadic: false,
				};
				let entry = SymbolTableEntry {
					data_type: dtype::DataType::Function(func_type),
					linkage,
					storage,
					is_incomplete: false,
				};
				let key = Namespace::Ordinary(decl.identifier.name.clone());
				self.symtab.insert(key, entry);
			}
			Some(Declarator::ParamList(param_list)) => {
				let mut params = vec![];
				for param in param_list.param_list.iter_mut() {
					let (_, param_type) = self.specifiers(&mut param.specifiers);
					let mut param_type = param_type.unwrap();
					self.declarator_list(&mut param.declarators, &mut param_type, true);
					params.push(param_type)
				}
				let is_variadic = param_list.is_variadic;
				let func_type = dtype::FuncType {
					params,
					ret: Box::new(ret_type),
					is_variadic,
				};
				let entry = SymbolTableEntry {
					data_type: dtype::DataType::Function(func_type),
					linkage,
					storage,
					is_incomplete: false,
				};
				let key = Namespace::Ordinary(decl.identifier.name.clone());
				self.symtab.insert(key, entry);
			}
			Some(Declarator::Array(array)) => {
				let kind = diag::DiagKind::ArrayOfFunctions(decl.identifier.name.clone());
				let diag = diag::Diagnostic::error(kind, array.span.clone());
				self.diagnostics.push(diag);
				return;
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
				return;
			}
		}
		self.symtab.increase_scope();
		{
			for declaration in decl.declaration_list.iter_mut() {
				self.declaration(declaration, StorageClass::Auto);
			}
			for item in decl.compound_stmt.blocks.iter_mut() {
				self.block_item(item)
			}
		}
		self.decrease_scope();
	}
	pub(super) fn declaration(&mut self, decl: &mut Declaration, default_sc: StorageClass) {
		let (maybe_sc, maybe_ty) = self.specifiers(&mut decl.specifiers);
		let storage = maybe_sc.unwrap_or(default_sc);
		let linkage = match default_sc {
			StorageClass::Auto | StorageClass::Register | StorageClass::Typedef => Linkage::None,
			StorageClass::Extern => Linkage::External,
			StorageClass::Static => Linkage::Internal,
		};

		for ref mut init_decl in decl.init_declarator_list.iter_mut() {
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
			self.declarator_list(&mut init_decl.declarator, &mut var_dtype, false);
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
	}
	fn specifiers(
		&mut self,
		specifiers: &mut Specifiers,
	) -> (Option<StorageClass>, Option<dtype::DataType>) {
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
				storage_class = Some(storage_class_specifier.storage_class);
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
		// only type-specifier and type-qualifier is syntactically allowed here.
		// let (_, ty_opt) = self.specifiers(&mut struct_decl.specifiers);
		// for decl in struct_decl.struct_declaration_list.iter_mut() {
		// 	//self.struct_declarator(decl);
		// 	let ident = decl.identifier.and_then(|v| Some(v.name));
		// 	result.push(dtype::MemberType { ident, dtype: () });
		// }
		// result

		todo!("struct-declarator")
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
		decl_list: &mut [Declarator],
		data_type: &mut dtype::DataType,
		is_param: bool,
	) {
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
								Err(ConversionError::Expr(_expr)) => dtype::ArrayLength::Variable,
							}
						} else {
							todo!()
						};

						dtype::ArrayType {
							component: Box::new(data_type.clone()),
							length,
						}
					} else {
						todo!()
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
				Declarator::IdentList(_) => {
					todo!("function identifier list")
				}
				Declarator::ParamList(type_list) => {
					let mut params = vec![];
					for param in type_list.param_list.iter_mut() {
						let (_, maybe_type) = self.specifiers(&mut param.specifiers);
						let mut param_type = maybe_type.unwrap();
						self.declarator_list(&mut param.declarators, &mut param_type, true);
						params.push(param_type);
					}

					let func_type = dtype::FuncType {
						params,
						ret: Box::new(data_type.clone()),
						is_variadic: type_list.is_variadic,
					};
					dtype::DataType::Function(func_type)
				}
			};
		}
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
