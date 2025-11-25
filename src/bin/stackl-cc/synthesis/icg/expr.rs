use crate::{
	analysis::syn::{
		self,
		IntegerKind,
	},
	synthesis::icg::{
		DataLayout,
		IntegerLayout,
	},
};

impl super::SSACodeGen<'_> {
	pub(super) fn expr(&mut self, expr: &syn::Expr) -> u32 {
		match expr {
			syn::Expr::Const(inner) => self.constant(inner),
			_ => todo!(),
		}
	}
	pub(super) fn constant(&mut self, constant: &syn::Constant) -> u32 {
		match &constant.kind {
			syn::ConstantKind::Integer(IntegerKind::I32(num)) => {
				let layout = &DataLayout::Integer(IntegerLayout {
					width: 32,
					is_signed: true,
				});
				let result_type = self.resolve_type(layout);
				self.builder.constant_bit32(result_type, *num as u32);
				todo!()
			}
			other => todo!("{other:?}"),
		}
	}
}
