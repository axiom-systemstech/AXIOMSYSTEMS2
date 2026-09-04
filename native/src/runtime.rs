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
    let mut variables = std::collections::HashMap::new();
    for statement in &main.body {
        match statement {
            Statement::Let { name, value } => {
                variables.insert(name.clone(), evaluate(value, &variables)?);
            }
            Statement::Call(call) if call.name == "print" => {
                let value = evaluate(&call.arguments[0], &variables)?;
                output.push_str(&value);
                output.push('\n');
            }
            Statement::Call(_) => {}
        }
    }
    Ok(output)
}

fn evaluate(
    expression: &Expression,
    variables: &std::collections::HashMap<String, String>,
) -> Result<String, RuntimeError> {
    match expression {
        Expression::String(value) => Ok(value.clone()),
        Expression::Integer(value) => Ok(value.to_string()),
        Expression::Boolean(value) => Ok(value.to_string()),
        Expression::Variable(name) => variables.get(name).cloned().ok_or_else(|| RuntimeError {
            message: format!("unknown variable '{name}'"),
        }),
        Expression::Binary { left, right, .. } => {
            let left = evaluate(left, variables)?
                .parse::<i64>()
                .map_err(|_| RuntimeError {
                    message: "+ expects integers".into(),
                })?;
            let right = evaluate(right, variables)?
                .parse::<i64>()
                .map_err(|_| RuntimeError {
                    message: "+ expects integers".into(),
                })?;
            Ok((left + right).to_string())
        }
    }
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

    #[test]
    fn runs_variable_and_addition() {
        assert_eq!(
            run("fn main() { let total = 20 + 22; print(total) }").unwrap(),
            "42\n"
        );
    }
}
