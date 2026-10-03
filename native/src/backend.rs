use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::parser::Program;
use crate::vm::{build_artifact, VmError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeTarget {
    pub triple: String,
}

impl NativeTarget {
    pub fn host() -> Self {
        Self {
            triple: "host".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeBuild {
    pub target: NativeTarget,
    pub source: PathBuf,
    pub output: PathBuf,
}

pub fn build_executable(
    source_path: &Path,
    program: &Program,
    output: Option<&Path>,
    target: &str,
) -> Result<NativeBuild, VmError> {
    let target = NativeTarget {
        triple: target.to_string(),
    };
    let output = output
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output(source_path));
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| VmError {
        message: format!(
            "cannot create native output directory '{}': {error}",
            parent.display()
        ),
    })?;

    let artifact = build_artifact(program);
    let generated = generate_rust_launcher(&artifact);
    let generated_path = output.with_extension("axn.rs");
    fs::write(&generated_path, generated).map_err(|error| VmError {
        message: format!(
            "cannot write generated native source '{}': {error}",
            generated_path.display()
        ),
    })?;

    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let mut cargo = Command::new("cargo");
    cargo
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--lib")
        .arg("--release");
    if target.triple != "host" {
        cargo.arg("--target").arg(&target.triple);
    }
    let result = cargo.output().map_err(|error| VmError {
        message: format!("cannot invoke cargo: {error}"),
    })?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(VmError {
            message: if stderr.is_empty() {
                format!("cargo failed with status {}", result.status)
            } else {
                format!("cargo failed: {stderr}")
            },
        });
    }

    let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
    let library = if target.triple == "host" {
        target_dir.join("release/libaxiom_native.rlib")
    } else {
        target_dir
            .join(&target.triple)
            .join("release/libaxiom_native.rlib")
    };
    let mut command = Command::new("rustc");
    command
        .arg(&generated_path)
        .arg("--edition=2021")
        .arg("--crate-name")
        .arg("axiom_app");
    if target.triple != "host" {
        command.arg("--target").arg(&target.triple);
    }
    command
        .arg("--extern")
        .arg(format!("axiom_native={}", library.display()));
    command.arg("-C").arg("opt-level=2");
    command.arg("-C").arg("codegen-units=1");
    command.arg("-C").arg("metadata=axiom_app");
    command
        .arg("--remap-path-prefix")
        .arg(format!("{}=.", env!("CARGO_MANIFEST_DIR")));
    command.arg("-o").arg(&output);
    let result = command.output().map_err(|error| VmError {
        message: format!("cannot invoke rustc: {error}"),
    })?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(VmError {
            message: if stderr.is_empty() {
                format!("rustc failed with status {}", result.status)
            } else {
                format!("rustc failed: {stderr}")
            },
        });
    }
    let _ = fs::remove_file(&generated_path);
    Ok(NativeBuild {
        target,
        source: source_path.to_path_buf(),
        output,
    })
}

fn default_output(source: &Path) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("axiom");
    source.with_file_name(stem)
}

fn generate_rust_launcher(artifact: &str) -> String {
    let encoded = artifact.escape_default().to_string();
    format!(
        "extern crate axiom_native;\n\nfn main() {{\n    const ARTIFACT: &str = \"{encoded}\";\n    let artifact = axiom_native::vm::Artifact::deserialize(ARTIFACT).expect(\"invalid embedded AXIOM artifact\");\n    match axiom_native::vm::execute_artifact(&artifact) {{\n        Ok(output) => print!(\"{{}}\", output),\n        Err(error) => {{ eprintln!(\"error: {{}}\", error.message); std::process::exit(1); }}\n    }}\n}}\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_target_is_explicit() {
        assert_eq!(NativeTarget::host().triple, "host");
    }

    #[test]
    fn generated_launcher_is_deterministic() {
        let artifact = "AXIOM_ARTIFACT_V1\nINSTR:PRINT:Hello";
        assert_eq!(
            generate_rust_launcher(artifact),
            generate_rust_launcher(artifact)
        );
    }
}
