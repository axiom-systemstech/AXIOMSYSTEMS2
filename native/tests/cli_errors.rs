use std::process::Command;

fn run_axiom(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_axiom"))
        .args(args)
        .output()
        .expect("axiom binary should be available in tests")
}

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("axiom_{}_{}", std::process::id(), name))
}

#[test]
fn semantic_errors_do_not_report_fake_zero_zero_location() {
    let program = "fn main() { let value: Int = true }";
    let temp = temp_path("semantic_error.ax");
    std::fs::write(&temp, program).unwrap();

    let output = run_axiom(&["check", temp.to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("variable 'value' has incompatible type"));
    assert!(
        !stderr.contains("at 0:0"),
        "stderr should not contain fabricated 0:0 location: {stderr}"
    );
}

#[test]
fn parse_errors_keep_real_source_location() {
    let program = "fn main() { let value = }";
    let temp = temp_path("parse_error.ax");
    std::fs::write(&temp, program).unwrap();

    let output = run_axiom(&["check", temp.to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("expected literal") || stderr.contains("expected expression"));
    assert!(
        stderr.contains("at 1:"),
        "stderr should contain a real source location: {stderr}"
    );
}

#[test]
fn build_writes_custom_artifact_and_run_executes_it() {
    let source_path = temp_path("artifact_source.ax");
    let artifact_path = temp_path("artifact_output.axm");
    std::fs::write(
        &source_path,
        "fn main() { let values = [10, 20, 30]; print(values[1]) }",
    )
    .unwrap();

    let build = run_axiom(&[
        "build",
        source_path.to_str().unwrap(),
        artifact_path.to_str().unwrap(),
    ]);
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(
        artifact_path.exists(),
        "build should create the requested artifact"
    );

    let run = run_axiom(&["run", artifact_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "artifact run failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "20\n");

    let _ = std::fs::remove_file(source_path);
    let _ = std::fs::remove_file(artifact_path);
}

#[test]
fn less_than_runs_in_source_and_compiled_artifact() {
    let source_path = temp_path("less_than_source.ax");
    let artifact_path = temp_path("less_than_output.axm");
    std::fs::write(
        &source_path,
        "fn main() { let count: Int = 0; while count < 3 { print(count); count = count + 1 } }",
    )
    .unwrap();

    let source_run = run_axiom(&["run", source_path.to_str().unwrap()]);
    assert!(
        source_run.status.success(),
        "source run failed: {}",
        String::from_utf8_lossy(&source_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&source_run.stdout), "0\n1\n2\n");

    let build = run_axiom(&[
        "build",
        source_path.to_str().unwrap(),
        artifact_path.to_str().unwrap(),
    ]);
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    let artifact_run = run_axiom(&["run", artifact_path.to_str().unwrap()]);
    assert!(
        artifact_run.status.success(),
        "artifact run failed: {}",
        String::from_utf8_lossy(&artifact_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&artifact_run.stdout), "0\n1\n2\n");

    let _ = std::fs::remove_file(source_path);
    let _ = std::fs::remove_file(artifact_path);
}

#[test]
fn unary_negation_runs_in_source_and_compiled_artifact() {
    let source_path = temp_path("negate_source.ax");
    let artifact_path = temp_path("negate_output.axm");
    std::fs::write(&source_path, "fn main() { print(-(2 + 3)); print(-2 * 3) }").unwrap();

    let source_run = run_axiom(&["run", source_path.to_str().unwrap()]);
    assert!(
        source_run.status.success(),
        "source run failed: {}",
        String::from_utf8_lossy(&source_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&source_run.stdout), "-5\n-6\n");

    let build = run_axiom(&[
        "build",
        source_path.to_str().unwrap(),
        artifact_path.to_str().unwrap(),
    ]);
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    let artifact_run = run_axiom(&["run", artifact_path.to_str().unwrap()]);
    assert!(
        artifact_run.status.success(),
        "artifact run failed: {}",
        String::from_utf8_lossy(&artifact_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&artifact_run.stdout), "-5\n-6\n");

    let _ = std::fs::remove_file(source_path);
    let _ = std::fs::remove_file(artifact_path);
}
