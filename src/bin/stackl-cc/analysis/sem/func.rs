use crate::analysis::sem::{DeclType, Namespace, SymbolTableEntry};
use crate::analysis::{sem::Linkage, syn};
use crate::diagnostics::{self as diag, ToSpan};
use crate::{data_types as dtype, WarnLevel};

impl super::SemanticParser<'_> {
    pub(super) fn function_definition(&mut self, decl: &mut syn::FunctionDefinition) -> bool {
		let maybe_ty = self.specifiers_dtype(&mut decl.specifiers);
		let maybe_sc = self.specifiers_storage(&mut decl.specifiers);
		let func_ident = &decl.ident;

		let (storage, linkage) = match &maybe_sc {
			None
			| Some(syn::StorageClassSpecifier {
				kind: syn::StorageClass::Extern,
				..
			}) => (syn::StorageClass::Extern, Linkage::External),
			Some(syn::StorageClassSpecifier {
				kind: syn::StorageClass::Static,
				..
			}) => (syn::StorageClass::Static, Linkage::Internal),
			Some(storage) => {
				let kind = diag::DiagKind::IllegalStorage(storage.kind);
				let diag = diag::Diagnostic::error(kind, storage.to_span());
				self.diagnostics.push(diag);
				return false;
			}
		};
		let mut data_type = self.unwrap_or_poison(maybe_ty, func_ident.clone());
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
				None,
			);
		}
		match decl.declarators.first_mut() {
			Some(syn::Declarator::IdentList(ident_list)) => {
				let func_type = dtype::FuncType {
					params: vec![],
					ret: Box::new(data_type),
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
			Some(syn::Declarator::ParamList(param_list)) => {
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
					ret: Box::new(data_type),
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
			Some(syn::Declarator::Array(array)) => {
				let kind = diag::DiagKind::ArrayOfFunctions(decl.ident.name.clone());
				let diag = diag::Diagnostic::error(kind, array.span.clone());
				self.diagnostics.push(diag);
				return false;
			}
			None | Some(syn::Declarator::Pointer(_)) => {
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
				self.declaration(declaration, syn::StorageClass::Auto);
			}
			for item in decl.compound_stmt.blocks.iter_mut() {
				self.block_item(item);
			}
		}
		self.decrease_scope();
		return true;
	}

	pub(super) fn param_list(
		&mut self,
		param_list: &mut syn::ParamList,
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
							Some(syn::Declarator::Array(syn::ArrayDecl { span, .. })) => {
								let kind = diag::DiagKind::ArrayOfVoid(None);
								let diag = diag::Diagnostic::error(kind, span.clone());
								self.diagnostics.push(diag);
								is_valid = false;
								span.clone()
							}
							Some(syn::Declarator::Pointer(_)) => {
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
							Some(syn::Declarator::ParamList(_)) => {
								if decl_type == DeclType::FnDef {
									let kind = diag::DiagKind::OmittedParamName;
									let diag = diag::Diagnostic::error(
										kind,
										param.specifiers.first_span.clone(),
									);
									self.diagnostics.push(diag);
									is_valid = false;
								}
								let implicit = syn::Declarator::Pointer(syn::PtrDecl {
									is_const: false,
									is_volatile: false,
									is_restrict: false,
								});
								param.declarators.push_front(implicit);
								param.specifiers.first_span.clone()
							}
							Some(syn::Declarator::IdentList(_)) => {
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
							Some(syn::Declarator::Array(syn::ArrayDecl { span, .. })) => {
								let kind = diag::DiagKind::ArrayOfVoid(Some(ident.name.clone()));
								let diag = diag::Diagnostic::error(kind, ident.to_span());
								self.diagnostics.push(diag);
								is_valid = false;
							}
							Some(syn::Declarator::IdentList(_)) => {
								let kind = diag::DiagKind::DeclIdentList;
								let diag = diag::Diagnostic::error(kind, ident.to_span());
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
}
