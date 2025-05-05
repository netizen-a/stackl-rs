use clap::crate_version;
use std::{path::PathBuf, process};

#[derive(Debug)]
#[non_exhaustive]
pub enum Standard {
    C89,
}

#[derive(Debug)]
pub struct Args {
    pub in_file: PathBuf,
    pub out_file: Option<PathBuf>,
    pub standard: Standard,
    pub stdout_pp: bool,
    pub include_comments: bool,
}

impl Args {
    pub fn parse() -> Self {
        let arg_list = [
            clap::Arg::new("file").required(true),
            clap::Arg::new("standard")
                .long("std")
                .default_value("c89")
                .value_names(["c89"]),
            clap::Arg::new("output").short('o'),
            clap::Arg::new("stdout-preproc")
                .short('E')
                .required(false)
                .action(clap::ArgAction::SetTrue),
            clap::Arg::new("include-comments")
                .long("include-comments")
                .requires("stdout-preproc")
                .action(clap::ArgAction::SetTrue),
        ];

        let cmd = clap::Command::new("stacklc")
            .version(crate_version!())
            .about("Stackl C compiler")
            .args(arg_list)
            .arg_required_else_help(true);

        let matches = cmd.get_matches();

        let maybe_std = matches.get_one::<String>("standard").unwrap();
        let standard = match maybe_std.as_str() {
            "c89" | "c90" => Standard::C89,
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

        let stdout_pp = matches.get_flag("stdout-preproc");

        let include_comments = matches.get_flag("include-comments");

        Self {
            in_file,
            out_file,
            standard,
            stdout_pp,
            include_comments,
        }
    }
}
