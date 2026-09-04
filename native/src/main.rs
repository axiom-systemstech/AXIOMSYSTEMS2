use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut arguments = env::args().skip(1);
    let Some(command) = arguments.next() else {
        print_help();
        return ExitCode::SUCCESS;
    };

    if command == "--version" || command == "-V" {
        println!("axiom 0.2.0-native");
        return ExitCode::SUCCESS;
    }

    if command == "--help" || command == "-h" {
        print_help();
        return ExitCode::SUCCESS;
    }

    let Some(path) = arguments.next() else {
        eprintln!("error: expected a source file");
        return ExitCode::from(2);
    };

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error: cannot read {path}: {error}");
            return ExitCode::from(1);
        }
    };

    match command.as_str() {
        "check" => match axiom_native::parser::parse(&source).and_then(|program| {
            axiom_native::semantic::analyze(&program)
                .map(|_| program)
                .map_err(|error| axiom_native::parser::ParseError {
                    message: error.message,
                    line: 0,
                    column: 0,
                })
        }) {
            Ok(_) => {
                println!("ok: {path}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!(
                    "error: {} at {}:{}",
                    error.message, error.line, error.column
                );
                ExitCode::from(1)
            }
        },
        "build" => match axiom_native::parser::parse(&source).and_then(|program| {
            axiom_native::semantic::analyze(&program)
                .map(|_| program)
                .map_err(|error| axiom_native::parser::ParseError {
                    message: error.message,
                    line: 0,
                    column: 0,
                })
        }) {
            Ok(program) => {
                let output_path = match axiom_native::vm::write_artifact_file(Path::new(&path), &program) {
                    Ok(path) => path,
                    Err(error) => {
                        eprintln!("error: {}", error.message);
                        return ExitCode::from(1);
                    }
                };
                let artifact = axiom_native::vm::build_artifact(&program);
                println!("build ok: {path}");
                println!("artifact file: {}", output_path.display());
                println!("artifact bytes: {}", artifact.len());
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!(
                    "error: {} at {}:{}",
                    error.message, error.line, error.column
                );
                ExitCode::from(1)
            }
        },
        "run" => {
            if Path::new(&path).extension().and_then(|ext| ext.to_str()) == Some("axm") {
                match fs::read_to_string(&path) {
                    Ok(encoded) => match axiom_native::vm::Artifact::deserialize(&encoded) {
                        Ok(artifact) => match axiom_native::vm::execute_artifact(&artifact) {
                            Ok(output) => {
                                print!("{output}");
                                ExitCode::SUCCESS
                            }
                            Err(error) => {
                                eprintln!("error: {}", error.message);
                                ExitCode::from(1)
                            }
                        },
                        Err(error) => {
                            eprintln!("error: {}", error.message);
                            ExitCode::from(1)
                        }
                    },
                    Err(error) => {
                        eprintln!("error: cannot read {path}: {error}");
                        ExitCode::from(1)
                    }
                }
            } else {
                match axiom_native::parser::parse(&source).and_then(|program| {
                    axiom_native::semantic::analyze(&program)
                        .map(|_| program)
                        .map_err(|error| axiom_native::parser::ParseError {
                            message: error.message,
                            line: 0,
                            column: 0,
                        })
                }) {
                    Ok(program) => match axiom_native::vm::execute_program(&program) {
                        Ok(output) => {
                            print!("{output}");
                            ExitCode::SUCCESS
                        }
                        Err(error) => {
                            eprintln!("error: {}", error.message);
                            ExitCode::from(1)
                        }
                    },
                    Err(error) => {
                        eprintln!(
                            "error: {} at {}:{}",
                            error.message, error.line, error.column
                        );
                        ExitCode::from(1)
                    }
                }
            }
        }
        _ => {
            eprintln!("error: unknown command '{command}'");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!("AXIOM native compiler");
    println!("Usage: axiom <check|build|run> <source.ax>");
}
