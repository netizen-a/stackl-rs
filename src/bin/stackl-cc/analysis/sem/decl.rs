use crate::analysis::sem::Linkage;
use crate::analysis::sem::Namespace;
use crate::analysis::sem::StorageClass;
use crate::analysis::sem::SymbolTableEntry;
use crate::analysis::syn;
use crate::analysis::syn::*;
use crate::analysis::tok;
use crate::data_types as dtype;
use crate::diagnostics as diag;
use crate::diagnostics::ToSpan;
use crate::WarnLevel;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeclType {
	Proto,
	FnDef,
	Decl,
}

impl super::SemanticParser<'_> {
	pub(super) fn function_definition(&mut self, decl: &mut FunctionDefinition) -> bool {
		let data_type = self.specifiers_dtype(&mut decl.specifiers);
		let maybe_sc = self.specifiers_storage(&mut decl.specifiers);

		let (storage, linkage) = match &maybe_sc {
			None
			| Some(StorageClassSpecifier {
				kind: StorageClass::Extern,
				..
			}) => (StorageClass::Extern, Linkage::External),
			Some(StorageClassSpecifier {
				kind: StorageClass::Static,
				..
			}) => (StorageClass::Static, Linkage::Internal),
			Some(storage) => {
				let kind = diag::DiagKind::IllegalStorage(storage.kind);
				let diag = diag::Diagnostic::error(kind, storage.to_span());
				self.diagnostics.push(diag);
				return false;
			}
		};
		let mut ret_type = data_type.unwrap();
		if !matches!(
			decl.declarators.first_mut(),
			None | Some(Declarator::Pointer(_))
		) {
			self.declarator_list(
				decl.ident.to_span(),
				&mut decl.declarators[1..],
				&mut ret_type,
				false,
				DeclType::FnDef,
				Some(decl.ident.name.clone()),
				None,
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
					data_type: dtype::DataType {
						kind: dtype::TypeKind::Function(func_type),
						qual: dtype::TypeQual::default(),
					},
					linkage,
					storage,
				};
				let key = Namespace::Ordinary(decl.ident.name.clone());
				self.symtab.insert(key, entry);
			}
			Some(Declarator::ParamList(param_list)) => {
				if param_list.param_list.len() > 127 && self.warn_lvl == WarnLevel::All {
					// 5.2.4.1 translation limit
					let diag = diag::Diagnostic::warn(
						diag::DiagKind::ParameterLimit,
						decl.ident.to_span(),
					);
					self.diagnostics.push(diag);
				}
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
				// TODO: qualifiers
				let entry = SymbolTableEntry {
					data_type: dtype::DataType {
						kind: dtype::TypeKind::Function(func_type),
						qual: dtype::TypeQual::default(),
					},
					linkage,
					storage,
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
			let data_type = self.specifiers_dtype(&mut param.specifiers).unwrap();
			let (param_name, param_span): (Option<String>, diag::Span) =
				match (param.name.as_ref(), data_type.kind) {
					(None, dtype::TypeKind::Void) => {
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
					(Some(ident), dtype::TypeKind::Void) => {
						match param.declarators.front() {
							Some(Declarator::Array(ArrayDecl { span, .. })) => {
								let kind = diag::DiagKind::ArrayOfVoid(Some(ident.name.clone()));
								let diag = diag::Diagnostic::error(kind, ident.to_span());
								self.diagnostics.push(diag);
								is_valid = false;
							}
							Some(Declarator::IdentList(_)) => {
								let kind = diag::DiagKind::DeclIdentList;
								let diag = diag::Diagnostic::error(kind, ident.to_span());
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
								let diag = diag::Diagnostic::error(kind, ident.to_span());
								self.diagnostics.push(diag);
								is_valid = false;
							}
						}
						(Some(ident.name.clone()), ident.to_span())
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
					(Some(ident), _) => (Some(ident.name.clone()), ident.to_span()),
				};
			let param_type = self.specifiers_dtype(&mut param.specifiers);
			let mut param_type = param_type.unwrap();
			is_valid &= self.declarator_list(
				param_span,
				param.declarators.make_contiguous(),
				&mut param_type,
				true,
				decl_type,
				param_name,
				None,
			);
			result.push(param_type)
		}
		match is_valid {
			true => Some(result),
			false => None,
		}
	}

	pub(super) fn declaration(&mut self, decl: &mut Declaration, default_sc: StorageClass) -> bool {
		let mut is_valid = true;
		let maybe_ty = self.specifiers_dtype(&mut decl.specifiers);
		let maybe_sc = self.specifiers_storage(&mut decl.specifiers);
		let (storage, linkage) = match maybe_sc.map(|v| v.kind).unwrap_or(default_sc) {
			StorageClass::Extern => (StorageClass::Extern, Linkage::External),
			StorageClass::Static => (StorageClass::Static, Linkage::Internal),
			storage => (storage, Linkage::None),
		};

		for init_decl in decl.init_declarator_list.iter_mut() {
			let mut init_list_count = None;
			if let Some(ref mut init) = init_decl.initializer {
				self.initializer(init, &mut init_list_count);
			}
			let ident = &init_decl.identifier;
			let data_type = match &maybe_ty {
				Ok(ty) => ty,
				Err(false) => {
					let diag = diag::Diagnostic::error(
						diag::DiagKind::ImplicitInt(ident.name.clone()),
						ident.to_span(),
					);
					self.diagnostics.push(diag);
					continue;
				}
				Err(true) => {
					// do nothing
					continue;
				}
			};
			if init_decl.declarator.len() > 12 && self.warn_lvl == WarnLevel::All {
				// 5.2.4.1 translation limit
				let diag = diag::Diagnostic::warn(diag::DiagKind::DeclaratorLimit, ident.to_span());
				self.diagnostics.push(diag);
			}
			let mut var_dtype = data_type.clone();
			is_valid &= self.declarator_list(
				ident.to_span(),
				&mut init_decl.declarator,
				&mut var_dtype,
				false,
				DeclType::Decl,
				Some(ident.name.clone()),
				init_list_count,
			);
			if !is_valid {
				return false;
			}
			let entry = SymbolTableEntry {
				data_type: var_dtype,
				linkage,
				storage,
			};
			let key = Namespace::Ordinary(ident.name.clone());
			self.symtab.insert(key, entry);
		}
		is_valid
	}

	fn enum_specifier(&mut self, _spec: &mut EnumSpecifier) {
		todo!("enum-specifier")
	}
	fn enumerator(&mut self, enumerator: &mut Enumerator) {
		if let Some(ref mut expr) = enumerator.constant_expr {
			self.expr(expr);
		}
	}

	pub(super) fn struct_declaration(
		&mut self,
		struct_decl: &mut StructDeclaration,
	) -> Option<Vec<dtype::MemberType>> {
		let mut result = vec![];
		let mut is_valid = true;
		// only type-specifier and type-qualifier is syntactically allowed here.
		let ty_opt = self.specifiers_dtype(&mut struct_decl.specifiers);
		for decl in struct_decl.struct_declaration_list.iter_mut() {
			let name_opt = decl.ident.as_ref().and_then(|v| Some(v.name.clone()));
			let span = match &decl.ident {
				Some(ident) => ident.to_span(),
				None => struct_decl.specifiers.first_span.clone(),
			};
			let Ok(mut data_type) = ty_opt.clone() else {
				is_valid = false;
				continue;
			};
			is_valid &= self.declarator_list(
				span.clone(),
				&mut decl.declarators,
				&mut data_type,
				false,
				DeclType::Decl,
				name_opt.clone(),
				None,
			);

			let bits = if let dtype::TypeKind::Scalar(scalar) = &data_type.kind {
				if !scalar.is_integral() && decl.const_expr.is_some() {
					let kind = diag::DiagKind::BitfieldNonIntegral(name_opt);
					let diag = diag::Diagnostic::error(kind, span.clone());
					self.diagnostics.push(diag);
					is_valid = false;
					continue;
				}
				match decl.const_expr.as_mut().map(|val| val.to_u32()) {
					Some(Ok(value)) => {
						if value <= scalar.bits() {
							Some(value)
						} else {
							let kind = diag::DiagKind::BitfieldRange(name_opt);
							let diag = diag::Diagnostic::error(kind, span.clone());
							self.diagnostics.push(diag);
							is_valid = false;
							continue;
						}
					}
					Some(Err(ConversionError::Expr(mut expr))) => {
						// collect errors from expression first
						is_valid &= self.expr(&mut expr);
						if is_valid {
							let kind = diag::DiagKind::NonConstExpr;
							let diag = diag::Diagnostic::error(kind, span.clone());
							self.diagnostics.push(diag);
							is_valid = false;
						}
						continue;
					}
					Some(Err(ConversionError::OutOfRange)) => {
						let kind = diag::DiagKind::BitfieldRange(name_opt);
						let diag = diag::Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
						is_valid = false;
						continue;
					}
					None => None,
				}
			} else if decl.const_expr.is_some() {
				let kind = diag::DiagKind::BitfieldNonIntegral(name_opt);
				let diag = diag::Diagnostic::error(kind, span.clone());
				self.diagnostics.push(diag);
				is_valid = false;
				continue;
			} else {
				None
			};
			result.push(dtype::MemberType {
				name: name_opt,
				dtype: Box::new(data_type),
				bits,
			});
		}
		match is_valid {
			true => Some(result),
			false => None,
		}
	}

	fn initializer(&mut self, init: &mut Initializer, list_count: &mut Option<u32>) -> bool {
		let mut is_valid = true;
		match init {
			Initializer::Expr(expr) => is_valid &= self.expr(expr),
			Initializer::InitializerList(InitializerList(list)) => {
				*list_count = Some(list.len().try_into().unwrap());
			}
		}
		is_valid
	}

	fn declarator_list(
		&mut self,
		span: diag::Span,
		decl_list: &mut [Declarator],
		data_type: &mut dtype::DataType,
		mut is_param: bool,
		mut decl_type: DeclType,
		name: Option<String>,
		init_list_count: Option<u32>,
	) -> bool {
		let mut is_valid = true;
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
							is_valid = false;
						} else if !is_param {
							let kind = diag::DiagKind::InvalidStar;
							let diag = diag::Diagnostic::error(kind, array.span.clone());
							self.diagnostics.push(diag);
							is_valid = false;
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
						is_valid = false;
					}
					last_is_ptr = false;
				}
				Declarator::ParamList(type_list) => {
					if !last_is_ptr {
						let kind = diag::DiagKind::FnRetFn(name.clone());
						let diag = diag::Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
						is_valid = false;
					}
					last_is_ptr = false;
				}
			};
		}

		if !is_valid {
			return false;
		}

		// reversed iterator because recursive type construction has
		// data type at the end
		for declarator in decl_list.iter_mut().rev() {
			*data_type = match declarator {
				Declarator::Array(array) => {
					let mut type_qual = dtype::TypeQual::default();
					for syn::TypeQualifier { kind, .. } in array.type_qualifiers.iter() {
						match kind {
							TypeQualifierKind::Const => type_qual.is_const = true,
							TypeQualifierKind::Restrict => type_qual.is_restrict = true,
							TypeQualifierKind::Volatile => type_qual.is_volatile = true,
						}
					}
					if !is_param || array.has_static {
						let length = if let Some(assign_expr) = &mut array.assignment_expr {
							match assign_expr.to_u32() {
								Ok(val) => {
									if val == 0 {
										let kind = diag::DiagKind::ArrayMinRange;
										let diag = diag::Diagnostic::error(kind, span.clone());
										self.diagnostics.push(diag);
										is_valid = false;
										continue;
									} else {
										dtype::ArrayLength::Fixed(val)
									}
								}
								Err(ConversionError::OutOfRange) => {
									let kind = diag::DiagKind::ArrayMaxRange;
									let diag = diag::Diagnostic::error(kind, span.clone());
									self.diagnostics.push(diag);
									is_valid = false;
									continue;
								}
								Err(ConversionError::Expr(_)) => dtype::ArrayLength::Variable,
							}
						} else if let Some(count) = init_list_count {
							dtype::ArrayLength::Fixed(count as u32)
						} else {
							dtype::ArrayLength::Incomplete
						};
						let array_type = dtype::ArrayType {
							component: Box::new(data_type.clone()),
							length,
						};
						dtype::DataType {
							kind: dtype::TypeKind::Array(array_type),
							qual: type_qual,
						}
					} else if array.has_star {
						let array_type = dtype::ArrayType {
							component: Box::new(data_type.clone()),
							length: dtype::ArrayLength::Variable,
						};
						dtype::DataType {
							kind: dtype::TypeKind::Array(array_type),
							qual: type_qual,
						}
					} else {
						let mut type_qual = dtype::TypeQual::default();
						for qual in array.type_qualifiers.iter() {
							match qual.kind {
								TypeQualifierKind::Const => type_qual.is_const = true,
								TypeQualifierKind::Restrict => type_qual.is_restrict = true,
								TypeQualifierKind::Volatile => type_qual.is_volatile = true,
							}
						}
						let ptr_type = dtype::PtrType(Box::new(data_type.clone()));
						dtype::DataType {
							kind: dtype::TypeKind::Pointer(ptr_type),
							qual: type_qual,
						}
					}
				}
				Declarator::Pointer(pointer) => {
					let ptr_type = dtype::PtrType(Box::new(data_type.clone()));
					dtype::DataType {
						kind: dtype::TypeKind::Pointer(ptr_type),
						qual: dtype::TypeQual {
							is_const: pointer.is_const,
							is_restrict: pointer.is_restrict,
							is_volatile: pointer.is_volatile,
						},
					}
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
					dtype::DataType {
						kind: dtype::TypeKind::Function(func_type),
						qual: dtype::TypeQual::default(),
					}
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
					dtype::DataType {
						kind: dtype::TypeKind::Function(func_type),
						qual: dtype::TypeQual::default(),
					}
				}
			};
		}
		return is_valid;
	}
	fn type_qualifier(&mut self, qual: &mut TypeQualifier) {
		match qual.kind {
			TypeQualifierKind::Const => (),
			TypeQualifierKind::Restrict => (),
			TypeQualifierKind::Volatile => (),
		}
	}
	fn designation(&mut self, desig: &mut Vec<Designator>) -> bool {
		let mut is_valid = true;
		for ref mut desig in desig.iter_mut() {
			is_valid &= self.designator(desig)
		}
		is_valid
	}
	fn designator(&mut self, desig: &mut Designator) -> bool {
		use Designator::*;
		let mut is_valid = true;
		match desig {
			ConstExpr(expr) => is_valid &= self.expr(expr),
			Dot(_) => (),
		}
		is_valid
	}
}
