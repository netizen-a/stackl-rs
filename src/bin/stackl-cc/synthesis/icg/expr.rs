// Copyright (c) 2024-2026 Jonathan A. Thomason

use crate::{
	analysis::syn::{
		self,
		CharKind,
		FloatingKind,
		IntegerKind,
	},
	synthesis::icg::{
		DataLayout,
		FloatLayout,
		IntegerLayout,
		PtrLayout,
	},
};
use std::mem;

impl super::SSACodeGen<'_> {
	pub(super) fn expr(&mut self, expr: &syn::Expr) -> (u32, DataLayout) {
		match expr {
			syn::Expr::Const(inner) => self.constant(inner),
			syn::Expr::Binary(inner) => self.binary(inner),
			syn::Expr::UnaryPrefix(inner) => self.unary_prefix(inner),
			syn::Expr::UnaryPostfix(inner) => self.unary_postfix(inner),
			syn::Expr::Ternary(inner) => self.ternary(inner),
			_ => todo!(),
		}
	}

	pub(super) fn binary(&mut self, expr: &syn::ExprBinary) -> (u32, DataLayout) {
		let lhs = self.expr(&expr.left);
		let rhs = self.expr(&expr.right);
		assert!(lhs.1 == rhs.1);
		let result_type = self.resolve_type(&lhs.1);
		let result_id = match (&lhs.1, &expr.op.kind) {
			(_, syn::expr::BinOpKind::Assign) => {
				let (id, _layout) = self.assign(&expr.left, &expr.right);
				id
			}
			(DataLayout::Integer(IntegerLayout { width: 32, .. }), syn::expr::BinOpKind::Add) => {
				self.builder.i_add(result_type, lhs.0, rhs.0).unwrap()
			}
			(DataLayout::Float(FloatLayout { width: _ }), syn::expr::BinOpKind::Add) => {
				self.builder.f_add(result_type, lhs.0, rhs.0).unwrap()
			}
			(DataLayout::Integer(IntegerLayout { width: 32, .. }), syn::expr::BinOpKind::Sub) => {
				self.builder.i_sub(result_type, lhs.0, rhs.0).unwrap()
			}
			(DataLayout::Float(FloatLayout { width: _ }), syn::expr::BinOpKind::Sub) => {
				self.builder.f_sub(result_type, lhs.0, rhs.0).unwrap()
			}
			(DataLayout::Integer(IntegerLayout { width: 32, .. }), syn::expr::BinOpKind::Mul) => {
				self.builder.i_mul(result_type, lhs.0, rhs.0).unwrap()
			}
			(DataLayout::Float(_), syn::expr::BinOpKind::Mul) => {
				self.builder.f_mul(result_type, lhs.0, rhs.0).unwrap()
			}
			(
				DataLayout::Integer(IntegerLayout {
					width: 32,
					is_signed: true,
				}),
				syn::expr::BinOpKind::Div,
			) => self.builder.s_div(result_type, lhs.0, rhs.0).unwrap(),
			(
				DataLayout::Integer(IntegerLayout {
					width: 32,
					is_signed: false,
				}),
				syn::expr::BinOpKind::Div,
			) => self.builder.u_div(result_type, lhs.0, rhs.0).unwrap(),
			(DataLayout::Float(_), syn::expr::BinOpKind::Div) => {
				self.builder.f_div(result_type, lhs.0, rhs.0).unwrap()
			}
			(
				DataLayout::Integer(IntegerLayout {
					width: 32,
					is_signed: true,
				}),
				syn::expr::BinOpKind::Rem,
			) => self.builder.s_rem(result_type, lhs.0, rhs.0).unwrap(),
			(
				DataLayout::Integer(IntegerLayout {
					width: 32,
					is_signed: false,
				}),
				syn::expr::BinOpKind::Rem,
			) => self.builder.u_rem(result_type, lhs.0, rhs.0).unwrap(),
			(DataLayout::Float(_), syn::expr::BinOpKind::Rem) => {
				self.builder.f_rem(result_type, lhs.0, rhs.0).unwrap()
			}
			_ => {
				todo!()
			}
		};
		(result_id, lhs.1)
	}

	pub(super) fn assign(&mut self, lhs: &syn::Expr, rhs: &syn::Expr) -> (u32, DataLayout) {
		let (rhs_id, rhs_layout) = self.expr(rhs);
		let (lhs_id, lhs_layout) = self.expr(lhs);

		// TODO: One of the asserts is expected to fail. Fix this later.
		assert!(
			matches!(lhs_layout, DataLayout::Pointer(_)),
			"Assignment: left-hand side must be a pointer"
		);
		assert!(
			!matches!(rhs_layout, DataLayout::Pointer(_)),
			"Assignment: right-hand side must not be a pointer (object only)"
		);

		self.builder.store(lhs_id, rhs_id);
		(lhs_id, lhs_layout)
	}

	pub(super) fn constant(&mut self, constant: &syn::Constant) -> (u32, DataLayout) {
		match &constant.kind {
			&syn::ConstantKind::Integer(IntegerKind::U32(num)) => {
				let layout = DataLayout::Integer(IntegerLayout {
					width: 32,
					is_signed: false,
				});
				let result_type = self.resolve_type(&layout);
				let id = self.builder.constant_bit32(result_type, num);
				(id, layout)
			}
			&syn::ConstantKind::Integer(IntegerKind::I32(num)) => {
				let layout = DataLayout::Integer(IntegerLayout {
					width: 32,
					is_signed: true,
				});
				let result_type = self.resolve_type(&layout);
				let id = unsafe {
					self.builder
						.constant_bit32(result_type, mem::transmute(num))
				};
				(id, layout)
			}
			&syn::ConstantKind::Floating(FloatingKind::Float(num)) => {
				let layout = DataLayout::Float(FloatLayout { width: 32 });
				let result_type = self.resolve_type(&layout);
				let id = unsafe {
					self.builder
						.constant_bit32(result_type, mem::transmute(num))
				};
				(id, layout)
			}
			other => todo!("{other:?}"),
		}
	}

	pub(super) fn unary_prefix(&mut self, expr: &syn::UnaryPrefix) -> (u32, DataLayout) {
		match &expr.op.kind {
			syn::PrefixKind::Plus => todo!("unary plus: return value unchanged"),
			syn::PrefixKind::Minus => todo!("unary minus: negate value"),
			syn::PrefixKind::Comp => todo!("unary bitwise complement"),
			syn::PrefixKind::Star => todo!("unary dereference"),
			syn::PrefixKind::Amp => todo!("unary address-of"),
			syn::PrefixKind::Sizeof => todo!("unary sizeof"),
			syn::PrefixKind::Neg => todo!("unary logical not"),
			_ => todo!("unary operator"),
		}
	}

	pub(super) fn unary_postfix(&mut self, expr: &syn::UnaryPostfix) -> (u32, DataLayout) {
		let inner = self.expr(&expr.expr);
		let result_type = inner.1;
		let result_id = match &expr.op.kind {
			&syn::PostfixKind::Inc => todo!("postfix increment"),
			&syn::PostfixKind::Dec => todo!("postfix decrement"),
			&syn::PostfixKind::Array(_) => todo!("array indexing"),
			&syn::PostfixKind::Dot(_) => todo!("struct member access"),
			&syn::PostfixKind::Arrow(_) => todo!("struct pointer member access"),
			&syn::PostfixKind::ArgExprList(_) => todo!("function call"),
		};
		(result_id, result_type)
	}

	pub(super) fn ternary(&mut self, expr: &syn::ExprTernary) -> (u32, DataLayout) {
		let (cond_id, cond_layout) = self.expr(&expr.expr_cond);
		let then_label_id = self.builder.id();
		let else_label_id = self.builder.id();
		let after_label_id = self.builder.id();

		self.builder.label(then_label_id).unwrap();
		self.builder
			.branch_conditional(cond_id, then_label_id, else_label_id)
			.unwrap();

		let then_result = self.expr(&expr.expr_then);
		self.builder.branch(after_label_id).unwrap();

		self.builder.label(else_label_id).unwrap();
		let else_result = self.expr(&expr.expr_else);
		self.builder.label(after_label_id).unwrap();

		let result_type = self.resolve_type(&then_result.1);
		let result_id = self
			.builder
			.phi(
				result_type,
				[
					(then_result.0, then_label_id),
					(else_result.0, else_label_id),
				],
			)
			.unwrap();

		assert!(
			then_result.1 == else_result.1,
			"DataLayout must be the same for both blocks"
		);
		(result_id, then_result.1)
	}
}
