use std::process::Command;

fn run_axiom(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_axiom"))
        .args(args)
        .output()
        .expect("axiom binary should be available in tests")
}

#[test]
fn semantic_errors_do_not_report_fake_zero_zero_location() {
    let program = "fn main() { let value: Int = true }";
    let temp = std::env::temp_dir().join("axiom_semantic_error.ax");
    std::fs::write(&temp, program).unwrap();

    let output = run_axiom(&["check", temp.to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("variable 'value' has incompatible type"));
    assert!(!stderr.contains("at 0:0"), "stderr should not contain fabricated 0:0 location: {stderr}");
}

#[test]
fn parse_errors_keep_real_source_location() {
    let program = "fn main() { let value = }";
    let temp = std::env::temp_dir().join("axiom_parse_error.ax");
    std::fs::write(&temp, program).unwrap();

    let output = run_axiom(&["check", temp.to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("expected literal") || stderr.contains("expected expression"));
    assert!(stderr.contains("at 1:"), "stderr should contain a real source location: {stderr}");
}
