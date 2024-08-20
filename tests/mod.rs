// Copyright (c) 2024-2026 Jonathan A. Thomason
use std::{
	env,
	path::PathBuf,
	process::{
		self,
		ExitCode,
	},
};

fn to_exitcode(cond: bool) -> ExitCode {
	if cond {
		ExitCode::SUCCESS
	} else {
		ExitCode::FAILURE
	}
}

#[test]
fn syntax() -> ExitCode {
	let compiler_path = PathBuf::from(env!("CARGO_BIN_EXE_stackl-cc"));
	let test1_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/src/syntax.c");
	println!("path: {}", test1_path.display());
	let out = process::Command::new(compiler_path)
		.arg(test1_path)
		.arg("--trace")
		.output()
		.unwrap();
	println!("stdout:\n{}", String::from_utf8(out.stdout).unwrap());
	println!("stderr:\n{}", String::from_utf8(out.stderr).unwrap());
	to_exitcode(out.status.success())
}
