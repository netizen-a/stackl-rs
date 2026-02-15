// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::analysis::syn;
use crate::analysis::tok::Const;
use crate::data_type::*;
use crate::diagnostics::*;
use crate::symtab as sym;

type CastScore = usize;

impl super::SemanticParser<'_> {
	pub(super) fn declare_tag(&mut self, data_type: &DataType, span: Span) {
		let name = match &data_type.kind {
			TypeKind::Tag(TagKind::Struct(Some(name), _)) => name,
			TypeKind::Tag(TagKind::Union(Some(name), _)) => name,
			TypeKind::Tag(TagKind::Enum(Some(name), enumerator_list)) => {
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

		// Add struct type as parent node
		if self.print_ast {
			let (_, reported_line, col) = self.diagnostics.get_location(&span).unwrap();
			let type_name = match &data_type.kind {
				TypeKind::Tag(TagKind::Struct(Some(n), _)) => format!("struct {}", n),
				TypeKind::Tag(TagKind::Union(Some(n), _)) => format!("union {}", n),
				_ => format!("struct {}", name),
			};
			self.tree_builder.add_empty_child(format!(
				"declarator <line:{reported_line}, col:{col}> '{type_name}'",
				type_name = type_name
			));
		}

		let identifier = syn::Identifier {
			name: name.to_string(),
			span,
		};
		self.declare_members(vec![identifier], data_type);
	}

	pub fn declare_members(&mut self, decl_ident: Vec<syn::Identifier>, decl_type: &DataType) {
		if let TypeKind::Tag(tag_kind) = &decl_type.kind {
			let member_type_list: &Vec<MemberType> = match tag_kind {
				TagKind::Struct(_, inner) => inner,
				TagKind::Union(_, inner) => inner,
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

				let (_, reported_line, col) = self
					.diagnostics
					.get_location(&member_ident.to_span())
					.unwrap();

				// Add tree node for member display
				if self.print_ast {
					// Handle struct member (e.g., struct Bar bar)
					if let TypeKind::Tag(TagKind::Struct(Some(tag_name), _)) = &member_type.dtype.kind {
						// Check if this member is already declared in the current scope
						let parent_path: Vec<String> = ident_list.iter().map(|i| i.name.clone()).collect();
						if !self.member_already_declared(member_ident, &parent_path) {
							let member_name = ident_list.last().unwrap().name.clone();
							let member_type_str = format!("struct {}", tag_name);
							self.tree_builder.begin_child(format!(
								"declarator <line:{reported_line}, col:{col}> `{member_name}` '{member_type_str}'",
								member_name = member_name,
								member_type_str = member_type_str
							));
							self.declare_members(ident_list.clone(), &member_type.dtype);
							self.tree_builder.end_child();
						}
					}
					// Handle anonymous struct member (e.g., struct {int k, v} foobar)
					else if let TypeKind::Tag(TagKind::Struct(None, _)) = &member_type.dtype.kind {
						// Check if this member is already declared in the current scope
						let parent_path: Vec<String> = ident_list.iter().map(|i| i.name.clone()).collect();
						if !self.member_already_declared(member_ident, &parent_path) {
							let member_name = ident_list.last().unwrap().name.clone();
							let member_type_name = format!("struct <anonymous>");
							self.tree_builder.begin_child(format!(
								"declarator <line:{reported_line}, col:{col}> `{member_name}` '{member_type_name}'",
								member_name = member_name
							));
							self.declare_members(ident_list.clone(), &member_type.dtype);
							self.tree_builder.end_child();
						}
					}
					// Handle regular member (e.g., int x, int y)
					else {
						let member_name = ident_list.last().unwrap().name.clone();
						let member_type_str = format!("{}", member_type.dtype);
						self.tree_builder.add_empty_child(format!(
							"declarator <line:{reported_line}, col:{col}> `{member_name}` '{member_type_str}'",
							member_name = member_name,
							member_type_str = member_type_str
						));
					}
				}
			}
		}
	}

	// Helper function to check if a member has already been declared
	fn member_already_declared(&self, ident: &syn::Identifier, parent_path: &[String]) -> bool {
		// Get the full path for the member
		let full_path: Vec<String> = parent_path.iter().cloned().chain(vec![ident.name.clone()]).collect();

		// Check if this member has been declared in the current scope
		for (name, entry) in self.ordinary_table.iter_current_scope().unwrap() {
			if matches!(entry.data_type.kind, TypeKind::Tag(_)) && full_path == vec![name.clone()] {
				return true;
			}
		}

		false
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
				Some(true) => to_scalar = ScalarType::SInt,
				Some(false) => to_scalar = ScalarType::UInt,
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
