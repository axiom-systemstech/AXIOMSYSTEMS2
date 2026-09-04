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

    let functions = &program.functions;
    let mut variables = std::collections::HashSet::new();
    for parameter in &main.parameters {
        variables.insert(parameter.clone());
    }
    check_block(&main.body, &mut variables, functions)
}

fn check_expression(
    expression: &Expression,
    variables: &std::collections::HashSet<String>,
    functions: &[crate::parser::Function],
) -> Result<(), SemanticError> {
    match expression {
        Expression::Variable(name) if !variables.contains(name) => Err(SemanticError {
            message: format!("unknown variable '{name}'"),
        }),
        Expression::Binary { left, right, .. } => {
            check_expression(left, variables, functions)?;
            check_expression(right, variables, functions)
        }
        Expression::Call(call) => {
            let function = functions
                .iter()
                .find(|function| function.name == call.name)
                .ok_or_else(|| SemanticError {
                    message: format!("unknown function '{}'", call.name),
                })?;
            if call.arguments.len() != function.parameters.len() {
                return Err(SemanticError {
                    message: format!(
                        "function '{}' expects {} arguments",
                        call.name,
                        function.parameters.len()
                    ),
                });
            }
            for argument in &call.arguments {
                check_expression(argument, variables, functions)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn check_block(
    statements: &[Statement],
    variables: &mut std::collections::HashSet<String>,
    functions: &[crate::parser::Function],
) -> Result<(), SemanticError> {
    for statement in statements {
        match statement {
            Statement::Let { name, value } => {
                check_expression(value, variables, functions)?;
                variables.insert(name.clone());
            }
            Statement::Return(value) => check_expression(value, variables, functions)?,
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                check_expression(condition, variables, functions)?;
                check_block(then_body, &mut variables.clone(), functions)?;
                check_block(else_body, &mut variables.clone(), functions)?;
            }
            Statement::While { condition, body } => {
                check_expression(condition, variables, functions)?;
                check_block(body, &mut variables.clone(), functions)?;
            }
            Statement::Assign { name, value } => {
                if !variables.contains(name) {
                    return Err(SemanticError {
                        message: format!("unknown variable '{name}'"),
                    });
                }
                check_expression(value, variables, functions)?;
            }
            Statement::Call(call) if call.name == "print" && call.arguments.len() == 1 => {
                check_expression(&call.arguments[0], variables, functions)?;
            }
            Statement::Call(call) => {
                let function = functions
                    .iter()
                    .find(|function| function.name == call.name)
                    .ok_or_else(|| SemanticError {
                        message: format!("unknown function '{}'", call.name),
                    })?;
                if call.arguments.len() != function.parameters.len() {
                    return Err(SemanticError {
                        message: format!(
                            "function '{}' expects {} arguments",
                            call.name,
                            function.parameters.len()
                        ),
                    });
                }
            }
        }
    }
    Ok(())
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
