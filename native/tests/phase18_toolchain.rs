use std::fs;
use std::path::PathBuf;

use axiom_native::toolchain::{
    build_project, discover, new_project, package_project, run_project, test_project,
};

fn fixture_root(name: &str) -> PathBuf {
    std::env::current_dir().unwrap().join("target").join(name)
}

#[test]
fn native_project_workflow_is_self_contained() {
    let root = fixture_root("phase18-project-test");
    let _ = fs::remove_dir_all(&root);
    new_project(&root).unwrap();

    let project = discover(&root).unwrap();
    assert_eq!(run_project(&project).unwrap(), "Hello AXIOM\n");
    assert_eq!(test_project(&project).unwrap(), 1);

    let artifact = build_project(&project, None).unwrap();
    assert!(artifact.ends_with("build/main.axm"));
    let package = root.with_extension("axpkg");
    package_project(&project, &package).unwrap();
    let encoded = fs::read_to_string(&package).unwrap();
    assert!(encoded.starts_with("AXIOM_PACKAGE_V1\n"));

    let _ = fs::remove_file(package);
    let _ = fs::remove_dir_all(root);
}
