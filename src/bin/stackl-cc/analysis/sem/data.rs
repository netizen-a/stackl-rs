use crate::analysis::syn;
use crate::data_types::*;
use crate::diagnostics as diag;
use diag::ToSpan;

impl super::SemanticParser<'_> {
    pub(super) fn unwrap_or_poison(&mut self, value: Option<DataType>, ident: syn::Identifier) -> DataType {
        let ident_span = ident.to_span();
        let ident_name = ident.name;
        match value {
		    Some(ty) => ty.clone(),
		    None => {
		    	let diag = diag::Diagnostic::error(
		    		diag::DiagKind::ImplicitInt(ident_name),
		    		ident_span,
		    	);
		    	self.diagnostics.push(diag);
		    	DataType {
		    		kind: TypeKind::Poison,
		    		qual: TypeQual::default(),
		    	}
		    }
        }
    }
    pub(super) fn dtype_eq(&self, lhs: &DataType, rhs: &DataType) -> bool {
        false
    }
}
