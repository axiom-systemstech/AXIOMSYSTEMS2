use crate::parser::{
    BinaryOperator, Expression, Program, Statement, Type, UnaryOperator,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    PushInt(i64),
    PushBool(bool),
    PushString(String),
    LoadVariable(String),
    StoreVariable(String),
    Binary { op: BinaryOperator },
    Unary { op: UnaryOperator },
    Call { name: String, argument_count: usize },
    Print,
    Return,
    If {
        then_body: Vec<Instruction>,
        else_body: Vec<Instruction>,
    },
    While {
        body: Vec<Instruction>,
    },
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
                parameters: function.parameters.iter().map(|parameter| parameter.name.clone()).collect(),
                return_type: function.return_type,
                instructions: lower_block(&function.body),
            })
            .collect(),
    }
}

fn lower_block(statements: &[Statement]) -> Vec<Instruction> {
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
                    then_body: lower_block(then_body),
                    else_body: lower_block(else_body),
                });
                body
            }
            Statement::Assign { name, value } => {
                let mut body = lower_expression(value);
                body.push(Instruction::StoreVariable(name.clone()));
                body
            }
            Statement::While { condition, body } => {
                let mut instructions = lower_expression(condition);
                instructions.push(Instruction::While {
                    body: lower_block(body),
                });
                instructions
            }
        })
    }
    instructions
}

fn lower_expression(expression: &Expression) -> Vec<Instruction> {
    match expression {
        Expression::String(value) => vec![Instruction::PushString(value.clone())],
        Expression::Integer(value) => vec![Instruction::PushInt(*value)],
        Expression::Boolean(value) => vec![Instruction::PushBool(*value)],
        Expression::Variable(name) => vec![Instruction::LoadVariable(name.clone())],
        Expression::Binary { left, operator, right } => {
            let mut instructions = lower_expression(left);
            instructions.extend(lower_expression(right));
            instructions.push(Instruction::Binary { op: *operator });
            instructions
        }
        Expression::Unary { operator, operand } => {
            let mut instructions = lower_expression(operand);
            instructions.push(Instruction::Unary { op: *operator });
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
