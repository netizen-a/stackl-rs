use crate::analysis::syn;
use crate::analysis::tok::Const;
use crate::data_type::*;
use crate::diagnostics::*;
use crate::symbol_table as sym;

type CastScore = usize;

impl super::SemanticParser {
	pub(super) fn declare_tag(&mut self, data_type: &DataType, span: Span) {
		let name = match &data_type.kind {
			TypeKind::Tag(TagKind::DeclStruct(name, _)) => name,
			TypeKind::Tag(TagKind::DeclUnion(name, _)) => name,
			TypeKind::Tag(TagKind::DeclEnum(name, enumerator_list)) => {
				for (const_ident, const_val) in enumerator_list.iter() {
					let const_type = DataType {
						kind: TypeKind::EnumConst(EnumConst {
							tag_name: name.clone(),
							value: *const_val,
						}),
						qual: TypeQual::default(),
					};
					let new_entry = sym::SymbolTableEntry {
						data_type: const_type,
						is_decl: true,
						linkage: sym::Linkage::Internal,
						span: const_ident.to_span(),
						storage: sym::StorageClass::Constant,
					};

					if let Err(sym::SymbolTableError::AlreadyExists(prev_entry)) = self
						.ordinary_table
						.insert(const_ident.name.clone(), new_entry.clone())
					{
						let kind = DiagKind::SymbolAlreadyExists(
							const_ident.name.clone(),
							prev_entry.data_type.clone(),
						);
						let mut error = Diagnostic::error(kind, prev_entry.to_span());
						error.push_span(
							new_entry.span,
							&format!("`{}` redefined here", const_ident.name),
						);
						if prev_entry.is_decl == false && new_entry.is_decl == false {
							if prev_entry.data_type.kind.is_incomplete() {}
							self.diagnostics.push(error);
						} else {
							// TODO: further type checking is required.
						}
					}
				}
				name
			}
			TypeKind::Pointer(inner) => {
				self.declare_tag(inner, span);
				return;
			}
			_ => {
				return;
			}
		};

		let new_entry = sym::SymbolTableEntry {
			data_type: data_type.clone(),
			is_decl: true,
			linkage: sym::Linkage::Internal,
			span: span.clone(),
			storage: sym::StorageClass::Typename,
		};

		if let Err(sym::SymbolTableError::AlreadyExists(prev_entry)) =
			self.tag_table.insert(name.clone(), new_entry.clone())
		{
			let kind = DiagKind::SymbolAlreadyExists(name.clone(), prev_entry.data_type.clone());
			let mut error = Diagnostic::error(kind, prev_entry.to_span());
			error.push_span(new_entry.span, &format!("`{name}` redefined here"));
			if prev_entry.is_decl == false && new_entry.is_decl == false {
				if prev_entry.data_type.kind.is_incomplete() {}
				self.diagnostics.push(error);
			} else {
				// TODO: further type checking is required.
			}
		}
		let identifier = syn::Identifier {
			name: name.to_string(),
			span,
		};
		self.declare_members(vec![identifier.clone()], data_type);
	}

	pub fn declare_members(&mut self, decl_ident: Vec<syn::Identifier>, decl_type: &DataType) {
		if let TypeKind::Tag(tag_kind) = &decl_type.kind {
			let member_type_list: &Vec<MemberType> = match tag_kind {
				TagKind::AnonStruct(inner) => inner,
				TagKind::AnonUnion(inner) => inner,
				TagKind::DeclStruct(_, inner) => inner,
				TagKind::DeclUnion(_, inner) => inner,
				_ => {
					// don't care
					return;
				}
			};
			for member_type in member_type_list.iter() {
				let Some(member_ident) = &member_type.ident else {
					continue;
				};

				let mut ident_list = decl_ident.clone();
				ident_list.push(member_ident.clone());

				let key: Vec<String> = ident_list.iter().map(|ident| ident.name.clone()).collect();

				let new_entry = sym::SymbolTableEntry {
					data_type: *member_type.dtype.clone(),
					is_decl: true,
					linkage: sym::Linkage::Internal,
					span: member_ident.to_span(),
					storage: sym::StorageClass::Typename,
				};

				if let Err(sym::SymbolTableError::AlreadyExists(prev_entry)) =
					self.member_table.insert(key, new_entry.clone())
				{
					let kind = DiagKind::SymbolAlreadyExists(
						member_ident.name.clone(),
						prev_entry.data_type.clone(),
					);
					let mut error = Diagnostic::error(kind, prev_entry.to_span());
					error.push_span(
						new_entry.span,
						&format!("`{}` redefined here", member_ident.name),
					);
					self.diagnostics.push(error);
				}
				self.declare_members(ident_list, &member_type.dtype);
			}
		}
	}

	pub(super) fn unwrap_or_poison(
		&mut self,
		value: Option<DataType>,
		name: Option<String>,
		span: Span,
	) -> DataType {
		match value {
			Some(ty) => ty.clone(),
			None => {
				let diag = Diagnostic::error(DiagKind::ImplicitInt(name), span);
				self.diagnostics.push(diag);
				DataType {
					kind: TypeKind::Poison,
					qual: TypeQual::default(),
				}
			}
		}
	}
	pub(super) fn dtype_eq(&mut self, lhs: &DataType, rhs: &DataType, callee_span: Span) -> bool {
		match (&lhs.kind, &rhs.kind) {
			(TypeKind::Void, TypeKind::Void) => true,
			(TypeKind::Scalar(l_scalar), TypeKind::Scalar(r_scalar)) => l_scalar == r_scalar,
			(TypeKind::Pointer(l_ptr), TypeKind::Pointer(r_ptr)) => {
				self.dtype_eq(&l_ptr, &r_ptr, callee_span)
			}
			(TypeKind::Pointer(ptr), TypeKind::Array(array)) => {
				if !array.is_decayed {
					return false;
				}
				if let (true, ArrayLength::Fixed(0 | 2..)) = (array.has_static, &array.length) {
					let kind = DiagKind::ArrayArgTooSmall;
					let warning = Diagnostic::warn(kind, callee_span.to_span());
					self.diagnostics.push(warning);
				}
				self.dtype_eq(&ptr, &array.component, callee_span)
			}
			(TypeKind::Array(array), TypeKind::Pointer(ptr)) => {
				if !array.is_decayed {
					return false;
				}
				if let (true, ArrayLength::Fixed(0 | 2..)) = (array.has_static, &array.length) {
					let kind = DiagKind::ArrayArgTooSmall;
					let warning = Diagnostic::warn(kind, callee_span.to_span());
					self.diagnostics.push(warning);
				}
				self.dtype_eq(&array.component, &ptr, callee_span)
			}
			(TypeKind::Array(l_array), TypeKind::Array(r_array)) => {
				if let (ArrayLength::Fixed(l_size), ArrayLength::Fixed(r_size)) =
					(&l_array.length, &r_array.length)
				{
					if *l_size != *r_size {
						return false;
					}
				}
				self.dtype_eq(&l_array.component, &r_array.component, callee_span)
			}
			(TypeKind::Function(l_func), TypeKind::Function(r_func)) => {
				let (l_params, r_params) = (&l_func.params, &r_func.params);
				let is_params_unchecked = l_params.is_empty() || r_params.is_empty();
				if !is_params_unchecked {
					if l_params.len() != r_params.len() || l_func.is_variadic != r_func.is_variadic
					{
						return false;
					} else {
						for (l_param, r_param) in l_params.iter().zip(r_params) {
							if let false = self.dtype_eq(l_param, r_param, callee_span.clone()) {
								return false;
							}
						}
					}
				}
				self.dtype_eq(&l_func.ret, &r_func.ret, callee_span)
			}
			(TypeKind::Poison, _) | (_, TypeKind::Poison) => true,
			(_, _) => false,
		}
	}

	pub fn convert_type(
		&mut self,
		expr: &mut syn::Expr,
		from_type: &DataType,
		to_type: &DataType,
		callee_span: Span,
	) -> CastScore {
		let mut result_score = 0;

		if to_type.is_poisoned() {
			return result_score;
		}

		if self.is_l_value(expr) {
			let expr_cast = syn::ExprCast {
				span: callee_span.to_span(),
				kind: syn::CastKind::LValueToRValue,
				expr: Box::new(expr.clone()),
			};
			*expr = syn::Expr::Cast(expr_cast);
			result_score += 1;
		}

		if self.dtype_eq(from_type, to_type, callee_span.to_span()) {
			return result_score;
		} else {
			match (&from_type.kind, &to_type.kind) {
				(TypeKind::Scalar(from_scalar), TypeKind::Scalar(to_scalar)) => {
					result_score +=
						self.convert_scalar(expr, *from_scalar, *to_scalar, callee_span.to_span());
				}
				_ => {
					let kind = DiagKind::CastError {
						from_type: from_type.clone(),
						to_type: to_type.clone(),
					};
					let error = Diagnostic::error(kind, callee_span.to_span());
					self.diagnostics.push(error);
					result_score = 0;
				}
			}
		}

		result_score
	}

	fn convert_scalar(
		&mut self,
		expr: &mut syn::Expr,
		from_scalar: ScalarType,
		mut to_scalar: ScalarType,
		callee_span: Span,
	) -> CastScore {
		let mut result_score = 0;

		use ScalarType::*;

		if to_scalar.bits() < 32 {
			match to_scalar.is_signed() {
				Some(true) => to_scalar = ScalarType::I32,
				Some(false) => to_scalar = ScalarType::U32,
				None => {}
			}
		}

		let (from_bits, to_bits) = (from_scalar.bits(), to_scalar.bits());

		let to_kind = Box::new(TypeKind::Scalar(to_scalar.clone()));

		let cast_kind = match (from_scalar.is_signed(), to_scalar.is_signed()) {
			(Some(true), Some(true)) if from_bits < to_bits => Some(syn::CastKind::SExt(to_kind)),
			(Some(false), Some(false)) if from_bits < to_bits => Some(syn::CastKind::ZExt(to_kind)),
			((Some(true), Some(true)) | (Some(false), Some(false))) if from_bits > to_bits => {
				Some(syn::CastKind::Trunc(to_kind))
			}
			(Some(true), None) if to_scalar.is_floating() => {
				result_score += 1;
				Some(syn::CastKind::SIToFP(to_kind))
			}
			(Some(false), None) if to_scalar.is_floating() => {
				result_score += 1;
				Some(syn::CastKind::UIToFP(to_kind))
			}
			(None, Some(true)) if from_scalar.is_floating() => {
				result_score += 2;
				Some(syn::CastKind::FPToSI(to_kind))
			}
			(None, Some(false)) if from_scalar.is_floating() => {
				result_score += 2;
				Some(syn::CastKind::FPToUI(to_kind))
			}
			(Some(_), None) if to_scalar == ScalarType::Bool => Some(syn::CastKind::IntToBool),
			(None, Some(_)) if from_scalar == ScalarType::Bool => {
				Some(syn::CastKind::ZExt(to_kind))
			}
			(None, None) if from_scalar.is_floating() && to_scalar.is_floating() => {
				if from_bits < to_bits {
					Some(syn::CastKind::FpExt(to_kind))
				} else if from_bits > to_bits {
					Some(syn::CastKind::FpTrunc(to_kind))
				} else {
					None
				}
			}
			_ => None,
		};

		if let Some(kind) = cast_kind {
			let expr_cast = syn::ExprCast {
				span: callee_span.to_span(),
				kind,
				expr: Box::new(expr.clone()),
			};
			*expr = syn::Expr::Cast(expr_cast);
		}

		result_score
	}
}
