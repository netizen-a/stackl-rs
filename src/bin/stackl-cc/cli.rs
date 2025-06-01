use clap::crate_version;
use std::{path::PathBuf, process};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(i32)]
#[non_exhaustive]
pub enum PreprocStdout {
	TokenComments = -2,
	Token = -1,
	Disabled = 0,
	Print = 1,
	PrintComments = 2,
}

#[derive(Debug)]
pub struct Args {
	pub in_file: PathBuf,
	pub out_file: Option<PathBuf>,
	pub pp_stdout: PreprocStdout,
}

impl Args {
	pub fn parse() -> Self {
		let arg_list = [
			clap::Arg::new("file").required(true),
			clap::Arg::new("output").short('o'),
			clap::Arg::new("pp-stdout")
				.short('E')
				.required(false)
				.action(clap::ArgAction::SetTrue),
			clap::Arg::new("pp-stdout-comments")
				.long("include-comments")
				.requires("pp-stdout")
				.action(clap::ArgAction::SetTrue),
			clap::Arg::new("pp-stdout-tokens")
				.long("pp-tokens")
				.requires("pp-stdout")
				.action(clap::ArgAction::SetTrue),
		];

		let cmd = clap::Command::new("stacklc")
			.version(crate_version!())
			.about("Stackl C compiler")
			.args(arg_list)
			.arg_required_else_help(true);

		let matches = cmd.get_matches();

		let in_file = matches.get_one::<String>("file").unwrap();
		let in_file = PathBuf::from(in_file);

		let out_file = matches
			.get_one::<String>("output")
			.cloned()
			.map(PathBuf::from);

		let pp_stdout = if matches.get_flag("pp-stdout") {
			let is_tokens = matches.get_flag("pp-stdout-tokens");
			let is_comments = matches.get_flag("pp-stdout-comments");
			match (is_tokens, is_comments) {
				(true, true) => PreprocStdout::TokenComments,
				(true, false) => PreprocStdout::Token,
				(false, false) => PreprocStdout::Print,
				(false, true) => PreprocStdout::PrintComments,
			}
		} else {
			PreprocStdout::Disabled
		};

		Self {
			in_file,
			out_file,
			pp_stdout,
		}
	}
}
