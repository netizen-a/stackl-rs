use std::process::Command;

const BIN_DIR: &str = "../stackl-rs/target/debug/";
const SRC_DIR: &str = "../stackl-rs/examples/";
const OUT_DIR: &str = "../stackl-rs/target/";

#[test]
fn test_example1() {
    let mut assembler_path: String = BIN_DIR.to_owned();
    assembler_path.push_str("stackl-as");

    let mut src_path: String = SRC_DIR.to_owned();
    src_path.push_str("test.asm");

    let mut out_path: String = OUT_DIR.to_owned();
    out_path.push_str("test.stackl");

    let mut interp_path: String = BIN_DIR.to_owned();
    interp_path.push_str("stackl");

    let output = Command::new(assembler_path)
        .arg(src_path)
        .arg("-o")
        .arg(&out_path)
        .output()
        .expect("Failed to execute 'stackl-as'");
    assert!(output.status.success());

    let output = Command::new(interp_path)
        .arg(&out_path)
        .output()
        .expect("Failed to execute 'stackl'");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    let mut expected = String::new();
    for _ in 0..10 {
        expected.push_str("hello world!\n");
    }

    assert_eq!(expected, stdout);
}
