use std::process::Command;

const RUN_PATH: &str = "../stackl-rs/target/debug/stackl";
const BLD_PATH: &str = "../stackl-rs/target/debug/stackl-as";

const SRC_DIR: &str = "../stackl-rs/tests/";
const OUT_DIR: &str = "../stackl-rs/target/";

#[test]
fn test_example1() {
    let mut src_path: String = SRC_DIR.to_owned();
    src_path.push_str("test.asm");

    let mut out_path: String = OUT_DIR.to_owned();
    out_path.push_str("test.stackl");

    let output = Command::new(BLD_PATH)
        .arg(src_path)
        .arg("-o")
        .arg(&out_path)
        .output()
        .expect("Failed to execute 'stackl-as'");
    assert!(output.status.success());

    let output = Command::new(RUN_PATH)
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

#[test]
fn test_example2() {
    let mut src_path: String = SRC_DIR.to_owned();
    src_path.push_str("test2.asm");

    let mut out_path: String = OUT_DIR.to_owned();
    out_path.push_str("test2.stackl");

    let output = Command::new(BLD_PATH)
        .arg(src_path)
        .arg("-o")
        .arg(&out_path)
        .output()
        .expect("Failed to execute 'stackl-as'");
    assert!(output.status.success());

    let output = Command::new(RUN_PATH)
        .arg(&out_path)
        .output()
        .expect("Failed to execute 'stackl'");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!("Hello World!\n", stdout);
}
