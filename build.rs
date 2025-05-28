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
		.set_in_dir("src/bin/stackl-cc/lex")
		.set_out_dir(out_dir.clone() + "/bin/stackl-cc/lex")
		.emit_rerun_directives(true)
		.process()
		.unwrap();
	// lalrpop::Configuration::new()
	// 	.set_in_dir("src/bin/stackl-cc/syn")
	// 	.set_out_dir(out_dir + "src/bin/stackl-cc/syn")
	// 	.process()
	// 	.unwrap();
}
