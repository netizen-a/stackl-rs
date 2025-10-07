use crate::analysis::syn::Expr;
use crate::data_types::*;
use crate::diagnostics as diag;
use crate::diagnostics::Span;
use diag::ToSpan;

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
	pub(super) fn unwrap_or_poison(
		&mut self,
		value: Option<DataType>,
		name: Option<String>,
		span: diag::Span,
	) -> DataType {
		match value {
			Some(ty) => ty.clone(),
			None => {
				let diag = diag::Diagnostic::error(diag::DiagKind::ImplicitInt(name), span);
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
			(TypeKind::Scalar(l_scalar), TypeKind::Scalar(r_scalar)) => Ok(l_scalar == r_scalar),
			(TypeKind::Pointer(l_ptr), TypeKind::Pointer(r_ptr)) => {
				self.dtype_eq(&l_ptr, &r_ptr, callee_span)
			}
			(TypeKind::Pointer(ptr), TypeKind::Array(array)) => {
				if !array.is_decayed {
					return Ok(false);
				}
				if let (true, ArrayLength::Fixed(0 | 2..)) = (array.has_static, &array.length) {
					let kind = diag::DiagKind::ArrayArgTooSmall;
					let warning = diag::Diagnostic::warn(kind, callee_span.to_span());
					self.diagnostics.push(warning);
				}
				self.dtype_eq(&ptr, &array.component, callee_span)
			}
			(TypeKind::Array(array), TypeKind::Pointer(ptr)) => {
				if !array.is_decayed {
					return Ok(false);
				}
				if let (true, ArrayLength::Fixed(0 | 2..)) = (array.has_static, &array.length) {
					let kind = diag::DiagKind::ArrayArgTooSmall;
					let warning = diag::Diagnostic::warn(kind, callee_span.to_span());
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
				todo!()
			}
			(TypeKind::Poison, _) | (_, TypeKind::Poison) => Err(DataType::POISON),
			(_, _) => todo!(),
		}
	}

	// code gen here
	pub fn try_convert(&mut self, from: &Expr, to: DataType) -> Option<(CastScore, Expr)> {
		todo!()
	}
}
