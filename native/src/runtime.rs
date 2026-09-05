use crate::parser::{Expression, Program, Statement};
use crate::{parser, semantic};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
}

enum Control {
    None,
    Return(String),
    Break,
    Continue,
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
        variables.insert(parameter.name.clone(), value);
    }
    for statement in &function.body {
        match statement {
            Statement::Let { name, value, .. } => {
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
                match execute_block(branch, &mut variables, functions, output)? {
                    Control::None => {}
                    Control::Return(value) => return Ok(Some(value)),
                    Control::Break | Control::Continue => {
                        return Err(RuntimeError {
                            message: "loop control escaped its loop".into(),
                        })
                    }
                }
            }
            Statement::Assign { target, value } => {
                let evaluated = evaluate(value, &variables, functions, output)?;
                assign_target(target, evaluated, &mut variables, functions, output)?;
            }
            Statement::While { condition, body } => {
                while evaluate(condition, &variables, functions, output)? == "true" {
                    match execute_block(body, &mut variables, functions, output)? {
                        Control::None | Control::Continue => {}
                        Control::Break => break,
                        Control::Return(value) => return Ok(Some(value)),
                    }
                }
            }
            Statement::For {
                initializer,
                condition,
                update,
                body,
            } => {
                if let Some(initializer) = initializer {
                    match execute_block(
                        std::slice::from_ref(initializer),
                        &mut variables,
                        functions,
                        output,
                    )? {
                        Control::None => {}
                        Control::Return(value) => return Ok(Some(value)),
                        Control::Break | Control::Continue => {
                            return Err(RuntimeError {
                                message: "loop control escaped its loop".into(),
                            })
                        }
                    }
                }
                while evaluate(condition, &variables, functions, output)? == "true" {
                    match execute_block(body, &mut variables, functions, output)? {
                        Control::None | Control::Continue => {}
                        Control::Break => break,
                        Control::Return(value) => return Ok(Some(value)),
                    }
                    if let Some(update) = update {
                        match execute_block(
                            std::slice::from_ref(update),
                            &mut variables,
                            functions,
                            output,
                        )? {
                            Control::None => {}
                            Control::Return(value) => return Ok(Some(value)),
                            Control::Break | Control::Continue => {
                                return Err(RuntimeError {
                                    message: "loop control escaped its loop".into(),
                                })
                            }
                        }
                    }
                }
            }
            Statement::Break | Statement::Continue => {
                return Err(RuntimeError {
                    message: "loop control escaped its loop".into(),
                })
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
) -> Result<Control, RuntimeError> {
    for statement in statements {
        match statement {
            Statement::Let { name, value, .. } => {
                let evaluated = evaluate(value, variables, functions, output)?;
                variables.insert(name.clone(), evaluated);
            }
            Statement::Return(value) => {
                return Ok(Control::Return(evaluate(
                    value, variables, functions, output,
                )?))
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
                match execute_block(branch, variables, functions, output)? {
                    Control::None => {}
                    flow => return Ok(flow),
                }
            }
            Statement::Assign { target, value } => {
                let evaluated = evaluate(value, variables, functions, output)?;
                assign_target(target, evaluated, variables, functions, output)?;
            }
            Statement::While { condition, body } => {
                while evaluate(condition, variables, functions, output)? == "true" {
                    match execute_block(body, variables, functions, output)? {
                        Control::None | Control::Continue => {}
                        Control::Break => break,
                        flow => return Ok(flow),
                    }
                }
            }
            Statement::For {
                initializer,
                condition,
                update,
                body,
            } => {
                if let Some(initializer) = initializer {
                    match execute_block(
                        std::slice::from_ref(initializer),
                        variables,
                        functions,
                        output,
                    )? {
                        Control::None => {}
                        flow => return Ok(flow),
                    }
                }
                while evaluate(condition, variables, functions, output)? == "true" {
                    match execute_block(body, variables, functions, output)? {
                        Control::None | Control::Continue => {}
                        Control::Break => break,
                        flow => return Ok(flow),
                    }
                    if let Some(update) = update {
                        match execute_block(
                            std::slice::from_ref(update),
                            variables,
                            functions,
                            output,
                        )? {
                            Control::None => {}
                            flow => return Ok(flow),
                        }
                    }
                }
            }
            Statement::Break => return Ok(Control::Break),
            Statement::Continue => return Ok(Control::Continue),
        }
    }
    Ok(Control::None)
}

fn assign_target(
    target: &Expression,
    value: String,
    variables: &mut std::collections::HashMap<String, String>,
    functions: &[crate::parser::Function],
    output: &mut String,
) -> Result<(), RuntimeError> {
    match target {
        Expression::Variable(name) => {
            variables.insert(name.clone(), value);
            Ok(())
        }
        Expression::Index { target, index } => {
            let target_value = evaluate(target, variables, functions, output)?;
            let index = integer(index, variables, functions, output)?;
            let mut elements = parse_array_literal(&target_value)?;
            if index < 0 || index >= elements.len() as i64 {
                return Err(RuntimeError {
                    message: "index out of bounds".into(),
                });
            }
            elements[index as usize] = value;
            let updated = format!("[{}]", elements.join(", "));
            match &**target {
                Expression::Variable(name) => {
                    variables.insert(name.clone(), updated);
                    Ok(())
                }
                _ => assign_target(target, updated, variables, functions, output),
            }
        }
        _ => Err(RuntimeError {
            message: "invalid assignment target".into(),
        }),
    }
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
        Expression::Float(value) => Ok(value.clone()),
        Expression::Boolean(value) => Ok(value.to_string()),
        Expression::Array(values) => {
            let items = values
                .iter()
                .map(|value| evaluate(value, variables, functions, output))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("[{}]", items.join(", ")))
        }
        Expression::Variable(name) => variables.get(name).cloned().ok_or_else(|| RuntimeError {
            message: format!("unknown variable '{name}'"),
        }),
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::Add,
        } => {
            let left = evaluate(left, variables, functions, output)?;
            let right = evaluate(right, variables, functions, output)?;
            if let (Ok(left_value), Ok(right_value)) = (left.parse::<i64>(), right.parse::<i64>()) {
                Ok((left_value + right_value).to_string())
            } else if let (Ok(left_value), Ok(right_value)) =
                (left.parse::<f64>(), right.parse::<f64>())
            {
                Ok(render_float(left_value + right_value))
            } else {
                Ok(format!("{left}{right}"))
            }
        }
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::Subtract,
        } => numeric_binary(left, right, variables, functions, output, |left, right| {
            left - right
        }),
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::Multiply,
        } => numeric_binary(left, right, variables, functions, output, |left, right| {
            left * right
        }),
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::Divide,
        } => {
            let left_value = evaluate(left, variables, functions, output)?;
            let right_value = evaluate(right, variables, functions, output)?;
            let divisor = right_value.parse::<f64>().map_err(|_| RuntimeError {
                message: "arithmetic operators expect numbers".into(),
            })?;
            if divisor == 0.0 {
                return Err(RuntimeError {
                    message: "division by zero".into(),
                });
            }
            if let (Ok(left), Ok(right)) = (left_value.parse::<i64>(), right_value.parse::<i64>()) {
                Ok((left / right).to_string())
            } else {
                Ok(render_float(
                    left_value.parse::<f64>().map_err(|_| RuntimeError {
                        message: "arithmetic operators expect numbers".into(),
                    })? / divisor,
                ))
            }
        }
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::Greater,
        } => {
            let left = evaluate(left, variables, functions, output)?
                .parse::<f64>()
                .map_err(|_| RuntimeError {
                    message: "> expects numbers".into(),
                })?;
            let right = evaluate(right, variables, functions, output)?
                .parse::<f64>()
                .map_err(|_| RuntimeError {
                    message: "> expects numbers".into(),
                })?;
            Ok((left > right).to_string())
        }
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::Less,
        } => {
            let left = evaluate(left, variables, functions, output)?
                .parse::<f64>()
                .map_err(|_| RuntimeError {
                    message: "< expects numbers".into(),
                })?;
            let right = evaluate(right, variables, functions, output)?
                .parse::<f64>()
                .map_err(|_| RuntimeError {
                    message: "< expects numbers".into(),
                })?;
            Ok((left < right).to_string())
        }
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::GreaterEqual,
        } => Ok((evaluate(left, variables, functions, output)?
            .parse::<f64>()
            .map_err(|_| RuntimeError {
                message: ">= expects numbers".into(),
            })?
            >= evaluate(right, variables, functions, output)?
                .parse::<f64>()
                .map_err(|_| RuntimeError {
                    message: ">= expects numbers".into(),
                })?)
        .to_string()),
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::LessEqual,
        } => Ok((evaluate(left, variables, functions, output)?
            .parse::<f64>()
            .map_err(|_| RuntimeError {
                message: "<= expects numbers".into(),
            })?
            <= evaluate(right, variables, functions, output)?
                .parse::<f64>()
                .map_err(|_| RuntimeError {
                    message: "<= expects numbers".into(),
                })?)
        .to_string()),
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::Equal,
        } => Ok((evaluate(left, variables, functions, output)?
            == evaluate(right, variables, functions, output)?)
        .to_string()),
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::NotEqual,
        } => Ok((evaluate(left, variables, functions, output)?
            != evaluate(right, variables, functions, output)?)
        .to_string()),
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::And,
        } => Ok((evaluate(left, variables, functions, output)? == "true"
            && evaluate(right, variables, functions, output)? == "true")
            .to_string()),
        Expression::Binary {
            left,
            right,
            operator: crate::parser::BinaryOperator::Or,
        } => Ok((evaluate(left, variables, functions, output)? == "true"
            || evaluate(right, variables, functions, output)? == "true")
            .to_string()),
        Expression::Unary {
            operator: crate::parser::UnaryOperator::Not,
            operand,
        } => Ok((evaluate(operand, variables, functions, output)? != "true").to_string()),
        Expression::Unary {
            operator: crate::parser::UnaryOperator::Negate,
            operand,
        } => {
            let value = evaluate(operand, variables, functions, output)?;
            if let Ok(value) = value.parse::<i64>() {
                value
                    .checked_neg()
                    .map(|value| value.to_string())
                    .ok_or_else(|| RuntimeError {
                        message: "integer negation overflow".into(),
                    })
            } else {
                Ok(render_float(-value.parse::<f64>().map_err(|_| {
                    RuntimeError {
                        message: "unary '-' requires a number".into(),
                    }
                })?))
            }
        }
        Expression::Index { target, index } => {
            let target_value = evaluate(target, variables, functions, output)?;
            let index_value = evaluate(index, variables, functions, output)?;
            let index = index_value.parse::<i64>().map_err(|_| RuntimeError {
                message: "array index requires Int".into(),
            })?;
            let target = parse_array_literal(&target_value)?;
            if index < 0 || index >= target.len() as i64 {
                return Err(RuntimeError {
                    message: "index out of bounds".into(),
                });
            }
            Ok(target[index as usize].clone())
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

fn integer(
    expression: &Expression,
    variables: &std::collections::HashMap<String, String>,
    functions: &[crate::parser::Function],
    output: &mut String,
) -> Result<i64, RuntimeError> {
    evaluate(expression, variables, functions, output)?
        .parse::<i64>()
        .map_err(|_| RuntimeError {
            message: "arithmetic operators expect integers".into(),
        })
}

fn numeric_binary<F>(
    left: &Expression,
    right: &Expression,
    variables: &std::collections::HashMap<String, String>,
    functions: &[crate::parser::Function],
    output: &mut String,
    operation: F,
) -> Result<String, RuntimeError>
where
    F: FnOnce(f64, f64) -> f64,
{
    let left = evaluate(left, variables, functions, output)?;
    let right = evaluate(right, variables, functions, output)?;
    let integer_operands = left.parse::<i64>().ok().zip(right.parse::<i64>().ok());
    if let Some((left, right)) = integer_operands {
        return Ok((operation(left as f64, right as f64) as i64).to_string());
    }
    let left = left.parse::<f64>().map_err(|_| RuntimeError {
        message: "arithmetic operators expect numbers".into(),
    })?;
    let right = right.parse::<f64>().map_err(|_| RuntimeError {
        message: "arithmetic operators expect numbers".into(),
    })?;
    Ok(render_float(operation(left, right)))
}

fn render_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

fn parse_array_literal(value: &str) -> Result<Vec<String>, RuntimeError> {
    let trimmed = value.trim();
    if trimmed == "[]" {
        return Ok(Vec::new());
    }
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(RuntimeError {
            message: "expected array literal".into(),
        });
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut elements = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;
    for (index, character) in inner.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            '[' => depth += 1,
            ']' if depth > 0 => depth -= 1,
            ']' => {
                return Err(RuntimeError {
                    message: "expected array literal".into(),
                });
            }
            ',' if depth == 0 => {
                elements.push(inner[start..index].trim().to_string());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    if depth != 0 || in_string {
        return Err(RuntimeError {
            message: "expected array literal".into(),
        });
    }
    elements.push(inner[start..].trim().to_string());
    Ok(elements)
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
            run("fn add(a, b) -> Int { return a + b } fn main() { print(add(20, 22)) }").unwrap(),
            "42\n"
        );
    }

    #[test]
    fn runs_string_concatenation() {
        assert_eq!(
            run("fn main() { print(\"Hello\" + \" \" + \"AXIOM\") }").unwrap(),
            "Hello AXIOM\n"
        );
    }

    #[test]
    fn runs_float_arithmetic() {
        assert_eq!(
            run("fn main() { let value: Float = 1.5 + 2.5; print(value); print(value / 2.0) }")
                .unwrap(),
            "4.0\n2.0\n"
        );
    }

    #[test]
    fn runs_if_else_with_comparison() {
        assert_eq!(
            run("fn main() { if 2 > 1 { print(\"yes\") } else { print(\"no\") } }").unwrap(),
            "yes\n"
        );
    }

    #[test]
    fn runs_while_with_assignment() {
        assert_eq!(
            run(
                "fn main() { let count = 0; while count == 0 { print(count); count = count + 1 } }"
            )
            .unwrap(),
            "0\n"
        );
    }

    #[test]
    fn runs_for_with_break_and_continue() {
        assert_eq!(
            run("fn main() { for (let i: Int = 0; i < 5; i = i + 1) { if i == 2 { continue } if i == 4 { break } print(i) } }").unwrap(),
            "0\n1\n3\n"
        );
    }

    #[test]
    fn runs_else_if_chains() {
        assert_eq!(
            run(
                "fn main() { let value = 2; if value == 1 { print(\"one\") } else if value == 2 { print(\"two\") } else { print(\"other\") } }"
            )
            .unwrap(),
            "two\n"
        );
    }

    #[test]
    fn runs_array_literal_and_indexing() {
        assert_eq!(
            run("fn main() { let values = [10, 20, 30]; print(values[1]) }").unwrap(),
            "20\n"
        );
    }

    #[test]
    fn runs_indexed_assignment() {
        assert_eq!(
            run("fn main() { let values = [10, 20, 30]; values[1] = 99; print(values[1]) }")
                .unwrap(),
            "99\n"
        );
    }

    #[test]
    fn runs_nested_indexed_assignment() {
        assert_eq!(
            run("fn main() { let matrix = [[10, 20], [30, 40]]; matrix[1][0] = 99; print(matrix[1][0]) }")
                .unwrap(),
            "99\n"
        );
    }

    #[test]
    fn runs_chained_array_indexing() {
        assert_eq!(
            run("fn main() { print([[10, 20]][0][1]) }").unwrap(),
            "20\n"
        );
    }

    #[test]
    fn runs_arithmetic_operators() {
        assert_eq!(run("fn main() { print(2 + 3 * 4 - 2) }").unwrap(), "12\n");
    }

    #[test]
    fn rejects_division_by_zero() {
        assert_eq!(
            run("fn main() { print(4 / 0) }").unwrap_err().message,
            "division by zero"
        );
    }

    #[test]
    fn runs_boolean_operators() {
        assert_eq!(
            run("fn main() { print(!false && true || false) }").unwrap(),
            "true\n"
        );
    }

    #[test]
    fn runs_typed_function_syntax() {
        assert_eq!(
            run("fn add(a: Int, b: Int) -> Int { return a + b } fn main() { let total: Int = add(20, 22); print(total) }").unwrap(),
            "42\n"
        );
    }
}
