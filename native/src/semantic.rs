use crate::parser::{
    BinaryOperator, Expression, Function, Parameter, Program, Statement, Type, UnaryOperator,
};

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
    for function in &program.functions {
        check_function(function, &program.functions)?;
    }
    check_function(main, &program.functions)
}

fn check_function(function: &Function, functions: &[Function]) -> Result<(), SemanticError> {
    let mut variables = std::collections::HashMap::new();
    for Parameter { name, type_name } in &function.parameters {
        variables.insert(name.clone(), *type_name);
    }
    check_block(
        &function.body,
        &mut variables,
        functions,
        function.return_type,
    )
}

fn check_expression(
    expression: &Expression,
    variables: &std::collections::HashMap<String, Option<Type>>,
    functions: &[Function],
) -> Result<Option<Type>, SemanticError> {
    match expression {
        Expression::String(_) => Ok(Some(Type::String)),
        Expression::Integer(_) => Ok(Some(Type::Int)),
        Expression::Boolean(_) => Ok(Some(Type::Bool)),
        Expression::Variable(name) => variables.get(name).copied().ok_or_else(|| SemanticError {
            message: format!("unknown variable '{name}'"),
        }),
        Expression::Binary {
            left,
            operator,
            right,
        } => {
            let left_type = check_expression(left, variables, functions)?;
            let right_type = check_expression(right, variables, functions)?;
            match operator {
                BinaryOperator::Add
                | BinaryOperator::Subtract
                | BinaryOperator::Multiply
                | BinaryOperator::Divide => {
                    require_types(left_type, right_type, Type::Int, "arithmetic operators")?;
                    Ok(Some(Type::Int))
                }
                BinaryOperator::Greater => {
                    require_types(left_type, right_type, Type::Int, "comparison operators")?;
                    Ok(Some(Type::Bool))
                }
                BinaryOperator::Equal => {
                    if left_type.is_some() && right_type.is_some() && left_type != right_type {
                        return Err(SemanticError {
                            message: "== requires matching types".into(),
                        });
                    }
                    Ok(Some(Type::Bool))
                }
                BinaryOperator::And | BinaryOperator::Or => {
                    require_types(left_type, right_type, Type::Bool, "logical operators")?;
                    Ok(Some(Type::Bool))
                }
            }
        }
        Expression::Unary {
            operator: UnaryOperator::Not,
            operand,
        } => {
            let operand_type = check_expression(operand, variables, functions)?;
            if operand_type.is_some() && operand_type != Some(Type::Bool) {
                return Err(SemanticError {
                    message: "! requires Bool".into(),
                });
            }
            Ok(Some(Type::Bool))
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
            for (argument, parameter) in call.arguments.iter().zip(&function.parameters) {
                let argument_type = check_expression(argument, variables, functions)?;
                if parameter.type_name.is_some()
                    && argument_type.is_some()
                    && parameter.type_name != argument_type
                {
                    return Err(SemanticError {
                        message: format!("argument for '{}' has incompatible type", parameter.name),
                    });
                }
            }
            Ok(function.return_type)
        }
    }
}

fn check_block(
    statements: &[Statement],
    variables: &mut std::collections::HashMap<String, Option<Type>>,
    functions: &[Function],
    return_type: Option<Type>,
) -> Result<(), SemanticError> {
    for statement in statements {
        match statement {
            Statement::Let { name, value, .. } => {
                let value_type = check_expression(value, variables, functions)?;
                if let Statement::Let {
                    type_name: Some(declared),
                    ..
                } = statement
                {
                    if value_type.is_some() && value_type != Some(*declared) {
                        return Err(SemanticError {
                            message: format!("variable '{name}' has incompatible type"),
                        });
                    }
                    variables.insert(name.clone(), Some(*declared));
                } else {
                    variables.insert(name.clone(), value_type);
                }
            }
            Statement::Return(value) => {
                let value_type = check_expression(value, variables, functions)?;
                if return_type.is_some() && value_type.is_some() && return_type != value_type {
                    return Err(SemanticError {
                        message: "return value has incompatible type".into(),
                    });
                }
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                require_bool(check_expression(condition, variables, functions)?)?;
                check_block(then_body, &mut variables.clone(), functions, return_type)?;
                check_block(else_body, &mut variables.clone(), functions, return_type)?;
            }
            Statement::While { condition, body } => {
                require_bool(check_expression(condition, variables, functions)?)?;
                check_block(body, &mut variables.clone(), functions, return_type)?;
            }
            Statement::Assign { name, value } => {
                let existing = variables.get(name).copied().ok_or_else(|| SemanticError {
                    message: format!("unknown variable '{name}'"),
                })?;
                let value_type = check_expression(value, variables, functions)?;
                if existing.is_some() && value_type.is_some() && existing != value_type {
                    return Err(SemanticError {
                        message: format!("assignment to '{name}' has incompatible type"),
                    });
                }
            }
            Statement::Call(call) if call.name == "print" && call.arguments.len() == 1 => {
                check_expression(&call.arguments[0], variables, functions)?;
            }
            Statement::Call(call) => {
                check_expression(&Expression::Call(call.clone()), variables, functions)?;
            }
        }
    }
    Ok(())
}

fn require_types(
    left: Option<Type>,
    right: Option<Type>,
    expected: Type,
    operation: &str,
) -> Result<(), SemanticError> {
    if (left.is_some() && left != Some(expected)) || (right.is_some() && right != Some(expected)) {
        return Err(SemanticError {
            message: format!("{operation} require {:?}", expected),
        });
    }
    Ok(())
}

fn require_bool(value: Option<Type>) -> Result<(), SemanticError> {
    if value.is_some() && value != Some(Type::Bool) {
        return Err(SemanticError {
            message: "condition requires Bool".into(),
        });
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

    #[test]
    fn rejects_mismatched_variable_type() {
        let error = analyze(&crate::parser::parse("fn main() { let value: Int = true }").unwrap())
            .unwrap_err();
        assert!(error.message.contains("incompatible type"));
    }

    #[test]
    fn rejects_non_boolean_condition() {
        let error = analyze(&crate::parser::parse("fn main() { if 1 { print(\"no\") } }").unwrap())
            .unwrap_err();
        assert_eq!(error.message, "condition requires Bool");
    }
}
