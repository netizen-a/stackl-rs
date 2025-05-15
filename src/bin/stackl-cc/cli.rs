use clap::crate_version;
use std::{path::PathBuf, process};

#[allow(dead_code)]
#[derive(Debug)]
#[repr(i32)]
#[non_exhaustive]
pub enum StandardVersion {
	C99 = 199901,
	C11 = 201112,
	C17 = 201710,
	C23 = 202311,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(i32)]
#[non_exhaustive]
pub enum PreprocStdout {
	Tokens = -1,
	Disabled = 0,
	Enabled = 1,
	Comments = 2,
}

#[derive(Debug)]
pub struct Args {
	pub in_file: PathBuf,
	pub out_file: Option<PathBuf>,
	pub std: StandardVersion,
	pub pp_stdout: PreprocStdout,
}

impl Args {
	pub fn parse() -> Self {
		let arg_list = [
			clap::Arg::new("file").required(true),
			clap::Arg::new("standard")
				.long("std")
				.default_value("c99")
				.value_names(["c99"]),
			clap::Arg::new("output").short('o'),
			clap::Arg::new("pp-stdout")
				.short('E')
				.required(false)
				.action(clap::ArgAction::SetTrue),
			clap::Arg::new("pp-stdout-comments")
				.long("include-comments")
				.requires("pp-stdout")
				.action(clap::ArgAction::SetTrue)
				.conflicts_with("pp-stdout-tokens"),
			clap::Arg::new("pp-stdout-tokens")
				.long("pp-tokens")
				.requires("pp-stdout")
				.action(clap::ArgAction::SetTrue)
				.conflicts_with("pp-stdout-comments"),
		];

		let cmd = clap::Command::new("stacklc")
			.version(crate_version!())
			.about("Stackl C compiler")
			.args(arg_list)
			.arg_required_else_help(true);

		let matches = cmd.get_matches();

		let maybe_std = matches.get_one::<String>("standard").unwrap();
		let std = match maybe_std.as_str() {
			"c99" => StandardVersion::C99,
			other => {
				eprintln!("Invalid standard `{other}`");
				process::exit(-1);
			}
		};
		let in_file = matches.get_one::<String>("file").unwrap();
		let in_file = PathBuf::from(in_file);

		let out_file = matches
			.get_one::<String>("output")
			.cloned()
			.map(PathBuf::from);

		let pp_stdout = if matches.get_flag("pp-stdout") {
			if matches.get_flag("pp-stdout-comments") {
				PreprocStdout::Comments
			} else if matches.get_flag("pp-stdout-tokens") {
				PreprocStdout::Tokens
			} else {
				PreprocStdout::Enabled
			}
		} else {
			PreprocStdout::Disabled
		};

		Self {
			in_file,
			out_file,
			std,
			pp_stdout,
		}
	}
}
