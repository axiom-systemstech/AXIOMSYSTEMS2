use crate::parser::{Expression, Program, Statement};
use crate::{parser, semantic};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
}

pub fn run(source: &str) -> Result<String, RuntimeError> {
    let program = parser::parse(source).map_err(|error| RuntimeError {
        message: format!("{} at {}:{}", error.message, error.line, error.column),
    })?;
    semantic::analyze(&program).map_err(|error| RuntimeError {
        message: error.message,
    })?;
    execute(&program)
}

pub fn execute(program: &Program) -> Result<String, RuntimeError> {
    let main = program
        .functions
        .iter()
        .find(|function| function.name == "main")
        .ok_or_else(|| RuntimeError {
            message: "program must define 'main'".into(),
        })?;
    let mut output = String::new();
    for statement in &main.body {
        let Statement::Call(call) = statement;
        if call.name == "print" {
            let value = match &call.arguments[0] {
                Expression::String(value) => value.clone(),
                Expression::Integer(value) => value.to_string(),
                Expression::Boolean(value) => value.to_string(),
            };
            output.push_str(&value);
            output.push('\n');
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_hello_program() {
        assert_eq!(
            run("fn main() { print(\"Hello AXIOM\") }").unwrap(),
            "Hello AXIOM\n"
        );
    }

    #[test]
    fn runs_typed_literals() {
        assert_eq!(
            run("fn main() { print(42); print(true) }").unwrap(),
            "42\ntrue\n"
        );
    }
}
