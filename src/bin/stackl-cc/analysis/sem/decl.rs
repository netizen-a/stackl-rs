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
pub enum DeclType {
	Proto,
	FnDef,
	Decl,
}

impl super::SemanticParser<'_> {
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

	pub(super) fn declarator_list(
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
					let array_length = if let Some(assign_expr) = &mut array.assignment_expr {
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
							Err(ConversionError::Expr(expr)) => dtype::ArrayLength::VLA(dtype::VlaLength::Expr(expr)),
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
