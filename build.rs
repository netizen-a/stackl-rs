// Copyright (c) 2024-2026 Jonathan A. Thomason
mod cli {
	pub mod stackl_as {
		include!(concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/src/bin/stackl-as/cli.rs"
		));
	}
	pub mod stackl_cc {
		include!(concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/src/bin/stackl-cc/cli.rs"
		));
	}
	pub mod stackl_vm {
		include!(concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/src/bin/stackl-vm/cli.rs"
		));
	}
}

const MAN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/target/man");

fn main() -> std::io::Result<()> {
	let out_dir = std::env::var("OUT_DIR").unwrap();
	// prepare man page directory
	std::fs::create_dir_all(MAN_DIR)?;
	stackl_as(&out_dir)?;
	stackl_cc(&out_dir)?;
	stackl_vm()?;
	Ok(())
}

fn stackl_as(out_dir: &str) -> std::io::Result<()> {
	lalrpop::Configuration::new()
		.set_in_dir("src/bin/stackl-as")
		.set_out_dir(out_dir.to_owned() + "/bin/stackl-as")
		.emit_rerun_directives(true)
		.process()
		.unwrap();
	let cmd = <cli::stackl_as::Args as clap::CommandFactory>::command().name("stackl-as");
	clap_mangen::generate_to(cmd, MAN_DIR)?;
	Ok(())
}

fn stackl_cc(out_dir: &str) -> std::io::Result<()> {
	lalrpop::Configuration::new()
		.set_in_dir("src/bin/stackl-cc/analysis/syn")
		.set_out_dir(out_dir.to_owned() + "/bin/stackl-cc/analysis/syn")
		.emit_rerun_directives(true)
		.process()
		.unwrap();

	let cmd = <cli::stackl_cc::Args as clap::CommandFactory>::command().name("stackl-cc");
	clap_mangen::generate_to(cmd, MAN_DIR)?;
	Ok(())
}

fn stackl_vm() -> std::io::Result<()> {
	let cmd = <cli::stackl_vm::Args as clap::CommandFactory>::command().name("stackl-vm");
	clap_mangen::generate_to(cmd, MAN_DIR)?;
	Ok(())
}
