// warnings are unhelpful when debugging hard errors
#![allow(warnings)]

use std::process::ExitCode;

mod analysis;
mod cli;
mod diagnostics;
mod symtab;
mod synthesis;
mod data_types;

fn main() -> ExitCode {
	let args = cli::Args::parse();
	let mut diag_engine = diagnostics::DiagnosticEngine::new();
	let _analysis_result = analysis::parse(args.in_file, &mut diag_engine);

	if diag_engine.contains_error() {
		diag_engine.print_errors();
		return ExitCode::FAILURE;
	}
	// We do not print warnings with errors.
	// Warnings are a liability if the code is erroneous.
	diag_engine.print_warnings();

	//synthesis::parse(&analysis_result.unwrap());
	println!("{:#?}", _analysis_result);

	ExitCode::SUCCESS
}
