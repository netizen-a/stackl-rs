use crate::analysis::syn;

impl super::SSACodeGen<'_> {
	pub(super) fn expr(&mut self, expr: &syn::Expr) -> u32 {
		match expr {
			syn::Expr::Const(syn::Constant {
				kind: syn::ConstantKind::Integer(inner),
				..
			}) => {
				todo!()
			}
			_ => todo!(),
		}
	}
}
