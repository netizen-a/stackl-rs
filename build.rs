use std::env;

fn main() {
	let out_dir = env::var("OUT_DIR").unwrap();
	lalrpop::Configuration::new()
		.set_in_dir("src/bin/stackl-as")
		.set_out_dir(out_dir.clone() + "/bin/stackl-as")
		.emit_rerun_directives(true)
		.process()
		.unwrap();
	lalrpop::Configuration::new()
		.set_in_dir("src/bin/stackl-cc/analysis/lex")
		.set_out_dir(out_dir.clone() + "/bin/stackl-cc/analysis/lex")
		.emit_rerun_directives(true)
		.process()
		.unwrap();
	lalrpop::Configuration::new()
		.set_in_dir("src/bin/stackl-cc/analysis/syn")
		.set_out_dir(out_dir + "/bin/stackl-cc/analysis/syn")
		.emit_rerun_directives(true)
		.process()
		.unwrap();
}
