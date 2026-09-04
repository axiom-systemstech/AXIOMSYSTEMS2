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
    let mut output = String::new();
    let functions = &program.functions;
    invoke("main", Vec::new(), functions, &mut output)?;
    Ok(output)
}

fn invoke(
    name: &str,
    arguments: Vec<String>,
    functions: &[crate::parser::Function],
    output: &mut String,
) -> Result<Option<String>, RuntimeError> {
    let function = functions
        .iter()
        .find(|function| function.name == name)
        .ok_or_else(|| RuntimeError {
            message: format!("unknown function '{name}'"),
        })?;
    let mut variables = std::collections::HashMap::new();
    for (parameter, value) in function.parameters.iter().zip(arguments) {
        variables.insert(parameter.clone(), value);
    }
    for statement in &function.body {
        match statement {
            Statement::Let { name, value } => {
                let evaluated = evaluate(value, &variables, functions, output)?;
                variables.insert(name.clone(), evaluated);
            }
            Statement::Call(call) if call.name == "print" => {
                let value = evaluate(&call.arguments[0], &variables, functions, output)?;
                output.push_str(&value);
                output.push('\n');
            }
            Statement::Call(call) => {
                let arguments = call
                    .arguments
                    .iter()
                    .map(|argument| evaluate(argument, &variables, functions, output))
                    .collect::<Result<Vec<_>, _>>()?;
                invoke(&call.name, arguments, functions, output)?;
            }
            Statement::Return(value) => {
                return Ok(Some(evaluate(value, &variables, functions, output)?))
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                let branch = if evaluate(condition, &variables, functions, output)? == "true" {
                    then_body
                } else {
                    else_body
                };
                if let Some(value) = execute_block(branch, &mut variables, functions, output)? {
                    return Ok(Some(value));
                }
            }
        }
    }
    Ok(None)
}

fn execute_block(
    statements: &[Statement],
    variables: &mut std::collections::HashMap<String, String>,
    functions: &[crate::parser::Function],
    output: &mut String,
) -> Result<Option<String>, RuntimeError> {
    for statement in statements {
        match statement {
            Statement::Let { name, value } => {
                let evaluated = evaluate(value, variables, functions, output)?;
                variables.insert(name.clone(), evaluated);
            }
            Statement::Return(value) => {
                return Ok(Some(evaluate(value, variables, functions, output)?))
            }
            Statement::Call(call) if call.name == "print" => {
                let value = evaluate(&call.arguments[0], variables, functions, output)?;
                output.push_str(&value);
                output.push('\n');
            }
            Statement::Call(call) => {
                let arguments = call
                    .arguments
                    .iter()
                    .map(|argument| evaluate(argument, variables, functions, output))
                    .collect::<Result<Vec<_>, _>>()?;
                invoke(&call.name, arguments, functions, output)?;
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                let branch = if evaluate(condition, variables, functions, output)? == "true" {
                    then_body
                } else {
                    else_body
                };
                if let Some(value) = execute_block(branch, variables, functions, output)? {
                    return Ok(Some(value));
                }
            }
        }
    }
    Ok(None)
}

fn evaluate(
    expression: &Expression,
    variables: &std::collections::HashMap<String, String>,
    functions: &[crate::parser::Function],
    output: &mut String,
) -> Result<String, RuntimeError> {
    match expression {
        Expression::String(value) => Ok(value.clone()),
        Expression::Integer(value) => Ok(value.to_string()),
        Expression::Boolean(value) => Ok(value.to_string()),
        Expression::Variable(name) => variables.get(name).cloned().ok_or_else(|| RuntimeError {
            message: format!("unknown variable '{name}'"),
        }),
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::Add,
        } => {
            let left = evaluate(left, variables, functions, output)?
                .parse::<i64>()
                .map_err(|_| RuntimeError {
                    message: "+ expects integers".into(),
                })?;
            let right = evaluate(right, variables, functions, output)?
                .parse::<i64>()
                .map_err(|_| RuntimeError {
                    message: "+ expects integers".into(),
                })?;
            Ok((left + right).to_string())
        }
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::Greater,
        } => {
            let left = evaluate(left, variables, functions, output)?
                .parse::<i64>()
                .map_err(|_| RuntimeError {
                    message: "> expects integers".into(),
                })?;
            let right = evaluate(right, variables, functions, output)?
                .parse::<i64>()
                .map_err(|_| RuntimeError {
                    message: "> expects integers".into(),
                })?;
            Ok((left > right).to_string())
        }
        Expression::Call(call) => {
            let arguments = call
                .arguments
                .iter()
                .map(|argument| evaluate(argument, variables, functions, output))
                .collect::<Result<Vec<_>, _>>()?;
            invoke(&call.name, arguments, functions, output)?.ok_or_else(|| RuntimeError {
                message: format!("function '{}' returned no value", call.name),
            })
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

    #[test]
    fn runs_function_with_parameters_and_return() {
        assert_eq!(
            run("fn add(a, b) { return a + b } fn main() { print(add(20, 22)) }").unwrap(),
            "42\n"
        );
    }

    #[test]
    fn runs_if_else_with_comparison() {
        assert_eq!(
            run("fn main() { if 2 > 1 { print(\"yes\") } else { print(\"no\") } }").unwrap(),
            "yes\n"
        );
    }
}
