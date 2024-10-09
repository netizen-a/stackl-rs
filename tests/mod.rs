use std::process::Command;

const BIN_DIR: &str = "../stackl-rs/target/debug/";
const SRC_DIR: &str = "../stackl-rs/examples/";
const OUT_DIR: &str = "../stackl-rs/target/";

#[test]
fn test_assembler() {
    let mut bin_path: String = BIN_DIR.to_owned();
    bin_path.push_str("stackl-as");

    let mut src_path: String = SRC_DIR.to_owned();
    src_path.push_str("test.asm");

    let mut out_path: String = OUT_DIR.to_owned();
    out_path.push_str("test.stackl");

    let output = Command::new(bin_path)
        .arg(src_path)
        .arg("-o")
        .arg(out_path)
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
}
