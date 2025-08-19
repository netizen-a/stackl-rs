use crate::diagnostics::{DiagKind, Diagnostic};

pub fn print_error(diag: &Diagnostic) {
	eprintln!("error: {}", diag.kind.to_string());
}
