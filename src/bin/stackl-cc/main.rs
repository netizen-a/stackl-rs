use std::process::ExitCode;

mod analysis;
mod cli;
mod diagnostics;
mod symtab;
mod synthesis;

fn main() -> ExitCode {
	let args = cli::Args::parse();
	let mut diag_engine = diagnostics::DiagnosticEngine::new();
	let analysis_result = analysis::parse(args.in_file, &mut diag_engine);

	if diag_engine.contains_error() {
		diag_engine.print_errors();
		return ExitCode::FAILURE;
	}
	if diag_engine.contains_warning() {
		diag_engine.print_warnings();
	}

	let ast = analysis_result.unwrap();
	synthesis::parse(&ast);

	ExitCode::SUCCESS
}
