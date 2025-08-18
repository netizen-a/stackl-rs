use crate::diagnostics::{DiagKind, Diagnostic};

pub fn print_error(diag: &Diagnostic) {
    eprint!("error: ");
    match diag.kind {
        DiagKind::MultStorageClasses => {
            eprintln!("multiple storage classes in declaration specifiers");
        }
        _ => unreachable!(),
    }
}
