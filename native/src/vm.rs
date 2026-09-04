use std::collections::HashMap;

use crate::ir::{lower_program, Instruction, LoweredFunction};
use crate::parser::Program;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
}

impl Value {
    fn as_int(&self) -> Result<i64, VmError> {
        match self {
            Value::Int(value) => Ok(*value),
            _ => Err(VmError {
                message: "expected Int value".into(),
            }),
        }
    }

    fn as_bool(&self) -> Result<bool, VmError> {
        match self {
            Value::Bool(value) => Ok(*value),
            _ => Err(VmError {
                message: "expected Bool value".into(),
            }),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(value) => write!(formatter, "{value}"),
            Value::Bool(value) => write!(formatter, "{value}"),
            Value::String(value) => write!(formatter, "{value}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmError {
    pub message: String,
}

pub fn execute_program(program: &Program) -> Result<String, VmError> {
    let lowered = lower_program(program);
    let mut machine = Machine::new(lowered.functions);
    machine.call_function("main", Vec::new())?;
    Ok(machine.output)
}

pub fn build_artifact(program: &Program) -> String {
    let lowered = lower_program(program);
    let mut output = String::new();
    for function in &lowered.functions {
        output.push_str(&format!("fn {}\n", function.name));
        for instruction in &function.instructions {
            output.push_str(&format!("  {:?}\n", instruction));
        }
    }
    output
}

struct Machine {
    functions: Vec<LoweredFunction>,
    output: String,
    stack: Vec<Value>,
}

impl Machine {
    fn new(functions: Vec<LoweredFunction>) -> Self {
        Self {
            functions,
            output: String::new(),
            stack: Vec::new(),
        }
    }

    fn call_function(&mut self, name: &str, arguments: Vec<Value>) -> Result<Option<Value>, VmError> {
        let function = self
            .functions
            .iter()
            .find(|function| function.name == name)
            .cloned()
            .ok_or_else(|| VmError {
                message: format!("unknown function '{name}'"),
            })?;

        let mut locals = HashMap::new();
        for (parameter, value) in function.parameters.iter().zip(arguments) {
            locals.insert(parameter.clone(), value);
        }

        self.execute_block(&function.instructions, &mut locals)
    }

    fn execute_block(
        &mut self,
        instructions: &[Instruction],
        locals: &mut HashMap<String, Value>,
    ) -> Result<Option<Value>, VmError> {
        let mut index = 0;
        while index < instructions.len() {
            match &instructions[index] {
                Instruction::PushInt(value) => self.stack.push(Value::Int(*value)),
                Instruction::PushBool(value) => self.stack.push(Value::Bool(*value)),
                Instruction::PushString(value) => self.stack.push(Value::String(value.clone())),
                Instruction::LoadVariable(name) => {
                    let value = locals
                        .get(name)
                        .cloned()
                        .ok_or_else(|| VmError {
                            message: format!("unknown variable '{name}'"),
                        })?;
                    self.stack.push(value);
                }
                Instruction::StoreVariable(name) => {
                    let value = self.pop_stack()?;
                    locals.insert(name.clone(), value);
                }
                Instruction::Binary { op } => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let value = match op {
                        crate::parser::BinaryOperator::Add => Value::Int(left.as_int()? + right.as_int()?),
                        crate::parser::BinaryOperator::Subtract => Value::Int(left.as_int()? - right.as_int()?),
                        crate::parser::BinaryOperator::Multiply => Value::Int(left.as_int()? * right.as_int()?),
                        crate::parser::BinaryOperator::Divide => {
                            let divisor = right.as_int()?;
                            if divisor == 0 {
                                return Err(VmError {
                                    message: "division by zero".into(),
                                });
                            }
                            Value::Int(left.as_int()? / divisor)
                        }
                        crate::parser::BinaryOperator::Greater => Value::Bool(left.as_int()? > right.as_int()?),
                        crate::parser::BinaryOperator::Equal => Value::Bool(left == right),
                        crate::parser::BinaryOperator::And => Value::Bool(left.as_bool()? && right.as_bool()?),
                        crate::parser::BinaryOperator::Or => Value::Bool(left.as_bool()? || right.as_bool()?),
                    };
                    self.stack.push(value);
                }
                Instruction::Unary { op } => {
                    let value = self.pop_stack()?;
                    let result = match op {
                        crate::parser::UnaryOperator::Not => Value::Bool(!value.as_bool()?),
                    };
                    self.stack.push(result);
                }
                Instruction::Call { name, argument_count } => {
                    let mut arguments = Vec::with_capacity(*argument_count);
                    for _ in 0..*argument_count {
                        arguments.push(self.pop_stack()?);
                    }
                    arguments.reverse();
                    match self.call_function(name, arguments)? {
                        Some(value) => self.stack.push(value),
                        None => {}
                    }
                }
                Instruction::Print => {
                    let value = self.pop_stack()?;
                    self.output.push_str(&value.to_string());
                    self.output.push('\n');
                }
                Instruction::Return => {
                    let value = self.pop_stack()?;
                    return Ok(Some(value));
                }
                Instruction::If {
                    then_body,
                    else_body,
                } => {
                    let condition = self.pop_stack()?.as_bool()?;
                    let block = if condition { then_body } else { else_body };
                    if let Some(value) = self.execute_block(block, locals)? {
                        return Ok(Some(value));
                    }
                }
                Instruction::While { condition, body } => {
                    loop {
                        self.execute_block(condition, locals)?;
                        let condition_value = self.pop_stack()?.as_bool()?;
                        if !condition_value {
                            break;
                        }
                        if let Some(value) = self.execute_block(body, locals)? {
                            return Ok(Some(value));
                        }
                    }
                }
            }
            index += 1;
        }
        Ok(None)
    }

    fn pop_stack(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or_else(|| VmError {
            message: "stack underflow".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn executes_print_and_arithmetic() {
        let program = parse("fn main() { let total: Int = 20 + 22; print(total) }").unwrap();
        let output = execute_program(&program).unwrap();
        assert_eq!(output, "42\n");
    }

    #[test]
    fn builds_ir_artifact() {
        let program = parse("fn main() { print(42) }").unwrap();
        let artifact = build_artifact(&program);
        assert!(artifact.contains("fn main"));
    }
}
