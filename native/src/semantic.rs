use crate::parser::{
    BinaryOperator, Expression, Function, Parameter, Program, Statement, Type, UnaryOperator,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub message: String,
}

pub fn analyze(program: &Program) -> Result<(), SemanticError> {
    validate_structs(program)?;
    if !program
        .functions
        .iter()
        .any(|function| function.name == "main")
    {
        return Err(SemanticError {
            message: "program must define 'main'".into(),
        });
    }
    for function in &program.functions {
        check_function(function, &program.functions, &program.structs)?;
    }
    Ok(())
}

fn validate_structs(program: &Program) -> Result<(), SemanticError> {
    let mut names = std::collections::HashSet::new();
    for structure in &program.structs {
        if !names.insert(&structure.name) {
            return Err(SemanticError {
                message: format!("duplicate struct '{}'", structure.name),
            });
        }
        let mut fields = std::collections::HashSet::new();
        for field in &structure.fields {
            if !fields.insert(&field.name) {
                return Err(SemanticError {
                    message: format!("struct '{}' has duplicate fields", structure.name),
                });
            }
        }
    }
    Ok(())
}

fn check_function(
    function: &Function,
    functions: &[Function],
    structs: &[crate::parser::StructDefinition],
) -> Result<(), SemanticError> {
    let mut variables = std::collections::HashMap::new();
    for Parameter { name, type_name } in &function.parameters {
        variables.insert(name.clone(), type_name.clone());
    }
    check_block(
        &function.body,
        &mut variables,
        functions,
        structs,
        function.return_type.clone(),
        0,
    )?;
    if function.return_type.is_some()
        && !function
            .body
            .iter()
            .any(|statement| matches!(statement, Statement::Return(_)))
    {
        return Err(SemanticError {
            message: format!("function '{}' must return a value", function.name),
        });
    }
    Ok(())
}

fn check_expression(
    expression: &Expression,
    variables: &std::collections::HashMap<String, Option<Type>>,
    functions: &[Function],
    structs: &[crate::parser::StructDefinition],
) -> Result<Option<Type>, SemanticError> {
    match expression {
        Expression::String(_) => Ok(Some(Type::String)),
        Expression::Integer(_) => Ok(Some(Type::Int)),
        Expression::Float(_) => Ok(Some(Type::Float)),
        Expression::Boolean(_) => Ok(Some(Type::Bool)),
        Expression::Array(elements) => {
            let mut inferred: Option<Type> = None;
            for element in elements {
                let element_type = check_expression(element, variables, functions, structs)?;
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
        Expression::StructLiteral { type_name, fields } => {
            let structure = structs
                .iter()
                .find(|structure| structure.name == *type_name)
                .ok_or_else(|| SemanticError {
                    message: format!("unknown struct '{type_name}'"),
                })?;
            let mut seen = std::collections::HashSet::new();
            for (field_name, value) in fields {
                if !seen.insert(field_name) {
                    return Err(SemanticError {
                        message: format!("struct '{type_name}' has duplicate fields"),
                    });
                }
                let field = structure
                    .fields
                    .iter()
                    .find(|field| field.name == *field_name)
                    .ok_or_else(|| SemanticError {
                        message: format!("unknown field '{field_name}'"),
                    })?;
                let value_type = check_expression(value, variables, functions, structs)?;
                if value_type.is_some() && value_type != field.type_name {
                    return Err(SemanticError {
                        message: format!("field '{field_name}' has incompatible type"),
                    });
                }
            }
            if fields.len() != structure.fields.len() {
                return Err(SemanticError {
                    message: format!("struct '{type_name}' has incompatible fields"),
                });
            }
            Ok(Some(Type::Named(type_name.clone())))
        }
        Expression::Variable(name) => variables.get(name).cloned().ok_or_else(|| SemanticError {
            message: format!("unknown variable '{name}'"),
        }),
        Expression::Binary {
            left,
            operator,
            right,
        } => {
            let left_type = check_expression(left, variables, functions, structs)?;
            let right_type = check_expression(right, variables, functions, structs)?;
            match operator {
                BinaryOperator::Add => {
                    if left_type == right_type
                        && (left_type == Some(Type::Int)
                            || left_type == Some(Type::Float)
                            || left_type == Some(Type::String))
                    {
                        Ok(left_type.or(Some(Type::Int)))
                    } else if left_type.is_none() && right_type.is_none() {
                        Ok(Some(Type::Int))
                    } else {
                        Err(SemanticError {
                            message: "+ requires matching Int, Float, or String operands".into(),
                        })
                    }
                }
                BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide => {
                    if left_type == Some(Type::Int) && right_type == Some(Type::Int) {
                        Ok(Some(Type::Int))
                    } else if left_type == Some(Type::Float) && right_type == Some(Type::Float) {
                        Ok(Some(Type::Float))
                    } else {
                        Err(SemanticError {
                            message: "arithmetic operators require matching numeric operands"
                                .into(),
                        })
                    }
                }
                BinaryOperator::Modulo => {
                    require_types(left_type, right_type, Type::Int, "modulo operator")?;
                    Ok(Some(Type::Int))
                }
                BinaryOperator::Greater
                | BinaryOperator::GreaterEqual
                | BinaryOperator::Less
                | BinaryOperator::LessEqual => {
                    if (left_type == Some(Type::Int) && right_type == Some(Type::Int))
                        || (left_type == Some(Type::Float) && right_type == Some(Type::Float))
                    {
                        Ok(Some(Type::Bool))
                    } else {
                        Err(SemanticError {
                            message: "comparison operators require matching numeric operands"
                                .into(),
                        })
                    }
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
            let operand_type = check_expression(operand, variables, functions, structs)?;
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
            let operand_type = check_expression(operand, variables, functions, structs)?;
            if operand_type.is_some()
                && operand_type != Some(Type::Int)
                && operand_type != Some(Type::Float)
            {
                return Err(SemanticError {
                    message: "unary '-' requires Int or Float".into(),
                });
            }
            Ok(operand_type)
        }
        Expression::Index { target, index } => {
            let target_type = check_expression(target, variables, functions, structs)?;
            let index_type = check_expression(index, variables, functions, structs)?;
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
        Expression::FieldAccess { target, field } => {
            let target_type = check_expression(target, variables, functions, structs)?;
            let Some(Type::Named(type_name)) = target_type else {
                return Err(SemanticError {
                    message: "field access requires a struct".into(),
                });
            };
            let structure = structs
                .iter()
                .find(|structure| structure.name == type_name)
                .ok_or_else(|| SemanticError {
                    message: format!("unknown struct '{type_name}'"),
                })?;
            let field_definition = structure
                .fields
                .iter()
                .find(|field_definition| field_definition.name == *field)
                .ok_or_else(|| SemanticError {
                    message: format!("unknown field '{field}'"),
                })?;
            Ok(field_definition.type_name.clone())
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
                let argument_type = check_expression(argument, variables, functions, structs)?;
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
    structs: &[crate::parser::StructDefinition],
    return_type: Option<Type>,
    loop_depth: usize,
) -> Result<(), SemanticError> {
    for statement in statements {
        match statement {
            Statement::Let { name, value, .. } => {
                let value_type = check_expression(value, variables, functions, structs)?;
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
                if return_type.is_none() {
                    return Err(SemanticError {
                        message: "function cannot return a value".into(),
                    });
                }
                let value_type = check_expression(value, variables, functions, structs)?;
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
                require_bool(check_expression(condition, variables, functions, structs)?)?;
                check_block(
                    then_body,
                    &mut variables.clone(),
                    functions,
                    structs,
                    return_type.clone(),
                    loop_depth,
                )?;
                check_block(
                    else_body,
                    &mut variables.clone(),
                    functions,
                    structs,
                    return_type.clone(),
                    loop_depth,
                )?;
            }
            Statement::While { condition, body } => {
                require_bool(check_expression(condition, variables, functions, structs)?)?;
                check_block(
                    body,
                    &mut variables.clone(),
                    functions,
                    structs,
                    return_type.clone(),
                    loop_depth + 1,
                )?;
            }
            Statement::For {
                initializer,
                condition,
                update,
                body,
            } => {
                let mut scoped = variables.clone();
                if let Some(initializer) = initializer {
                    check_block(
                        std::slice::from_ref(initializer),
                        &mut scoped,
                        functions,
                        structs,
                        return_type.clone(),
                        loop_depth,
                    )?;
                }
                require_bool(check_expression(condition, &scoped, functions, structs)?)?;
                if let Some(update) = update {
                    check_block(
                        std::slice::from_ref(update),
                        &mut scoped,
                        functions,
                        structs,
                        return_type.clone(),
                        loop_depth,
                    )?;
                }
                check_block(
                    body,
                    &mut scoped,
                    functions,
                    structs,
                    return_type.clone(),
                    loop_depth + 1,
                )?;
            }
            Statement::Break if loop_depth > 0 => {}
            Statement::Continue if loop_depth > 0 => {}
            Statement::Break => {
                return Err(SemanticError {
                    message: "break must be inside a loop".into(),
                });
            }
            Statement::Continue => {
                return Err(SemanticError {
                    message: "continue must be inside a loop".into(),
                });
            }
            Statement::Assign { target, value } => {
                let target_type = check_expression(target, variables, functions, structs)?;
                let value_type = check_expression(value, variables, functions, structs)?;
                if target_type.is_some() && value_type.is_some() && target_type != value_type {
                    return Err(SemanticError {
                        message: "assignment has incompatible type".into(),
                    });
                }
            }
            Statement::Call(call) if call.name == "print" && call.arguments.len() == 1 => {
                check_expression(&call.arguments[0], variables, functions, structs)?;
            }
            Statement::Call(call) => {
                check_expression(
                    &Expression::Call(call.clone()),
                    variables,
                    functions,
                    structs,
                )?;
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

    #[test]
    fn rejects_loop_control_outside_loop() {
        for (keyword, expected) in [
            ("break", "break must be inside a loop"),
            ("continue", "continue must be inside a loop"),
        ] {
            let error =
                analyze(&parse(&format!("fn main() {{ if true {{ {keyword} }} }}")).unwrap())
                    .unwrap_err();
            assert_eq!(error.message, expected);
        }
    }

    #[test]
    fn rejects_return_without_declared_type() {
        let error = analyze(&parse("fn main() { return 42 }").unwrap()).unwrap_err();
        assert_eq!(error.message, "function cannot return a value");
    }

    #[test]
    fn rejects_declared_return_without_return_statement() {
        let error =
            analyze(&parse("fn answer() -> Int { print(1) } fn main() { print(0) }").unwrap())
                .unwrap_err();
        assert_eq!(error.message, "function 'answer' must return a value");
    }

    #[test]
    fn accepts_valid_struct_access() {
        let source = "struct Point { x: Int y: Int } fn main() { let point: Point = Point { x: 10 y: 20 } print(point.x) }";
        analyze(&parse(source).unwrap()).unwrap();
    }

    #[test]
    fn rejects_unknown_struct_field() {
        let source = "struct Point { x: Int } fn main() { let point: Point = Point { x: 10 } print(point.y) }";
        let error = analyze(&parse(source).unwrap()).unwrap_err();
        assert_eq!(error.message, "unknown field 'y'");
    }

    #[test]
    fn rejects_struct_field_type_mismatch() {
        let source = "struct Point { x: Int } fn main() { let point: Point = Point { x: true } }";
        let error = analyze(&parse(source).unwrap()).unwrap_err();
        assert_eq!(error.message, "field 'x' has incompatible type");
    }

    #[test]
    fn rejects_duplicate_struct_fields() {
        let source =
            "struct Point { x: Int } fn main() { let point: Point = Point { x: 10 x: 20 } }";
        let error = analyze(&parse(source).unwrap()).unwrap_err();
        assert_eq!(error.message, "struct 'Point' has duplicate fields");
    }

    #[test]
    fn rejects_duplicate_struct_definitions() {
        let source = "struct Point { x: Int } struct Point { y: Int } fn main() { print(1) }";
        let error = analyze(&parse(source).unwrap()).unwrap_err();
        assert_eq!(error.message, "duplicate struct 'Point'");
    }

    #[test]
    fn rejects_field_access_on_non_struct() {
        let source = "fn main() { let value: Int = 10 print(value.x) }";
        let error = analyze(&parse(source).unwrap()).unwrap_err();
        assert_eq!(error.message, "field access requires a struct");
    }

    #[test]
    fn rejects_mixed_integer_and_float_addition() {
        let error = analyze(&parse("fn main() { print(1 + 2.0) }").unwrap()).unwrap_err();
        assert_eq!(
            error.message,
            "+ requires matching Int, Float, or String operands"
        );
    }
}
