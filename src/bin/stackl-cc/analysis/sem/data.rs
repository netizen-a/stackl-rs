use crate::analysis::syn::Expr;
use crate::analysis::tok::Const;
use crate::data_type::*;
use crate::diagnostics::*;
use crate::symbol_table as sym;

pub enum CastScore {
	NoOperation = 0,
	IntegralToFloating = 1,
	FloatingToIntegral = 2,
}

pub enum ValueCategory {
	RValue,
	LValue,
}

impl super::SemanticParser {
	// TODO: change name to `declare_tag`
	pub(super) fn declare_tag(&mut self, data_type: &DataType, span: Span) {
		let (name, tag_kind) = match &data_type.kind {
			TypeKind::Tag(tag_kind @ TagKind::DeclStruct(name, _)) => (name, tag_kind),
			TypeKind::Tag(tag_kind @ TagKind::DeclUnion(name, _)) => (name, tag_kind),
			TypeKind::Tag(tag_kind @ TagKind::DeclEnum(name, _)) => (name, tag_kind),
			TypeKind::Pointer(inner) => {
				self.declare_tag(inner, span);
				return;
			}
			other => {
				return;
			}
		};

		let new_entry = sym::SymbolTableEntry {
			data_type: data_type.clone(),
			is_decl: true,
			linkage: sym::Linkage::Internal,
			span,
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
	pub(super) fn dtype_eq(
		&mut self,
		lhs: &DataType,
		rhs: &DataType,
		callee_span: Span,
	) -> Result<bool, DataType> {
		match (&lhs.kind, &rhs.kind) {
			(TypeKind::Void, TypeKind::Void) => Ok(true),
			(TypeKind::Scalar(l_scalar), TypeKind::Scalar(r_scalar)) => Ok(l_scalar == r_scalar),
			(TypeKind::Pointer(l_ptr), TypeKind::Pointer(r_ptr)) => {
				self.dtype_eq(&l_ptr, &r_ptr, callee_span)
			}
			(TypeKind::Pointer(ptr), TypeKind::Array(array)) => {
				if !array.is_decayed {
					return Ok(false);
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
					return Ok(false);
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
						return Ok(false);
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
						return Ok(false);
					} else {
						for (l_param, r_param) in l_params.iter().zip(r_params) {
							if let Ok(false) = self.dtype_eq(l_param, r_param, callee_span.clone())
							{
								return Ok(false);
							}
						}
					}
				}
				self.dtype_eq(&l_func.ret, &r_func.ret, callee_span)
			}
			(TypeKind::Poison, _) | (_, TypeKind::Poison) => Err(DataType::POISON),
			(_, _) => Ok(false),
		}
	}

	// code gen here
	pub fn try_convert(&mut self, from: &Expr, to: DataType) -> Option<(CastScore, Expr)> {
		todo!()
	}
}
