use std::fs;
use std::path::{Path, PathBuf};

use crate::parser::{parse, Program};
use crate::vm::{build_artifact, execute_artifact, write_artifact_file_with_target, VmError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub root: PathBuf,
    pub source: PathBuf,
    pub tests: Vec<PathBuf>,
}

pub fn discover(path: &Path) -> Result<Project, VmError> {
    let root = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or_else(|| Path::new("."))
    };
    let source = if path.is_file() {
        path.to_path_buf()
    } else {
        let candidate = root.join("src/main.ax");
        if candidate.is_file() {
            candidate
        } else {
            root.join("main.ax")
        }
    };
    if !source.is_file() {
        return Err(error(format!(
            "project entry source not found: {}",
            source.display()
        )));
    }
    let tests_root = root.join("tests");
    let mut tests = Vec::new();
    if tests_root.is_dir() {
        collect_sources(&tests_root, &mut tests)?;
    }
    Ok(Project {
        root: root.to_path_buf(),
        source,
        tests,
    })
}

fn collect_sources(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), VmError> {
    for entry in
        fs::read_dir(root).map_err(|e| error(format!("cannot read '{}': {e}", root.display())))?
    {
        let entry = entry.map_err(|e| error(format!("cannot read directory entry: {e}")))?;
        let path = entry.path();
        if path.is_dir() {
            collect_sources(&path, output)?;
        } else if path.extension().and_then(|v| v.to_str()) == Some("ax") {
            output.push(path);
        }
    }
    output.sort();
    Ok(())
}

pub fn read_program(path: &Path) -> Result<(String, Program), VmError> {
    let source = fs::read_to_string(path)
        .map_err(|e| error(format!("cannot read '{}': {e}", path.display())))?;
    let program =
        parse(&source).map_err(|e| error(format!("{} at {}:{}", e.message, e.line, e.column)))?;
    crate::semantic::analyze(&program).map_err(|e| error(e.message.clone()))?;
    Ok((source, program))
}

pub fn build_project(project: &Project, output: Option<&Path>) -> Result<PathBuf, VmError> {
    let (_, program) = read_program(&project.source)?;
    let default = project.root.join("build/main.axm");
    let target = output.unwrap_or(&default);
    write_artifact_file_with_target(&project.source, Some(target), &program)
}

pub fn run_project(project: &Project) -> Result<String, VmError> {
    let (_, program) = read_program(&project.source)?;
    let artifact = crate::vm::compile_program(&program);
    execute_artifact(&artifact)
}

pub fn test_project(project: &Project) -> Result<usize, VmError> {
    for test in &project.tests {
        let (_, program) = read_program(test)?;
        let artifact = crate::vm::compile_program(&program);
        execute_artifact(&artifact)?;
    }
    Ok(project.tests.len())
}
pub fn new_project(path: &Path) -> Result<(), VmError> {
    if path.exists() {
        return Err(error(format!(
            "project path already exists: {}",
            path.display()
        )));
    }
    fs::create_dir_all(path.join("src"))
        .map_err(|e| error(format!("cannot create project: {e}")))?;
    fs::create_dir_all(path.join("tests"))
        .map_err(|e| error(format!("cannot create tests: {e}")))?;
    let name = path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("axiom-project");
    fs::write(
        path.join("axiom.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[dependencies]\n"),
    )
    .map_err(|e| error(format!("cannot write manifest: {e}")))?;
    fs::write(
        path.join("src/main.ax"),
        "fn main() { print(\"Hello AXIOM\") }\n",
    )
    .map_err(|e| error(format!("cannot write main.ax: {e}")))?;
    fs::write(
        path.join("tests/main.ax"),
        "fn main() { print(\"test ok\") }\n",
    )
    .map_err(|e| error(format!("cannot write test: {e}")))?;
    Ok(())
}

pub fn package_project(project: &Project, output: &Path) -> Result<(), VmError> {
    let mut files = Vec::new();
    collect_package_files(&project.root, &project.root, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut encoded = String::from("AXIOM_PACKAGE_V1\n");
    for (name, data) in files {
        encoded.push_str(&format!(
            "FILE|{}|{}\n",
            hex_encode(name.as_bytes()),
            hex_encode(&data)
        ));
    }
    fs::write(output, encoded)
        .map_err(|e| error(format!("cannot write package '{}': {e}", output.display())))?;
    Ok(())
}

fn collect_package_files(
    root: &Path,
    base: &Path,
    output: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), VmError> {
    for entry in
        fs::read_dir(root).map_err(|e| error(format!("cannot read '{}': {e}", root.display())))?
    {
        let entry = entry.map_err(|e| error(format!("cannot read directory entry: {e}")))?;
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap_or(&path);
        if relative == Path::new("target")
            || relative == Path::new(".git")
            || path.extension().and_then(|v| v.to_str()) == Some("axpkg")
        {
            continue;
        }
        if path.is_dir() {
            collect_package_files(&path, base, output)?;
        } else {
            let name = relative.to_string_lossy().replace('\\', "/");
            output.push((
                name,
                fs::read(&path).map_err(|e| error(format!("cannot read package file: {e}")))?,
            ));
        }
    }
    Ok(())
}
pub fn add_local_package(root: &Path, name: &str, registry: &Path) -> Result<String, VmError> {
    let package = registry.join(name);
    let manifest = package.join("axiom.toml");
    if !manifest.is_file() {
        return Err(error(format!("package '{}' not found in registry", name)));
    }
    let version = manifest_version(&manifest)?;
    let destination = root.join("vendor").join(name);
    if destination.exists() {
        return Err(error(format!("package '{}' is already vendored", name)));
    }
    copy_tree(&package, &destination)?;
    let project_manifest = root.join("axiom.toml");
    let mut manifest_text = if project_manifest.is_file() {
        fs::read_to_string(&project_manifest)
            .map_err(|e| error(format!("cannot read axiom.toml: {e}")))?
    } else {
        String::from("[package]\nname = \"project\"\nversion = \"0.1.0\"\n\n[dependencies]\n")
    };
    if !manifest_text.contains("[dependencies]") {
        manifest_text.push_str("\n[dependencies]\n");
    }
    if !manifest_text
        .lines()
        .any(|line| line.trim_start().starts_with(&format!("{name} =")))
    {
        manifest_text.push_str(&format!(
            r#"{name} = "{version}"
"#
        ));
    }
    fs::write(&project_manifest, manifest_text)
        .map_err(|e| error(format!("cannot update axiom.toml: {e}")))?;
    let lock = root.join("axiom.lock");
    fs::write(lock, format!("AXIOM_LOCK_V1\n{name}={version}\n"))
        .map_err(|e| error(format!("cannot write axiom.lock: {e}")))?;
    Ok(version)
}

fn manifest_version(path: &Path) -> Result<String, VmError> {
    let text = fs::read_to_string(path).map_err(|e| error(format!("cannot read manifest: {e}")))?;
    for line in text.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("version") {
            if let Some(value) = value.split('=').nth(1) {
                return Ok(value.trim().trim_matches('"').to_string());
            }
        }
    }
    Err(error("package manifest has no version".into()))
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), VmError> {
    fs::create_dir_all(destination)
        .map_err(|e| error(format!("cannot create vendor directory: {e}")))?;
    for entry in fs::read_dir(source).map_err(|e| error(format!("cannot read package: {e}")))? {
        let entry = entry.map_err(|e| error(format!("cannot read package entry: {e}")))?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| error(format!("cannot copy package file: {e}")))?;
        }
    }
    Ok(())
}

fn hex_encode(data: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(data.len() * 2);
    for byte in data {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn error(message: String) -> VmError {
    VmError { message }
}
pub fn install_layout(prefix: &Path, binary: &Path) -> Result<PathBuf, VmError> {
    let bin = prefix.join("bin");
    fs::create_dir_all(&bin).map_err(|e| error(format!("cannot create install directory: {e}")))?;
    let destination = bin.join("axiom");
    fs::copy(binary, &destination)
        .map_err(|e| error(format!("cannot install '{}': {e}", binary.display())))?;
    Ok(destination)
}

pub fn artifact_from_source(path: &Path) -> Result<String, VmError> {
    let (_, program) = read_program(path)?;
    Ok(build_artifact(&program))
}
