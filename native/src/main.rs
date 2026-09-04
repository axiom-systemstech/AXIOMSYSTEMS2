use std::env;
use std::fs;
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
        "run" => match axiom_native::runtime::run(&source) {
            Ok(output) => {
                print!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error: {}", error.message);
                ExitCode::from(1)
            }
        },
        _ => {
            eprintln!("error: unknown command '{command}'");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!("AXIOM native compiler");
    println!("Usage: axiom <check|run> <source.ax>");
}
