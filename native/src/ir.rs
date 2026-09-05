use crate::parser::{BinaryOperator, Expression, Program, Statement, Type, UnaryOperator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    PushInt(i64),
    PushFloat(String),
    PushBool(bool),
    PushString(String),
    LoadVariable(String),
    StoreVariable(String),
    MakeArray {
        length: usize,
    },
    Binary {
        op: BinaryOperator,
    },
    ShortCircuitAnd {
        right: Vec<Instruction>,
    },
    ShortCircuitOr {
        right: Vec<Instruction>,
    },
    Unary {
        op: UnaryOperator,
    },
    Call {
        name: String,
        argument_count: usize,
    },
    Index,
    StoreIndex,
    Print,
    Return,
    If {
        then_body: Vec<Instruction>,
        else_body: Vec<Instruction>,
    },
    While {
        condition: Vec<Instruction>,
        body: Vec<Instruction>,
    },
    For {
        initializer: Vec<Instruction>,
        condition: Vec<Instruction>,
        update: Vec<Instruction>,
        body: Vec<Instruction>,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: Option<Type>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredProgram {
    pub functions: Vec<LoweredFunction>,
}

pub fn lower_program(program: &Program) -> LoweredProgram {
    LoweredProgram {
        functions: program
            .functions
            .iter()
            .map(|function| LoweredFunction {
                name: function.name.clone(),
                parameters: function
                    .parameters
                    .iter()
                    .map(|parameter| parameter.name.clone())
                    .collect(),
                return_type: function.return_type.clone(),
                instructions: lower_block(&function.body),
            })
            .collect(),
    }
}

fn lower_block(statements: &[Statement]) -> Vec<Instruction> {
    lower_block_with_counter(statements, &mut 0)
}

fn lower_block_with_counter(statements: &[Statement], counter: &mut usize) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    for statement in statements {
        instructions.extend(match statement {
            Statement::Let { name, value, .. } => {
                let mut body = lower_expression(value);
                body.push(Instruction::StoreVariable(name.clone()));
                body
            }
            Statement::Call(call) if call.name == "print" => {
                let mut body = lower_expression(&call.arguments[0]);
                body.push(Instruction::Print);
                body
            }
            Statement::Call(call) => {
                let mut body = Vec::new();
                for argument in &call.arguments {
                    body.extend(lower_expression(argument));
                }
                body.push(Instruction::Call {
                    name: call.name.clone(),
                    argument_count: call.arguments.len(),
                });
                body
            }
            Statement::Return(value) => {
                let mut body = lower_expression(value);
                body.push(Instruction::Return);
                body
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                let mut body = lower_expression(condition);
                body.push(Instruction::If {
                    then_body: lower_block_with_counter(then_body, counter),
                    else_body: lower_block_with_counter(else_body, counter),
                });
                body
            }
            Statement::Assign { target, value } => lower_assignment(target, value, counter),
            Statement::While { condition, body } => {
                let mut instructions = Vec::new();
                instructions.push(Instruction::While {
                    condition: lower_expression(condition),
                    body: lower_block_with_counter(body, counter),
                });
                instructions
            }
            Statement::For {
                initializer,
                condition,
                update,
                body,
            } => vec![Instruction::For {
                initializer: initializer.as_deref().map_or_else(Vec::new, |statement| {
                    lower_block_with_counter(std::slice::from_ref(statement), counter)
                }),
                condition: lower_expression(condition),
                update: update.as_deref().map_or_else(Vec::new, |statement| {
                    lower_block_with_counter(std::slice::from_ref(statement), counter)
                }),
                body: lower_block_with_counter(body, counter),
            }],
            Statement::Break => vec![Instruction::Break],
            Statement::Continue => vec![Instruction::Continue],
        })
    }
    instructions
}

fn lower_assignment(
    target: &Expression,
    value: &Expression,
    counter: &mut usize,
) -> Vec<Instruction> {
    match target {
        Expression::Variable(name) => {
            let mut body = lower_expression(value);
            body.push(Instruction::StoreVariable(name.clone()));
            body
        }
        Expression::Index { .. } => {
            let mut indexes = Vec::new();
            let mut current = target;
            while let Expression::Index {
                target: nested_target,
                index,
            } = current
            {
                indexes.push(index.as_ref().clone());
                current = nested_target;
            }
            let Expression::Variable(base_name) = current else {
                let mut body = lower_expression(value);
                body.push(Instruction::StoreVariable("_tmp".to_string()));
                return body;
            };
            indexes.reverse();

            let mut body = Vec::new();
            let mut current_var = base_name.clone();
            let mut path_updates = Vec::new();

            for index in &indexes[..indexes.len().saturating_sub(1)] {
                body.push(Instruction::LoadVariable(current_var.clone()));
                body.extend(lower_expression(index));
                body.push(Instruction::Index);
                let temp_name = format!("__axiom_tmp_{}", counter);
                *counter += 1;
                body.push(Instruction::StoreVariable(temp_name.clone()));
                path_updates.push((current_var.clone(), index.clone()));
                current_var = temp_name;
            }

            let final_index = indexes
                .last()
                .cloned()
                .unwrap_or_else(|| Expression::Integer(0));
            body.push(Instruction::LoadVariable(current_var.clone()));
            body.extend(lower_expression(&final_index));
            body.extend(lower_expression(value));
            body.push(Instruction::StoreIndex);
            body.push(Instruction::StoreVariable(current_var.clone()));

            for (parent_var, index) in path_updates.into_iter().rev() {
                body.push(Instruction::LoadVariable(parent_var.clone()));
                body.extend(lower_expression(&index));
                body.push(Instruction::LoadVariable(current_var.clone()));
                body.push(Instruction::StoreIndex);
                body.push(Instruction::StoreVariable(parent_var.clone()));
                current_var = parent_var;
            }

            body
        }
        _ => {
            let mut body = lower_expression(value);
            body.push(Instruction::StoreVariable("_tmp".to_string()));
            body
        }
    }
}

fn lower_expression(expression: &Expression) -> Vec<Instruction> {
    match expression {
        Expression::String(value) => vec![Instruction::PushString(value.clone())],
        Expression::Integer(value) => vec![Instruction::PushInt(*value)],
        Expression::Float(value) => vec![Instruction::PushFloat(value.clone())],
        Expression::Boolean(value) => vec![Instruction::PushBool(*value)],
        Expression::Array(values) => {
            let mut instructions = Vec::new();
            for value in values {
                instructions.extend(lower_expression(value));
            }
            instructions.push(Instruction::MakeArray {
                length: values.len(),
            });
            instructions
        }
        Expression::Variable(name) => vec![Instruction::LoadVariable(name.clone())],
        Expression::Binary {
            left,
            operator,
            right,
        } => {
            let mut instructions = lower_expression(left);
            match operator {
                BinaryOperator::And => instructions.push(Instruction::ShortCircuitAnd {
                    right: lower_expression(right),
                }),
                BinaryOperator::Or => instructions.push(Instruction::ShortCircuitOr {
                    right: lower_expression(right),
                }),
                _ => {
                    instructions.extend(lower_expression(right));
                    instructions.push(Instruction::Binary { op: *operator });
                }
            }
            instructions
        }
        Expression::Unary { operator, operand } => {
            let mut instructions = lower_expression(operand);
            instructions.push(Instruction::Unary { op: *operator });
            instructions
        }
        Expression::Index { target, index } => {
            let mut instructions = lower_expression(target);
            instructions.extend(lower_expression(index));
            instructions.push(Instruction::Index);
            instructions
        }
        Expression::Call(call) => {
            let mut instructions = Vec::new();
            for argument in &call.arguments {
                instructions.extend(lower_expression(argument));
            }
            instructions.push(Instruction::Call {
                name: call.name.clone(),
                argument_count: call.arguments.len(),
            });
            instructions
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn lowers_main_function() {
        let program = parse("fn main() { print(42) }").unwrap();
        let lowered = lower_program(&program);
        assert_eq!(lowered.functions[0].name, "main");
        assert!(lowered.functions[0]
            .instructions
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Print)));
    }
}
