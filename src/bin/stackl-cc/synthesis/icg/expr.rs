use crate::analysis::syn;

impl super::SSACodeGen<'_> {
	pub(super) fn expr(&mut self, expr: &syn::Expr) -> u32 {
		match expr {
			syn::Expr::Const(inner) => self.constant(inner),
			_ => todo!(),
		}
	}
	pub(super) fn constant(&mut self, constant: &syn::Constant) -> u32 {
		match &constant.kind {
			syn::ConstantKind::Integer(_inner) => {
				todo!()
			}
			syn::ConstantKind::CharConst(_inner) => {
				todo!()
			}
			syn::ConstantKind::Floating(_inner) => {
				todo!()
			}
		}
	}
}
