pub mod lex;
pub mod sem;
pub mod syn;
pub mod tok;

use std::io::Read;
use std::{
	fs,
	path::{Path, PathBuf},
	rc::Rc,
};

use crate::analysis::syn::ExternalDeclaration;
use crate::analysis::tok::TokenTriple;
use crate::diagnostics::DiagnosticEngine;
use lalrpop_util as lalr;
