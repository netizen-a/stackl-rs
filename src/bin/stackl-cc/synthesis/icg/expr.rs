// Copyright (c) 2024-2025 Jonathan Thomason

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
	pub(super) fn expr(&mut self, expr: &syn::Expr) -> u32 {
		match expr {
			syn::Expr::Const(inner) => self.constant(inner),
			_ => todo!(),
		}
	}
	pub(super) fn constant(&mut self, constant: &syn::Constant) -> u32 {
		match &constant.kind {
			&syn::ConstantKind::Integer(IntegerKind::U32(num)) => {
				let layout = &DataLayout::Integer(IntegerLayout {
					width: 32,
					is_signed: false,
				});
				let result_type = self.resolve_type(layout);
				self.builder.constant_bit32(result_type, num);
				todo!()
			}
			&syn::ConstantKind::Integer(IntegerKind::I32(num)) => {
				let layout = &DataLayout::Integer(IntegerLayout {
					width: 32,
					is_signed: true,
				});
				let result_type = self.resolve_type(layout);
				unsafe {
					self.builder
						.constant_bit32(result_type, mem::transmute(num));
				}
				todo!()
			}
			&syn::ConstantKind::Floating(FloatingKind::Float(num)) => {
				let layout = &DataLayout::Float(FloatLayout { width: 32 });
				let result_type = self.resolve_type(layout);
				unsafe {
					self.builder
						.constant_bit32(result_type, mem::transmute(num));
				}
				todo!()
			}
			other => todo!("{other:?}"),
		}
	}
}
