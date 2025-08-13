use std::process::ExitCode;

mod analysis;
mod cli;
mod symtab;
mod synthesis;

fn main() -> ExitCode {
	let args = cli::Args::parse();
	let analysis_result = analysis::parse(args.in_file);
	let ast = analysis_result.unwrap();
	synthesis::parse(&ast);

	ExitCode::SUCCESS
}
