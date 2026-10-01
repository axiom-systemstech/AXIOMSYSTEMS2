use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        return ExitCode::SUCCESS;
    };
    if matches!(command.as_str(), "--version" | "-V") {
        println!("axiom 0.3.0-native");
        return ExitCode::SUCCESS;
    }
    if matches!(command.as_str(), "--help" | "-h") {
        print_help();
        return ExitCode::SUCCESS;
    }
    if command == "doctor" {
        println!("axiom 0.3.0-native");
        println!("runtime native");
        println!("python dependency none");
        return ExitCode::SUCCESS;
    }
    if command == "new" {
        let Some(path) = args.next() else {
            return fail("error: expected project path");
        };
        return finish(
            axiom_native::toolchain::new_project(Path::new(&path)),
            &format!("created: {path}"),
        );
    }
    if command == "install" {
        let Some(prefix) = args.next() else {
            return fail("error: expected install prefix");
        };
        let binary = env::current_exe().unwrap_or_else(|_| PathBuf::from("axiom"));
        return match axiom_native::toolchain::install_layout(Path::new(&prefix), &binary) {
            Ok(path) => {
                println!("installed: {}", path.display());
                ExitCode::SUCCESS
            }
            Err(error) => fail(&format!("error: {}", error.message)),
        };
    }
    let Some(path) = args.next() else {
        return fail("error: expected source or project path");
    };
    let path = PathBuf::from(path);
    if command == "run" && path.extension().and_then(|ext| ext.to_str()) == Some("axm") {
        return match std::fs::read_to_string(&path)
            .map_err(|e| axiom_native::vm::VmError {
                message: format!("cannot read '{}': {e}", path.display()),
            })
            .and_then(|encoded| axiom_native::vm::Artifact::deserialize(&encoded))
            .and_then(|artifact| axiom_native::vm::execute_artifact(&artifact))
        {
            Ok(output) => {
                print!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => fail(&format!("error: {}", error.message)),
        };
    }
    if command == "add" {
        let Some(name) = args.next() else {
            return fail("error: expected package name");
        };
        let registry = args.next().unwrap_or_else(|| "registry".into());
        return match axiom_native::toolchain::add_local_package(&path, &name, Path::new(&registry))
        {
            Ok(version) => {
                println!("added: {name} {version}");
                ExitCode::SUCCESS
            }
            Err(error) => fail(&format!("error: {}", error.message)),
        };
    }
    let project = match axiom_native::toolchain::discover(&path) {
        Ok(project) => project,
        Err(error) => return fail(&format!("error: {}", error.message)),
    };
    match command.as_str() {
        "check" => match axiom_native::toolchain::read_program(&project.source) {
            Ok(_) => {
                println!("ok: {}", project.source.display());
                ExitCode::SUCCESS
            }
            Err(error) => fail(&format!("error: {}", error.message)),
        },
        "run" => match axiom_native::toolchain::run_project(&project) {
            Ok(output) => {
                print!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => fail(&format!("error: {}", error.message)),
        },
        "build" => {
            let output = parse_output(&mut args);
            match axiom_native::toolchain::build_project(&project, output.as_deref()) {
                Ok(path) => {
                    println!("build ok: {}", path.display());
                    ExitCode::SUCCESS
                }
                Err(error) => fail(&format!("error: {}", error.message)),
            }
        }
        "test" => match axiom_native::toolchain::test_project(&project) {
            Ok(count) => {
                println!("test ok: {count} source file(s)");
                ExitCode::SUCCESS
            }
            Err(error) => fail(&format!("error: {}", error.message)),
        },
        "package" => {
            let output =
                parse_output(&mut args).unwrap_or_else(|| project.root.with_extension("axpkg"));
            match axiom_native::toolchain::package_project(&project, &output) {
                Ok(()) => {
                    println!("packaged: {}", output.display());
                    ExitCode::SUCCESS
                }
                Err(error) => fail(&format!("error: {}", error.message)),
            }
        }
        "native-build" => {
            let output = parse_output(&mut args);
            match axiom_native::toolchain::read_program(&project.source) {
                Ok((_, program)) => match axiom_native::backend::build_executable(
                    &project.source,
                    &program,
                    output.as_deref(),
                    "host",
                ) {
                    Ok(build) => {
                        println!("native build ok: {}", build.output.display());
                        ExitCode::SUCCESS
                    }
                    Err(error) => fail(&format!("error: {}", error.message)),
                },
                Err(error) => fail(&format!("error: {}", error.message)),
            }
        }
        _ => {
            eprintln!("error: unknown command '{command}'");
            print_help();
            ExitCode::from(2)
        }
    }
}
fn parse_output(args: &mut impl Iterator<Item = String>) -> Option<PathBuf> {
    let mut output = None;
    while let Some(argument) = args.next() {
        if argument == "-o" || argument == "--output" {
            output = args.next().map(PathBuf::from);
        } else if output.is_none() {
            output = Some(PathBuf::from(argument));
        } else {
            return None;
        }
    }
    output
}

fn finish(result: Result<(), axiom_native::vm::VmError>, message: &str) -> ExitCode {
    match result {
        Ok(()) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => fail(&format!("error: {}", error.message)),
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(1)
}

fn print_help() {
    println!("AXIOM native toolchain");
    println!(
        "Usage: axiom <new|check|build|run|test|package|add|native-build|install> <path> [options]"
    );
    println!("  new <path>                 create a project");
    println!("  check <path>               validate a source or project");
    println!("  build <path> [-o FILE]     build AXIOM_ARTIFACT_V1");
    println!("  run <path>                 execute source or project");
    println!("  test <project>             run project tests");
    println!("  package <project> [-o FILE] create AXIOM_PACKAGE_V1");
    println!("  add <project> <name> [registry]  vendor a local package");
    println!("  native-build <path> [-o FILE]   create standalone executable");
    println!("  install <prefix>            install this executable");
}
