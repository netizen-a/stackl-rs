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
	},
};
use std::mem;

impl super::SSACodeGen<'_> {
	pub(super) fn expr(&mut self, expr: &syn::Expr) -> (u32, DataLayout) {
		match expr {
			syn::Expr::Const(inner) => self.constant(inner),
			syn::Expr::Binary(inner) => self.binary(inner),
			_ => todo!(),
		}
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
	pub(super) fn binary(&mut self, expr: &syn::ExprBinary) -> (u32, DataLayout) {
		let lhs = self.expr(&expr.left);
		let rhs = self.expr(&expr.right);
		assert!(lhs.1 == rhs.1);
		let result_type = self.resolve_type(&lhs.1);
		let result_id = match (&lhs.1, &expr.op.kind) {
			(DataLayout::Integer(IntegerLayout { width: _, is_signed: true }), syn::expr::BinOpKind::Add) => {
				self.builder.i_add(result_type, lhs.0, rhs.0).unwrap()
			}
			_ => {
				todo!()
			}
		};
		(result_id, lhs.1)
	}
}
