// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::analysis::sema::DeclType;
use crate::analysis::syn;
use crate::cli::WarnLevel;
use crate::data_type::*;
use crate::diagnostics::*;
use crate::symtab as sym;
use crate::symtab::StorageClass;
use crate::synthesis::icg;
use stackl::ssa::data as ssa;

impl super::SemanticParser<'_> {
	pub(super) fn function_definition(&mut self, decl: &mut syn::FunctionDefinition) -> bool {
		let func_ident = &decl.ident;
		self.tree_builder
			.begin_child(format!("function-definition `{}`", func_ident.name));
		let maybe_sc = self.specifiers_storage(&mut decl.specifiers);
		let maybe_ty = self.specifiers_dtype(&mut decl.specifiers, false);

		let (storage, linkage): (StorageClass, sym::Linkage) = match &maybe_sc {
			None
			| Some(syn::StorageClassSpecifier {
				kind: syn::StorageClass::Extern,
				..
			}) => (StorageClass::Automatic, sym::Linkage::External),
			Some(syn::StorageClassSpecifier {
				kind: syn::StorageClass::Static,
				..
			}) => (StorageClass::Automatic, sym::Linkage::Internal),
			Some(storage) => {
				let kind = DiagKind::IllegalStorage(storage.kind);
				let diag = Diagnostic::error(kind, storage.to_span());
				self.diagnostics.push(diag);
				self.tree_builder.end_child();
				return false;
			}
		};
		let mut data_type = self.unwrap_or_poison(
			maybe_ty,
			Some(func_ident.name.clone()),
			func_ident.to_span(),
		);
		if !matches!(
			decl.declarators.first_mut(),
			None | Some(syn::Declarator::Pointer(_))
		) {
			self.declarator_list(
				decl.ident.to_span(),
				&mut decl.declarators[1..],
				&mut data_type,
				false,
				DeclType::FnDef,
				Some(decl.ident.name.clone()),
				vec![],
			);
		}

		let mut declaration_list = vec![];

		match decl.declarators.first_mut() {
			Some(syn::Declarator::IdentList(ident_list)) => {
				self.tree_builder
					.begin_child("identifier-list ( )".to_string());
				let func_type = FuncType {
					params: vec![],
					ret: Box::new(data_type.clone()),
					is_variadic: false,
					is_inline: !decl.specifiers.inline_list.is_empty(),
				};
				let new_entry = sym::SymbolTableEntry {
					data_type: DataType {
						kind: TypeKind::Function(func_type),
						qual: TypeQual::default(),
					},
					linkage,
					storage,
					span: decl.ident.to_span(),
					is_decl: false,
				};
				let key = decl.ident.name.clone();
				if let Err(sym::SymbolTableError::AlreadyExists(prev_entry)) =
					self.ordinary_table.insert(key.clone(), new_entry.clone())
				{
					let kind = DiagKind::SymbolAlreadyExists(
						decl.ident.name.clone(),
						prev_entry.data_type.clone(),
					);
					let mut error = Diagnostic::error(kind, prev_entry.to_span());
					error.push_span(
						new_entry.span,
						&format!("`{}` redefined here", decl.ident.name.clone()),
					);
					if prev_entry.is_decl == false && new_entry.is_decl == false {
						// redefinition. don't even need to check types
						self.diagnostics.push(error);
					} else {
						// TODO: further type checking is required.
					}
				}
				self.tree_builder.end_child();
			}
			Some(syn::Declarator::ParamList(param_list)) => {
				if param_list.param_list.len() > 127 && self.warn_lvl == WarnLevel::All {
					// 5.2.4.1 translation limit
					let diag = Diagnostic::warn(DiagKind::ParameterLimit, decl.ident.to_span());
					self.diagnostics.push(diag);
				}
				let Some(symbol_list) = self.param_list(param_list, DeclType::FnDef) else {
					// failed to get param types
					return false;
				};

				declaration_list = symbol_list;

				let func_type = FuncType {
					params: declaration_list.iter().map(|s| s.1.clone()).collect(),
					ret: Box::new(data_type.clone()),
					is_variadic: param_list.is_variadic,
					is_inline: !decl.specifiers.inline_list.is_empty(),
				};
				let new_entry = sym::SymbolTableEntry {
					data_type: DataType {
						kind: TypeKind::Function(func_type),
						qual: TypeQual::default(),
					},
					linkage,
					storage,
					span: func_ident.to_span(),
					is_decl: false,
				};
				let key = decl.ident.name.clone();
				if let Err(sym::SymbolTableError::AlreadyExists(prev_entry)) =
					self.ordinary_table.insert(key.clone(), new_entry.clone())
				{
					let kind = DiagKind::SymbolAlreadyExists(
						decl.ident.name.clone(),
						prev_entry.data_type.clone(),
					);
					let mut error = Diagnostic::error(kind, prev_entry.to_span());
					error.push_span(
						new_entry.span,
						&format!("`{}` redefined here", func_ident.name.clone()),
					);
					if prev_entry.is_decl == false {
						error.push_note(&format!(
							"`{}` must be defined only once in the ordinary namespace of this translation unit",
							func_ident.name.clone()
						));
						// redefinition. don't even need to check types
						self.diagnostics.push(error);
					} else {
						let callee_span = prev_entry.to_span();
						if !self.dtype_eq(&prev_entry.data_type, &new_entry.data_type, callee_span)
						{
							self.diagnostics.push(error);
						}
					}
				}
			}
			Some(syn::Declarator::Array(array)) => {
				let kind = DiagKind::ArrayOfFunctions {
					name: Some(decl.ident.name.clone()),
					dtype: data_type,
				};
				let diag = Diagnostic::error(kind, array.span.clone());
				self.diagnostics.push(diag);
				self.tree_builder.end_child();
				return false;
			}
			token @ (None | Some(syn::Declarator::Pointer(_))) => {
				let kind = DiagKind::UnrecognizedToken {
					token: format!("{token:?}"),
					expected: vec![
						"\"=\"".to_string(),
						"\",\"".to_string(),
						"\";\"".to_string(),
						"\"asm\"".to_string(),
					],
				};
				let diag = Diagnostic::error(kind, decl.compound_stmt.lcurly.clone());
				self.diagnostics.push(diag);
				return false;
			}
		}

		self.increase_scope();
		self.label_table.increase_scope();

		for (decl_maybe, decl_type, type_span) in declaration_list.iter() {
			let Some(decl_ident) = decl_maybe else {
				// missing parameter name
				continue;
			};
			let new_entry = sym::SymbolTableEntry {
				data_type: decl_type.clone(),
				linkage: sym::Linkage::Internal,
				storage: sym::StorageClass::Automatic,
				span: decl_ident.to_span(),
				is_decl: false,
			};
			let key = decl_ident.name.clone();
			if let Err(sym::SymbolTableError::AlreadyExists(prev_entry)) =
				self.ordinary_table.insert(key.clone(), new_entry.clone())
			{
				let kind = DiagKind::SymbolAlreadyExists(
					decl_ident.name.clone(),
					prev_entry.data_type.clone(),
				);
				let mut error = Diagnostic::error(kind, prev_entry.to_span());
				error.push_span(
					new_entry.span,
					&format!("`{}` redefined here", decl_ident.name.clone()),
				);
				if prev_entry.is_decl == false && new_entry.is_decl == false {
					// redefinition. don't even need to check types
					self.diagnostics.push(error);
				} else {
					// TODO: further type checking is required.
					todo!()
				}
			}

			self.declare_tag(decl_type, type_span.clone());
		}

		if decl.declaration_list.len() > 0 {
			self.tree_builder
				.begin_child("declaration-list".to_string());
		}
		for declaration in decl.declaration_list.iter_mut() {
			self.declaration(declaration, syn::StorageClass::Auto, true);
		}
		if decl.declaration_list.len() > 0 {
			self.tree_builder.end_child();
		}
		self.tree_builder
			.begin_child("compound-stmt { }".to_string());
		for item in decl.compound_stmt.blocks.iter_mut() {
			self.block_item(item);
		}
		self.tree_builder.end_child();

		if let (Ok(sc), Ok(layout)) = (
			ssa::StorageClass::try_from(storage),
			icg::DataLayout::try_from(data_type.kind),
		) {
			self.data_layouts.as_mut().map(|h| h.insert(layout.clone()));
			decl.specifiers.storage = Some(sc);
			decl.specifiers.layout = Some(layout);
		}

		self.label_table.decrease_scope();
		self.decrease_scope();
		self.tree_builder.end_child();
		return true;
	}

	pub(super) fn param_list(
		&mut self,
		param_list: &mut syn::ParamList,
		decl_type: DeclType,
	) -> Option<Vec<(Option<syn::Identifier>, DataType, Span)>> {
		self.tree_builder.begin_child("param-list ( )".to_string());
		let param_count = param_list.param_list.len();
		let mut result = vec![];
		let mut is_valid = true;
		for (index, param) in param_list.param_list.iter_mut().enumerate() {
			let name_opt = param.ident.as_ref().and_then(|v| Some(v.name.clone()));
			let param_span = match &param.ident {
				Some(ident) => ident.to_span(),
				None => param.specifiers.to_span(),
			};
			let maybe_type = self.specifiers_dtype(&mut param.specifiers, true);
			let mut param_type =
				self.unwrap_or_poison(maybe_type, name_opt.clone(), param_span.clone());
			self.tree_builder.add_empty_child(format!(
				"`{}` '{}'",
				name_opt.clone().unwrap_or("<anonymous>".to_string()),
				param_type
			));
			if let TypeKind::Poison = param_type.kind {
				continue;
			}
			match (param.ident.as_ref(), &param_type.kind) {
				(None, TypeKind::Void) => match param.declarators.front() {
					Some(syn::Declarator::Array(syn::ArrayDecl { span, .. })) => {
						let kind = DiagKind::ArrayOfVoid(None);
						let diag = Diagnostic::error(kind, span.clone());
						self.diagnostics.push(diag);
						is_valid = false;
					}
					Some(syn::Declarator::Pointer(_)) => {
						if decl_type == DeclType::FnDef {
							let kind = DiagKind::OmittedParamName;
							let diag = Diagnostic::error(kind, param.specifiers.to_span());
							self.diagnostics.push(diag);
							is_valid = false;
						}
					}
					Some(syn::Declarator::ParamList(_)) => {
						if decl_type == DeclType::FnDef {
							let kind = DiagKind::OmittedParamName;
							let diag = Diagnostic::error(kind, param.specifiers.to_span());
							self.diagnostics.push(diag);
							is_valid = false;
						}
						let implicit = syn::Declarator::Pointer(syn::PtrDecl {
							is_const: false,
							is_volatile: false,
							is_restrict: false,
						});
						param.declarators.push_front(implicit);
					}
					Some(syn::Declarator::IdentList(_)) => {
						let kind = DiagKind::DeclIdentList;
						let diag = Diagnostic::error(kind, param.specifiers.to_span());
						self.diagnostics.push(diag);
						is_valid = false;
					}
					None => {
						if param_count > 1 {
							let kind = DiagKind::OnlyVoid;
							let diag = Diagnostic::error(kind, param.specifiers.to_span());
							self.diagnostics.push(diag);
							is_valid = false;
						}
					}
				},
				(Some(ident), TypeKind::Void) => match param.declarators.front() {
					Some(syn::Declarator::Array(syn::ArrayDecl { span, .. })) => {
						let kind = DiagKind::ArrayOfVoid(Some(ident.name.clone()));
						let diag = Diagnostic::error(kind, ident.to_span());
						self.diagnostics.push(diag);
						is_valid = false;
					}
					Some(syn::Declarator::IdentList(_)) => {
						let kind = DiagKind::DeclIdentList;
						let diag = Diagnostic::error(kind, ident.to_span());
						self.diagnostics.push(diag);
						is_valid = false;
					}
					Some(syn::Declarator::ParamList(_)) => {
						let implicit = syn::Declarator::Pointer(syn::PtrDecl {
							is_const: false,
							is_volatile: false,
							is_restrict: false,
						});
						param.declarators.push_front(implicit);
					}
					Some(syn::Declarator::Pointer(_)) => {}
					None => {
						let kind = DiagKind::OnlyVoid;
						let diag = Diagnostic::error(kind, ident.to_span());
						self.diagnostics.push(diag);
						is_valid = false;
					}
				},
				(None, _) => {
					if decl_type == DeclType::FnDef {
						let kind = DiagKind::OmittedParamName;
						let diag = Diagnostic::error(kind, param.specifiers.to_span());
						self.diagnostics.push(diag);
						is_valid = false;
					}
				}
				(Some(ident), _) => {}
			}
			self.declarator_list(
				param_span.to_span(),
				param.declarators.make_contiguous(),
				&mut param_type,
				true,
				decl_type,
				name_opt.clone(),
				vec![],
			);
			if let Ok(layout) = icg::DataLayout::try_from(param_type.kind.clone()) {
				self.data_layouts.as_mut().map(|h| h.insert(layout.clone()));
				param.specifiers.storage = Some(ssa::StorageClass::Automatic);
				param.specifiers.layout = Some(layout);
			}
			result.push((param.ident.clone(), param_type, param.specifiers.to_span()))
		}
		self.tree_builder.end_child();
		match is_valid {
			true => Some(result),
			false => None,
		}
	}
}
