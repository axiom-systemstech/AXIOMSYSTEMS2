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
        variables.insert(name.clone(), type_name.clone());
    }
    check_block(
        &function.body,
        &mut variables,
        functions,
        function.return_type.clone(),
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
        Expression::Array(elements) => {
            let mut inferred: Option<Type> = None;
            for element in elements {
                let element_type = check_expression(element, variables, functions)?;
                if let Some(element_type) = element_type {
                    if let Some(current) = inferred.as_ref() {
                        if *current != element_type {
                            return Err(SemanticError {
                                message: "array elements must share the same type".into(),
                            });
                        }
                    } else {
                        inferred = Some(element_type);
                    }
                }
            }
            Ok(inferred.map(|value| Type::Array(Box::new(value))))
        }
        Expression::Variable(name) => variables.get(name).cloned().ok_or_else(|| SemanticError {
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
                BinaryOperator::Add => {
                    if left_type == right_type && (left_type == Some(Type::Int) || left_type == Some(Type::String)) {
                        Ok(left_type.or(Some(Type::Int)))
                    } else if left_type.is_none() && right_type.is_none() {
                        Ok(Some(Type::Int))
                    } else {
                        Err(SemanticError {
                            message: "+ requires Int or String operands".into(),
                        })
                    }
                }
                BinaryOperator::Subtract
                | BinaryOperator::Multiply
                | BinaryOperator::Divide => {
                    require_types(left_type, right_type, Type::Int, "arithmetic operators")?;
                    Ok(Some(Type::Int))
                }
                BinaryOperator::Greater
                | BinaryOperator::GreaterEqual
                | BinaryOperator::Less
                | BinaryOperator::LessEqual => {
                    require_types(left_type, right_type, Type::Int, "comparison operators")?;
                    Ok(Some(Type::Bool))
                }
                BinaryOperator::Equal | BinaryOperator::NotEqual => {
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
        Expression::Unary {
            operator: UnaryOperator::Negate,
            operand,
        } => {
            let operand_type = check_expression(operand, variables, functions)?;
            if operand_type.is_some() && operand_type != Some(Type::Int) {
                return Err(SemanticError {
                    message: "unary '-' requires Int".into(),
                });
            }
            Ok(Some(Type::Int))
        }
        Expression::Index { target, index } => {
            let target_type = check_expression(target, variables, functions)?;
            let index_type = check_expression(index, variables, functions)?;
            if index_type.is_some() && index_type != Some(Type::Int) {
                return Err(SemanticError {
                    message: "array index requires Int".into(),
                });
            }
            match target_type {
                Some(Type::Array(inner)) => Ok(Some(*inner)),
                _ => Err(SemanticError {
                    message: "index requires an array".into(),
                }),
            }
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
            Ok(function.return_type.clone())
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
                    if value_type.is_some() && value_type != Some(declared.clone()) {
                        return Err(SemanticError {
                            message: format!("variable '{name}' has incompatible type"),
                        });
                    }
                    variables.insert(name.clone(), Some(declared.clone()));
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
                check_block(
                    then_body,
                    &mut variables.clone(),
                    functions,
                    return_type.clone(),
                )?;
                check_block(
                    else_body,
                    &mut variables.clone(),
                    functions,
                    return_type.clone(),
                )?;
            }
            Statement::While { condition, body } => {
                require_bool(check_expression(condition, variables, functions)?)?;
                check_block(body, &mut variables.clone(), functions, return_type.clone())?;
            }
            Statement::Assign { target, value } => {
                let target_type = check_expression(target, variables, functions)?;
                let value_type = check_expression(value, variables, functions)?;
                if target_type.is_some() && value_type.is_some() && target_type != value_type {
                    return Err(SemanticError {
                        message: "assignment has incompatible type".into(),
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
    let expected = Some(expected);
    if (left.is_some() && left != expected) || (right.is_some() && right != expected) {
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
