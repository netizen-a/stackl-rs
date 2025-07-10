use std::process::ExitCode;

mod analysis;
mod cli;
mod synthesis;

fn main() -> ExitCode {
	let args = cli::Args::parse();
	analysis::parse(args.in_file);

	ExitCode::SUCCESS
}
