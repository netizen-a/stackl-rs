use crate::analysis::sem::DeclType;
use crate::analysis::sem::Linkage;
use crate::analysis::sem::Namespace;
use crate::analysis::sem::StorageClass;
use crate::analysis::sem::SymbolTableEntry;
use crate::analysis::syn;
use crate::analysis::syn::*;
use crate::analysis::tok;
use crate::cli::WarnLevel;
use crate::data_types::DataType;
use crate::data_types::FuncType;
use crate::data_types::TypeKind;
use crate::data_types as dtype;
use crate::diagnostics::*;
use crate::symtab::SymbolTableError;

impl super::SemanticParser {
	pub(super) fn declaration(&mut self, decl: &mut Declaration, default_sc: StorageClass) -> bool {
		self.tree_builder.begin_child("declaration".to_string());
		let mut is_valid = true;
		let maybe_ty = self.specifiers_dtype(&mut decl.specifiers);
		let maybe_sc = self.specifiers_storage(&mut decl.specifiers);
		let (storage, linkage) = match maybe_sc.map(|v| v.kind).unwrap_or(default_sc) {
			StorageClass::Extern => (StorageClass::Extern, Linkage::External),
			StorageClass::Static => (StorageClass::Static, Linkage::Internal),
			storage => (storage, Linkage::None),
		};

		for init_decl in decl.init_declarator_list.iter_mut() {
			let ident = &init_decl.identifier;
			{
				let span = ident.to_span();
				let (actual_line, reported_line, col) =
					self.diagnostics.get_location(&span).unwrap();
				let text = format!(
					"init-declarator <line:{actual_line}:{reported_line}, col:{col}> {}",
					ident.name
				);
				self.tree_builder.begin_child(text);
			}
			let mut init_list_count = None;
			if let Some(ref mut init) = init_decl.initializer {
				self.initializer(init, &mut init_list_count);
			}
			let data_type =
				self.unwrap_or_poison(maybe_ty.clone(), Some(ident.name.clone()), ident.to_span());
			if init_decl.declarator.len() > 12 && self.warn_lvl == WarnLevel::All {
				// 5.2.4.1 translation limit
				let diag = Diagnostic::warn(DiagKind::DeclaratorLimit, ident.to_span());
				self.diagnostics.push(diag);
			}
			let mut var_dtype = data_type.clone();
			self.declarator_list(
				ident.to_span(),
				&mut init_decl.declarator,
				&mut var_dtype,
				false,
				DeclType::Decl,
				Some(ident.name.clone()),
				init_list_count,
			);
			let entry = SymbolTableEntry {
				data_type: var_dtype,
				linkage,
				storage,
				span: ident.to_span(),
			};
			let key = Namespace::Ordinary(ident.name.clone());
			match self.symtab.insert(key.clone(), entry) {
				Err(SymbolTableError::AlreadyExists) => {
					// test warning
					let entry = self.symtab.global_lookup(&key).unwrap();
					let kind = DiagKind::SymbolAlreadyExists(ident.name.clone());
					let error = Diagnostic::warn(kind, entry.span.clone());
					self.diagnostics.push(error);
				}
				Err(SymbolTableError::InvalidScope) => {
					// this should never happen
					let kind = DiagKind::Internal("symbol table has invalid scope".to_string());
					let error = Diagnostic::fatal(kind, Some(ident.to_span()));
					self.diagnostics.push_and_exit(error);
				}
				Ok(_) => {
					// all good :)
				}
			}
			self.tree_builder.end_child();
		}
		self.tree_builder.end_child();
		is_valid
	}

	fn enum_specifier(&mut self, _spec: &mut EnumSpecifier) {
		todo!("enum-specifier")
	}
	// fn enumerator(&mut self, enumerator: &mut Enumerator) {
	// 	if let Some(ref mut expr) = enumerator.constant_expr {
	// 		self.expr(expr);
	// 	}
	// }

	pub(super) fn struct_declaration(
		&mut self,
		struct_decl: &mut StructDeclaration,
		member_is_named: &mut bool,
	) -> Option<Vec<dtype::MemberType>> {
		self.tree_builder
			.begin_child("struct-declarator".to_string());
		let mut result = vec![];
		let mut is_valid = true;
		// only type-specifier and type-qualifier is syntactically allowed here.
		let ty_opt = self.specifiers_dtype(&mut struct_decl.specifiers);
		for decl in struct_decl.struct_declarator_list.iter_mut() {
			let name_opt = decl.ident.as_ref().and_then(|v| Some(v.name.clone()));
			*member_is_named |= name_opt.is_some();

			let member_span = match &decl.ident {
				Some(ident) => ident.to_span(),
				None => struct_decl.specifiers.first_span.clone(),
			};
			let mut data_type =
				self.unwrap_or_poison(ty_opt.clone(), name_opt.clone(), member_span.clone());
			if let dtype::TypeKind::Poison = data_type.kind {
				is_valid = false;
				continue;
			}

			self.declarator_list(
				member_span.clone(),
				&mut decl.declarators,
				&mut data_type,
				false,
				DeclType::Decl,
				name_opt.clone(),
				None,
			);

			if let dtype::TypeKind::Poison = data_type.kind {
				is_valid = false;
			}

			let bits = if let dtype::TypeKind::Scalar(scalar) = &data_type.kind {
				if !scalar.is_integral() && decl.const_expr.is_some() {
					let kind = DiagKind::BitfieldNonIntegral(name_opt);
					let diag = Diagnostic::error(kind, member_span.clone());
					self.diagnostics.push(diag);
					is_valid = false;
					data_type.kind = dtype::TypeKind::Poison;
					continue;
				}
				match decl.const_expr.as_mut().map(|val| val.to_u32()) {
					Some(Ok(value)) => {
						if value <= scalar.bits() {
							Some(value)
						} else {
							let kind = DiagKind::BitfieldRange(name_opt);
							let diag = Diagnostic::error(kind, member_span.clone());
							self.diagnostics.push(diag);
							is_valid = false;
							data_type.kind = dtype::TypeKind::Poison;
							continue;
						}
					}
					Some(Err(ConversionError::Expr(mut expr))) => {
						// collect errors from expression first
						is_valid &= !self.expr(&mut expr).is_poisoned();
						if is_valid {
							let kind = DiagKind::NonConstExpr;
							let diag = Diagnostic::error(kind, member_span.clone());
							self.diagnostics.push(diag);
							is_valid = false;
							data_type.kind = dtype::TypeKind::Poison;
						}
						continue;
					}
					Some(Err(ConversionError::OutOfRange)) => {
						let kind = DiagKind::BitfieldRange(name_opt);
						let diag = Diagnostic::error(kind, member_span.clone());
						self.diagnostics.push(diag);
						is_valid = false;
						data_type.kind = dtype::TypeKind::Poison;
						continue;
					}
					None => None,
				}
			} else if decl.const_expr.is_some() {
				let kind = DiagKind::BitfieldNonIntegral(name_opt);
				let diag = Diagnostic::error(kind, member_span.clone());
				self.diagnostics.push(diag);
				is_valid = false;
				data_type.kind = dtype::TypeKind::Poison;
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
		self.tree_builder.end_child();

		match is_valid {
			true => Some(result),
			false => None,
		}
	}

	fn initializer(&mut self, init: &mut Initializer, list_count: &mut Option<u32>) -> bool {
		let mut is_valid = true;
		match init {
			Initializer::Expr(expr) => is_valid &= !self.expr(expr).is_poisoned(),
			Initializer::InitializerList(InitializerList(list)) => {
				*list_count = Some(list.len().try_into().unwrap());
				self.tree_builder
					.add_empty_child("initializer-list".to_string());
			}
		}
		is_valid
	}

	pub(super) fn declarator_list(
		&mut self,
		span: Span,
		decl_list: &mut [Declarator],
		data_type: &mut dtype::DataType,
		is_param: bool,
		mut decl_type: DeclType,
		name: Option<String>,
		init_list_count: Option<u32>,
	) {
		let mut last_is_ptr = decl_type != DeclType::FnDef;
		let mut last_is_arr = false;
		// first iteration is for type checking
		for declarator in decl_list.iter_mut() {
			match declarator {
				Declarator::Array(array) => {
					if array.has_star && decl_type != DeclType::Proto {
						if is_param && decl_type == DeclType::FnDef {
							let kind = DiagKind::UnboundVLA;
							let diag = Diagnostic::error(kind, span.clone());
							self.diagnostics.push(diag);
							data_type.kind = dtype::TypeKind::Poison;
						} else if !is_param {
							let kind = DiagKind::InvalidStar;
							let diag = Diagnostic::error(kind, array.span.clone());
							self.diagnostics.push(diag);
							data_type.kind = dtype::TypeKind::Poison;
						}
					}
					last_is_ptr = false;
					last_is_arr = true;
				}
				Declarator::Pointer(_) => {
					last_is_ptr = true;
					last_is_arr = false;
				}
				Declarator::IdentList(_) => {
					if last_is_arr {
						let error_type = DataType {
							kind: TypeKind::Function(FuncType { params: vec![], ret: Box::new(data_type.clone()), is_variadic: false, is_inline: false }),
							qual: Default::default(),
						};
						let kind = DiagKind::ArrayOfFunctions{name: name.clone(), dtype: error_type};
						let diag = Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
						data_type.kind = dtype::TypeKind::Poison;
					} else if !last_is_ptr {
						let kind = DiagKind::FnRetFn(name.clone());
						let diag = Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
						data_type.kind = dtype::TypeKind::Poison;
					}
					last_is_ptr = false;
					last_is_arr = false;
				}
				Declarator::ParamList(type_list) => {
					if last_is_arr {
						if DeclType::FnDef == decl_type && is_param {
							decl_type = DeclType::Proto;
						}
						let Some(params) = self.param_list(type_list, decl_type) else {
							data_type.kind = dtype::TypeKind::Poison;
							return;
						};
						let error_type = DataType {
							kind: TypeKind::Function(FuncType { params: vec![], ret: Box::new(data_type.clone()), is_variadic: false, is_inline: false }),
							qual: Default::default(),
						};
						let kind = DiagKind::ArrayOfFunctions{name: name.clone(), dtype: error_type.clone()};
						let diag = Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
						data_type.kind = dtype::TypeKind::Poison;
					} else if !last_is_ptr {
						let kind = DiagKind::FnRetFn(name.clone());
						let diag = Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
						data_type.kind = dtype::TypeKind::Poison;
					}
					last_is_ptr = false;
					last_is_arr = false;
				}
			};
		}

		if let dtype::TypeKind::Poison = data_type.kind {
			return;
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
					let array_length = if let Some(assign_expr) = &mut array.assignment_expr {
						match assign_expr.to_u32() {
							Ok(val) => {
								if val == 0 {
									let kind = DiagKind::ArrayMinRange;
									let diag = Diagnostic::error(kind, span.clone());
									self.diagnostics.push(diag);
									data_type.kind = dtype::TypeKind::Poison;
									continue;
								} else {
									dtype::ArrayLength::Fixed(val)
								}
							}
							Err(ConversionError::OutOfRange) => {
								let kind = DiagKind::ArrayMaxRange;
								let diag = Diagnostic::error(kind, span.clone());
								self.diagnostics.push(diag);
								data_type.kind = dtype::TypeKind::Poison;
								continue;
							}
							Err(ConversionError::Expr(expr)) => {
								dtype::ArrayLength::VLA(dtype::VlaLength::Expr(expr))
							}
						}
					} else if array.has_star {
						dtype::ArrayLength::VLA(dtype::VlaLength::Star)
					} else if let (Some(count), true) = (init_list_count, !is_param) {
						dtype::ArrayLength::Fixed(count)
					} else {
						println!("incomplete: {name:?}");
						dtype::ArrayLength::Incomplete
					};
					let array_type = dtype::ArrayType {
						component: Box::new(data_type.clone()),
						length: array_length,
						is_decayed: is_param,
						has_static: array.has_static,
					};
					dtype::DataType {
						kind: dtype::TypeKind::Array(array_type),
						qual: type_qual,
					}
				}
				Declarator::Pointer(pointer) => dtype::DataType {
					kind: dtype::TypeKind::Pointer(Box::new(data_type.clone())),
					qual: dtype::TypeQual {
						is_const: pointer.is_const,
						is_restrict: pointer.is_restrict,
						is_volatile: pointer.is_volatile,
					},
				},
				Declarator::IdentList(ident_list) => {
					// TODO: check if this error is valid
					if ident_list.ident_list.len() > 0 {
						let kind = DiagKind::DeclIdentList;
						let diag = Diagnostic::error(kind, ident_list.span.clone());
						self.diagnostics.push(diag);
					}
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
					let Some(params) = self.param_list(type_list, decl_type) else {
						data_type.kind = dtype::TypeKind::Poison;
						return;
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
	}
}
