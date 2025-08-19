use crate::diagnostics::{DiagKind, Diagnostic};

pub fn print_error(diag: &Diagnostic) {
	eprint!("error: {}", diag.kind.to_string());
}
