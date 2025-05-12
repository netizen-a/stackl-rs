fn main() {
	//lalrpop::process_root().unwrap()
	lalrpop::Configuration::new()
		.set_in_dir("src/bin/stackl-as")
		.process()
		.unwrap();
}
