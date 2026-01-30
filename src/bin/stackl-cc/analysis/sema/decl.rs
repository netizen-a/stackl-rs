// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::cell::OnceCell;

use super::expr::ExprContext;
use crate::analysis::sema::DeclType;
use crate::analysis::syn;
use crate::analysis::tok;
use crate::cli::WarnLevel;
use crate::data_type::*;
use crate::diagnostics::*;
use crate::symtab as sym;
use crate::synthesis::icg;
use stackl::ssa::data as ssa;

impl super::SemanticParser<'_> {
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
		let (storage, linkage): (sym::StorageClass, sym::Linkage) = match maybe_sc
			.map(|v| v.kind)
			.unwrap_or(default_sc)
		{
			syn::StorageClass::Auto => (sym::StorageClass::Automatic, sym::Linkage::Internal),
			syn::StorageClass::Extern => (sym::StorageClass::Static, sym::Linkage::External),
			syn::StorageClass::Register => (sym::StorageClass::Register, sym::Linkage::Internal),
			syn::StorageClass::Static => (sym::StorageClass::Static, sym::Linkage::Internal),
			syn::StorageClass::Typedef => (sym::StorageClass::Typename, sym::Linkage::Internal),
		};

		if let Some(data_type) = &maybe_ty {
			self.declare_tag(data_type, decl.specifiers.to_span());
			decl.specifiers.storage = ssa::StorageClass::try_from(storage).ok();
		}

		for init_decl in decl.init_declarator_list.iter_mut() {
			let ident = &init_decl.identifier;
			let mut init_list_type = vec![];
			if let Some(ref mut init) = init_decl.initializer {
				init_list_type = self.initializer(init, in_func);
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
				init_list_type,
			);

			let span = ident.to_span();
			let (_, reported_line, col) = self.diagnostics.get_location(&span).unwrap();
			let text = format!(
				"init-declarator <line:{reported_line}, col:{col}> `{}` '{var_dtype}'",
				ident.name
			);
			self.tree_builder.begin_child(text);
			if let Some(syn::Initializer::Expr(expr)) = &mut init_decl.initializer {
				let from_type = &self.expr_no_print(
					expr,
					&ExprContext {
						in_func,
						is_mut: true,
						enabled_diag: false,
					},
				);
				let to_type = &var_dtype;
				self.convert_type(expr, from_type, to_type, expr.to_span());
				if self.print_ast {
					self.expr(
						expr,
						&ExprContext {
							in_func,
							is_mut: false,
							enabled_diag: false,
						},
					);
				}
			}

			let new_entry = sym::SymbolTableEntry {
				data_type: var_dtype,
				linkage,
				storage: storage.clone(),
				span: ident.to_span(),
				is_decl: true,
			};
			let key = ident.name.clone();
			if let Err(sym::SymbolTableError::AlreadyExists(prev_entry)) =
				self.ordinary_table.insert(key.clone(), new_entry.clone())
			{
				let kind =
					DiagKind::SymbolAlreadyExists(ident.name.clone(), prev_entry.data_type.clone());
				let mut error = Diagnostic::error(kind, prev_entry.span.clone());
				error.push_span(
					new_entry.span,
					&format!("`{}` redefined here", ident.name.clone()),
				);
				self.diagnostics.push(error);
			} else if let (Ok(sc), Ok(layout)) = (
				ssa::StorageClass::try_from(storage),
				icg::DataLayout::try_from(new_entry.data_type.kind),
			) {
				self.data_layouts.as_mut().map(|h| h.insert(layout.clone()));
				decl.specifiers.storage = Some(sc);
				decl.specifiers.layout = Some(layout);
			}
			self.tree_builder.end_child();
		}
		self.tree_builder.end_child();
		is_valid
	}

	pub(super) fn struct_declaration(
		&mut self,
		struct_decl: &mut syn::StructDeclaration,
		member_is_named: &mut bool,
		in_func: bool,
	) -> Option<Vec<MemberType>> {

		let mut result = vec![];
		let mut is_valid = true;
		// only type-specifier and type-qualifier is syntactically allowed here.
		let ty_opt = self.specifiers_dtype(&mut struct_decl.specifiers, in_func);

		if let Some(dtype) = &ty_opt {
			self.tree_builder
				.begin_child(format!("struct-declarator '{dtype}'"));
		} else {
			self.tree_builder
				.begin_child("struct-declarator '<undefined>'".to_string());
		}
		for decl in struct_decl.struct_declarator_list.iter_mut() {
			let name_opt = decl.ident.as_ref().and_then(|v| Some(v.name.clone()));
			*member_is_named |= name_opt.is_some();

			let member_span = match &decl.ident {
				Some(ident) => ident.to_span(),
				None => struct_decl.specifiers.to_span(),
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
						let expr_context = ExprContext {
							in_func,
							is_mut: true,
							enabled_diag: true,
						};
						is_valid &= !self.expr(&mut expr, &expr_context).is_poisoned();
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

			if let Some(name) = &name_opt {
				let (_, reported_line, col) = self.diagnostics.get_location(&member_span.clone()).unwrap();
				self.tree_builder.add_empty_child(format!("declarator <line:{reported_line}, col:{col}> `{}` '{}'", name, data_type));
			} else {
				self.tree_builder.add_empty_child(format!("declarator `<anonymous>` '{}'", data_type));
			}

			result.push(MemberType {
				ident: decl.ident.clone(),
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
		in_func: bool,
	) -> Vec<(syn::Expr, DataType, u32)> {
		match init {
			syn::Initializer::Expr(expr) => {
				let expr_context = ExprContext {
					in_func,
					is_mut: true,
					enabled_diag: true,
				};
				vec![(expr.clone(), self.expr_no_print(expr, &expr_context), 0)]
			}
			syn::Initializer::InitializerList(syn::InitializerList { span, list }) => {
				self.tree_builder
					.begin_child("initializer-list".to_string());
				let mut result: Vec<(syn::Expr, DataType, u32)> = vec![];
				let mut once = OnceCell::new();
				for (index, (_desig_list, init)) in list.iter_mut().enumerate() {
					let mut inner_data = self.initializer(init, in_func);
					let mut curr_data = inner_data.first_mut().unwrap();
					let last_data = once.get_or_init(|| curr_data.clone());
					if !self.dtype_eq(&curr_data.1, &last_data.1, span.to_span()) {
						let mut l_type = last_data.1.clone();
						let mut r_type = curr_data.1.clone();
						let callee_span = curr_data.0.to_span();
						self.convert_type(&mut curr_data.0, &mut r_type, &mut l_type, callee_span);
					}
				}

				if let Some((expr, data, _)) = once.get().cloned() {
					let kind = TypeKind::Array(ArrayType {
						component: Box::new(data),
						length: ArrayLength::Fixed(list.len() as _),
						is_decayed: false,
						has_static: false,
					});
					let array = DataType {
						kind,
						qual: Default::default(),
					};
					result.push((expr, array, list.len() as _));
				}

				self.tree_builder.end_child();
				result
			}
		}
	}

	pub(super) fn declarator_list(
		&mut self,
		span: Span,
		decl_list: &mut [syn::Declarator],
		data_type: &mut DataType,
		is_param: bool,
		mut decl_type: DeclType,
		name: Option<String>,
		mut init_list_vec: Vec<(syn::Expr, DataType, u32)>,
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
						let Some(symbol_list) = self.param_list(type_list, decl_type) else {
							data_type.kind = TypeKind::Poison;
							return;
						};
						let error_type = DataType {
							kind: TypeKind::Function(FuncType {
								params: symbol_list.iter().map(|s| s.1.clone()).collect(),
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
								} else if let Some((expr, _, init_size)) = init_list {
									if init_size > val {
										let kind = DiagKind::ArrayExcessElements;
										let error = Diagnostic::error(kind, expr.to_span());
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
								if let Some((span, _, _)) = init_list {
									let kind = DiagKind::VlaInitList;
									let diag = Diagnostic::error(kind, expr.to_span());
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
					} else if let (Some((_, _, count)), true) = (init_list_vec.pop(), !is_param) {
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
					let Some(symbol_list) = self.param_list(type_list, decl_type) else {
						data_type.kind = TypeKind::Poison;
						return;
					};

					let func_type = FuncType {
						params: symbol_list.iter().map(|s| s.1.clone()).collect(),
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
		for (expr, dtype, count) in init_list_vec {
			if let TypeKind::Array(_) = &dtype.kind {
				let kind = DiagKind::ArrayExcessElements;
				let error = Diagnostic::error(kind, expr.to_span());
				self.diagnostics.push(error);
			}
		}
	}
}
