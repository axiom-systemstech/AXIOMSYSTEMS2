use crate::parser::{Expression, Program, Statement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub message: String,
}

pub fn analyze(program: &Program) -> Result<(), SemanticError> {
    let main = program
        .functions
        .iter()
        .find(|function| function.name == "main")
        .ok_or_else(|| SemanticError {
            message: "program must define 'main'".into(),
        })?;

    let mut variables = std::collections::HashSet::new();
    for statement in &main.body {
        match statement {
            Statement::Let { name, value } => {
                check_expression(value, &variables)?;
                variables.insert(name);
            }
            Statement::Call(call) if call.name == "print" && call.arguments.len() == 1 => {
                check_expression(&call.arguments[0], &variables)?;
            }
            Statement::Call(call) if call.name == "print" => {
                return Err(SemanticError {
                    message: "print expects exactly one argument".into(),
                });
            }
            Statement::Call(call) => {
                return Err(SemanticError {
                    message: format!("unknown function '{}'", call.name),
                });
            }
        }
    }
    Ok(())
}

fn check_expression(
    expression: &Expression,
    variables: &std::collections::HashSet<&String>,
) -> Result<(), SemanticError> {
    match expression {
        Expression::Variable(name) if !variables.iter().any(|value| value.as_str() == name) => {
            Err(SemanticError {
                message: format!("unknown variable '{name}'"),
            })
        }
        Expression::Binary { left, right, .. } => {
            check_expression(left, variables)?;
            check_expression(right, variables)
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn accepts_hello_program() {
        analyze(&parse("fn main() { print(\"Hello AXIOM\") }").unwrap()).unwrap();
    }

    #[test]
    fn requires_main() {
        let error = analyze(&parse("fn start() { print(\"Hello AXIOM\") }").unwrap()).unwrap_err();
        assert_eq!(error.message, "program must define 'main'");
    }
}
