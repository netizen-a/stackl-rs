use crate::analysis::sem::DeclType;
use crate::analysis::syn;
use crate::analysis::tok;
use crate::cli::WarnLevel;
use crate::data_type::*;
use crate::diagnostics::*;
use crate::symbol_table::StorageClass;
use crate::symbol_table as sym;

impl super::SemanticParser {
	pub(super) fn declaration(
		&mut self,
		decl: &mut syn::Declaration,
		default_sc: syn::StorageClass,
		in_func: bool,
	) -> bool {
		self.tree_builder.begin_child("declaration".to_string());
		let mut is_valid = true;
		let maybe_ty = self.specifiers_dtype(&mut decl.specifiers, in_func);
		let maybe_sc = self.specifiers_storage(&mut decl.specifiers);
		let (storage, linkage) = match maybe_sc.map(|v| v.kind).unwrap_or(default_sc) {
			syn::StorageClass::Extern => (syn::StorageClass::Extern, sym::Linkage::External),
			syn::StorageClass::Static => (syn::StorageClass::Static, sym::Linkage::Internal),
			storage => (storage, sym::Linkage::None),
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
			let mut init_list_count = vec![];
			if let Some(ref mut init) = init_decl.initializer {
				self.initializer(init, &mut init_list_count, in_func);
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
			// convert C storage class to symbol storage class
			let storage: StorageClass = match storage {
				syn::StorageClass::Auto => StorageClass::Automatic,
				syn::StorageClass::Extern => StorageClass::Static,
				syn::StorageClass::Register => StorageClass::Register,
				syn::StorageClass::Static => StorageClass::Static,
				syn::StorageClass::Typedef => StorageClass::Typedef,
			};
			let new_entry = sym::SymbolTableEntry {
				data_type: var_dtype,
				linkage,
				storage,
				span: ident.to_span(),
				is_decl: true,
			};
			let key = sym::Namespace::Ordinary(ident.name.clone());
			if let Err(sym::SymbolTableError::AlreadyExists(prev_entry)) =
				self.symtab.insert(key.clone(), new_entry.clone())
			{
				let kind =
					DiagKind::SymbolAlreadyExists(ident.name.clone(), prev_entry.data_type.clone());
				let mut error = Diagnostic::error(kind, prev_entry.span.clone());
				error.push_span(
					new_entry.span,
					&format!("`{}` redefined here", ident.name.clone()),
				);
				self.diagnostics.push(error);
			}
			self.tree_builder.end_child();
		}
		self.tree_builder.end_child();
		is_valid
	}

	fn enum_specifier(&mut self, _spec: &mut syn::EnumSpecifier) {
		todo!("enum-specifier")
	}
	// fn enumerator(&mut self, enumerator: &mut Enumerator) {
	// 	if let Some(ref mut expr) = enumerator.constant_expr {
	// 		self.expr(expr);
	// 	}
	// }

	pub(super) fn struct_declaration(
		&mut self,
		struct_decl: &mut syn::StructDeclaration,
		member_is_named: &mut bool,
		in_func: bool,
	) -> Option<Vec<MemberType>> {
		self.tree_builder
			.begin_child("struct-declarator".to_string());
		let mut result = vec![];
		let mut is_valid = true;
		// only type-specifier and type-qualifier is syntactically allowed here.
		let ty_opt = self.specifiers_dtype(&mut struct_decl.specifiers, in_func);
		for decl in struct_decl.struct_declarator_list.iter_mut() {
			let name_opt = decl.ident.as_ref().and_then(|v| Some(v.name.clone()));
			*member_is_named |= name_opt.is_some();

			let member_span = match &decl.ident {
				Some(ident) => ident.to_span(),
				None => struct_decl.specifiers.first_span.clone(),
			};
			let mut data_type =
				self.unwrap_or_poison(ty_opt.clone(), name_opt.clone(), member_span.clone());
			if let TypeKind::Poison = data_type.kind {
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
				vec![],
			);

			if let TypeKind::Poison = data_type.kind {
				is_valid = false;
			}

			let bits = if let TypeKind::Scalar(scalar) = &data_type.kind {
				if !scalar.is_integral() && decl.const_expr.is_some() {
					let kind = DiagKind::BitfieldNonIntegral(name_opt);
					let diag = Diagnostic::error(kind, member_span.clone());
					self.diagnostics.push(diag);
					is_valid = false;
					data_type.kind = TypeKind::Poison;
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
							data_type.kind = TypeKind::Poison;
							continue;
						}
					}
					Some(Err(syn::ConversionError::Expr(mut expr))) => {
						// collect errors from expression first
						is_valid &= !self.expr(&mut expr, in_func).is_poisoned();
						if is_valid {
							let kind = DiagKind::NonIntConstExpr;
							let diag = Diagnostic::error(kind, member_span.clone());
							self.diagnostics.push(diag);
							is_valid = false;
							data_type.kind = TypeKind::Poison;
						}
						continue;
					}
					Some(Err(syn::ConversionError::OutOfRange)) => {
						let kind = DiagKind::BitfieldRange(name_opt);
						let diag = Diagnostic::error(kind, member_span.clone());
						self.diagnostics.push(diag);
						is_valid = false;
						data_type.kind = TypeKind::Poison;
						continue;
					}
					None => None,
				}
			} else if decl.const_expr.is_some() {
				let kind = DiagKind::BitfieldNonIntegral(name_opt);
				let diag = Diagnostic::error(kind, member_span.clone());
				self.diagnostics.push(diag);
				is_valid = false;
				data_type.kind = TypeKind::Poison;
				continue;
			} else {
				None
			};
			result.push(MemberType {
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

	fn initializer(
		&mut self,
		init: &mut syn::Initializer,
		list_count: &mut Vec<(Span, u32)>,
		in_func: bool,
	) -> bool {
		let mut is_valid = true;
		match init {
			syn::Initializer::Expr(expr) => is_valid &= !self.expr(expr, in_func).is_poisoned(),
			syn::Initializer::InitializerList(span, syn::InitializerList(list)) => {
				list_count.push((span.clone(), list.len().try_into().unwrap()));
				self.tree_builder
					.begin_child("initializer-list".to_string());
				for (desig_list, init) in list.iter_mut() {
					self.initializer(init, list_count, in_func);
				}
				self.tree_builder.end_child();
			}
		}
		is_valid
	}

	pub(super) fn declarator_list(
		&mut self,
		span: Span,
		decl_list: &mut [syn::Declarator],
		data_type: &mut DataType,
		is_param: bool,
		mut decl_type: DeclType,
		name: Option<String>,
		mut init_list_vec: Vec<(Span, u32)>,
	) {
		let mut last_is_ptr = decl_type != DeclType::FnDef;
		let mut last_is_arr = false;
		// first iteration is for type checking
		for declarator in decl_list.iter_mut() {
			match declarator {
				syn::Declarator::Array(array) => {
					if array.has_star && decl_type != DeclType::Proto {
						if is_param && decl_type == DeclType::FnDef {
							let kind = DiagKind::UnboundVLA;
							let diag = Diagnostic::error(kind, span.clone());
							self.diagnostics.push(diag);
							data_type.kind = TypeKind::Poison;
						} else if !is_param {
							let kind = DiagKind::InvalidStar;
							let diag = Diagnostic::error(kind, array.span.clone());
							self.diagnostics.push(diag);
							data_type.kind = TypeKind::Poison;
						}
					}
					last_is_ptr = false;
					last_is_arr = true;
				}
				syn::Declarator::Pointer(_) => {
					last_is_ptr = true;
					last_is_arr = false;
				}
				syn::Declarator::IdentList(_) => {
					if last_is_arr {
						let error_type = DataType {
							kind: TypeKind::Function(FuncType {
								params: vec![],
								ret: Box::new(data_type.clone()),
								is_variadic: false,
								is_inline: false,
							}),
							qual: Default::default(),
						};
						let kind = DiagKind::ArrayOfFunctions {
							name: name.clone(),
							dtype: error_type,
						};
						let diag = Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
						data_type.kind = TypeKind::Poison;
					} else if !last_is_ptr {
						let kind = DiagKind::FnRetFn(name.clone());
						let diag = Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
						data_type.kind = TypeKind::Poison;
					}
					last_is_ptr = false;
					last_is_arr = false;
				}
				syn::Declarator::ParamList(type_list) => {
					if last_is_arr {
						if DeclType::FnDef == decl_type && is_param {
							decl_type = DeclType::Proto;
						}
						let Some(params) = self.param_list(type_list, decl_type) else {
							data_type.kind = TypeKind::Poison;
							return;
						};
						let error_type = DataType {
							kind: TypeKind::Function(FuncType {
								params,
								ret: Box::new(data_type.clone()),
								is_variadic: false,
								is_inline: false,
							}),
							qual: Default::default(),
						};
						let kind = DiagKind::ArrayOfFunctions {
							name: name.clone(),
							dtype: error_type.clone(),
						};
						let diag = Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
						data_type.kind = TypeKind::Poison;
					} else if !last_is_ptr {
						let kind = DiagKind::FnRetFn(name.clone());
						let diag = Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
						data_type.kind = TypeKind::Poison;
					}
					last_is_ptr = false;
					last_is_arr = false;
				}
			};
		}

		if let TypeKind::Poison = data_type.kind {
			return;
		}

		// reversed iterator because recursive type construction has
		// data type at the end
		for declarator in decl_list.iter_mut().rev() {
			*data_type = match declarator {
				syn::Declarator::Array(array) => {
					let mut type_qual = TypeQual::default();
					for syn::TypeQualifier { kind, .. } in array.type_qualifiers.iter() {
						match kind {
							syn::TypeQualifierKind::Const => type_qual.is_const = true,
							syn::TypeQualifierKind::Restrict => type_qual.is_restrict = true,
							syn::TypeQualifierKind::Volatile => type_qual.is_volatile = true,
						}
					}
					let array_length = if let Some(assign_expr) = &mut array.assignment_expr {
						let init_list = init_list_vec.pop();
						match assign_expr.to_u32() {
							Ok(val) => {
								if val == 0 {
									let kind = DiagKind::ArrayMinRange;
									let diag = Diagnostic::error(kind, span.clone());
									self.diagnostics.push(diag);
									data_type.kind = TypeKind::Poison;
									continue;
								} else if let Some((span, init_size)) = init_list {
									if init_size > val {
										let kind = DiagKind::ArrayExcessElements;
										let error = Diagnostic::error(kind, span.clone());
										self.diagnostics.push(error);
										data_type.kind = TypeKind::Poison;
										continue;
									} else {
										ArrayLength::Fixed(val)
									}
								} else {
									ArrayLength::Fixed(val)
								}
							}
							Err(syn::ConversionError::OutOfRange) => {
								let kind = DiagKind::ArrayMaxRange;
								let diag = Diagnostic::error(kind, span.clone());
								self.diagnostics.push(diag);
								data_type.kind = TypeKind::Poison;
								continue;
							}
							Err(syn::ConversionError::Expr(expr)) => {
								if let Some((span, _)) = init_list {
									let kind = DiagKind::VlaInitList;
									let diag = Diagnostic::error(kind, span.clone());
									self.diagnostics.push(diag);
									data_type.kind = TypeKind::Poison;
									continue;
								} else {
									ArrayLength::VLA(VlaLength::Expr(expr))
								}
							}
						}
					} else if array.has_star {
						ArrayLength::VLA(VlaLength::Star)
					} else if let (Some((_, count)), true) = (init_list_vec.pop(), !is_param) {
						ArrayLength::Fixed(count)
					} else {
						ArrayLength::Incomplete
					};
					if let (ArrayLength::Incomplete, false) = (&array_length, is_param) {
						let kind = DiagKind::ArrayDeclIncomplete;
						let error = Diagnostic::error(kind, span.clone());
						self.diagnostics.push(error);
						DataType::POISON
					} else {
						let array_type = ArrayType {
							component: Box::new(data_type.clone()),
							length: array_length,
							is_decayed: is_param,
							has_static: array.has_static,
						};
						DataType {
							kind: TypeKind::Array(array_type),
							qual: type_qual,
						}
					}
				}
				syn::Declarator::Pointer(pointer) => DataType {
					kind: TypeKind::Pointer(Box::new(data_type.clone())),
					qual: TypeQual {
						is_const: pointer.is_const,
						is_restrict: pointer.is_restrict,
						is_volatile: pointer.is_volatile,
					},
				},
				syn::Declarator::IdentList(ident_list) => {
					// TODO: check if this error is valid
					if ident_list.ident_list.len() > 0 {
						let kind = DiagKind::DeclIdentList;
						let diag = Diagnostic::error(kind, ident_list.span.clone());
						self.diagnostics.push(diag);
					}
					let func_type = FuncType {
						params: vec![],
						ret: Box::new(data_type.clone()),
						is_variadic: false,
						is_inline: false,
					};
					DataType {
						kind: TypeKind::Function(func_type),
						qual: TypeQual::default(),
					}
				}
				syn::Declarator::ParamList(type_list) => {
					if DeclType::FnDef == decl_type && is_param {
						decl_type = DeclType::Proto;
					}
					let Some(params) = self.param_list(type_list, decl_type) else {
						data_type.kind = TypeKind::Poison;
						return;
					};

					let func_type = FuncType {
						params,
						ret: Box::new(data_type.clone()),
						is_variadic: type_list.is_variadic,
						is_inline: false,
					};
					DataType {
						kind: TypeKind::Function(func_type),
						qual: TypeQual::default(),
					}
				}
			};
		}
		for (span, count) in init_list_vec {
			let kind = DiagKind::ArrayExcessElements;
			let error = Diagnostic::error(kind, span);
			self.diagnostics.push(error);
		}
	}
}
