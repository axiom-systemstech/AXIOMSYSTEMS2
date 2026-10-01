use std::collections::HashMap;

use crate::ir::{lower_program, Instruction, LoweredFunction};
use crate::parser::{BinaryOperator, Program, UnaryOperator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Float(String),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
    Struct(HashMap<String, Value>),
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

    fn as_array(&self) -> Result<&[Value], VmError> {
        match self {
            Value::Array(values) => Ok(values),
            _ => Err(VmError {
                message: "expected Array value".into(),
            }),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(value) => write!(formatter, "{value}"),
            Value::Float(value) => write!(formatter, "{value}"),
            Value::Bool(value) => write!(formatter, "{value}"),
            Value::String(value) => write!(formatter, "{value}"),
            Value::Array(values) => {
                let rendered = values
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(formatter, "[{rendered}]")
            }
            Value::Struct(fields) => {
                let rendered = fields
                    .iter()
                    .map(|(name, value)| format!("{name}={value}"))
                    .collect::<Vec<_>>()
                    .join(",");
                write!(formatter, "{{{rendered}}}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub functions: Vec<CompiledFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub instructions: Vec<Instruction>,
}

impl Artifact {
    pub fn serialize(&self) -> String {
        let mut lines = vec!["AXIOM_ARTIFACT_V1".to_string()];
        for function in &self.functions {
            lines.push(format!("FUNCTION:{}", escape_string(&function.name)));
            lines.push(format!("PARAMS:{}", function.parameters.len()));
            for parameter in &function.parameters {
                lines.push(format!("PARAM:{}", escape_string(parameter)));
            }
            lines.push(format!("INSTR:{}", encode_sequence(&function.instructions)));
        }
        lines.join("\n")
    }

    pub fn deserialize(input: &str) -> Result<Self, VmError> {
        let mut lines = input.lines();
        let header = lines.next().ok_or_else(|| VmError {
            message: "missing artifact header".into(),
        })?;
        if header != "AXIOM_ARTIFACT_V1" {
            return Err(VmError {
                message: "unsupported artifact format".into(),
            });
        }

        let mut functions = Vec::new();
        while let Some(line) = lines.next() {
            if !line.starts_with("FUNCTION:") {
                continue;
            }
            let name = unescape_string(line.strip_prefix("FUNCTION:").unwrap());
            let params_line = lines.next().ok_or_else(|| VmError {
                message: format!("missing parameter count for '{name}'"),
            })?;
            let params_count = params_line
                .strip_prefix("PARAMS:")
                .ok_or_else(|| VmError {
                    message: format!("invalid parameter count for '{name}'"),
                })?
                .parse::<usize>()
                .map_err(|_| VmError {
                    message: format!("invalid parameter count for '{name}'"),
                })?;

            let mut parameters = Vec::with_capacity(params_count);
            for _ in 0..params_count {
                let param_line = lines.next().ok_or_else(|| VmError {
                    message: format!("missing parameter for '{name}'"),
                })?;
                let value = param_line.strip_prefix("PARAM:").ok_or_else(|| VmError {
                    message: format!("invalid parameter entry for '{name}'"),
                })?;
                parameters.push(unescape_string(value));
            }

            let instructions_line = lines.next().ok_or_else(|| VmError {
                message: format!("missing instructions for '{name}'"),
            })?;
            let encoded = instructions_line
                .strip_prefix("INSTR:")
                .ok_or_else(|| VmError {
                    message: format!("invalid instruction entry for '{name}'"),
                })?;
            let instructions = decode_sequence(encoded)?;
            functions.push(CompiledFunction {
                name,
                parameters,
                instructions,
            });
        }

        Ok(Self { functions })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmError {
    pub message: String,
}

enum Flow {
    None,
    Return(Value),
    Break,
    Continue,
}

pub fn compile_program(program: &Program) -> Artifact {
    let lowered = lower_program(program);
    Artifact {
        functions: lowered
            .functions
            .into_iter()
            .map(|function| CompiledFunction {
                name: function.name,
                parameters: function.parameters,
                instructions: function.instructions,
            })
            .collect(),
    }
}

pub fn execute_program(program: &Program) -> Result<String, VmError> {
    let lowered = lower_program(program);
    let mut machine = Machine::new(lowered.functions);
    machine.call_function("main", Vec::new())?;
    Ok(machine.output)
}

pub fn execute_artifact(artifact: &Artifact) -> Result<String, VmError> {
    let functions = artifact
        .functions
        .iter()
        .cloned()
        .map(|function| LoweredFunction {
            name: function.name,
            parameters: function.parameters,
            return_type: None,
            instructions: function.instructions,
        })
        .collect();
    let mut machine = Machine::new(functions);
    machine.call_function("main", Vec::new())?;
    Ok(machine.output)
}

pub fn build_artifact(program: &Program) -> String {
    compile_program(program).serialize()
}

pub fn write_artifact_file(
    path: &std::path::Path,
    program: &Program,
) -> Result<std::path::PathBuf, VmError> {
    let out_path = path.with_extension("axm");
    write_artifact_file_with_target(path, Some(&out_path), program)
}

pub fn write_artifact_file_with_target(
    source_path: &std::path::Path,
    target_path: Option<&std::path::Path>,
    program: &Program,
) -> Result<std::path::PathBuf, VmError> {
    let artifact = compile_program(program);
    let serialized = artifact.serialize();
    let out_path = target_path
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| source_path.with_extension("axm"));
    let parent = out_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    if !parent.exists() {
        std::fs::create_dir_all(parent).map_err(|error| VmError {
            message: format!(
                "cannot create artifact directory '{}': {error}",
                parent.display()
            ),
        })?;
    }
    std::fs::write(&out_path, &serialized).map_err(|error| VmError {
        message: format!("cannot write artifact '{}': {error}", out_path.display()),
    })?;
    Ok(out_path)
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

    fn call_function(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
    ) -> Result<Option<Value>, VmError> {
        if let Some(value) = call_standard_library(name, &arguments)? {
            return Ok(Some(value));
        }
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

        match self.execute_block(&function.instructions, &mut locals)? {
            Flow::None => Ok(None),
            Flow::Return(value) => Ok(Some(value)),
            Flow::Break | Flow::Continue => Err(VmError {
                message: "loop control escaped its loop".into(),
            }),
        }
    }

    fn execute_block(
        &mut self,
        instructions: &[Instruction],
        locals: &mut HashMap<String, Value>,
    ) -> Result<Flow, VmError> {
        let mut index = 0;
        while index < instructions.len() {
            match &instructions[index] {
                Instruction::PushInt(value) => self.stack.push(Value::Int(*value)),
                Instruction::PushFloat(value) => self.stack.push(Value::Float(value.clone())),
                Instruction::PushBool(value) => self.stack.push(Value::Bool(*value)),
                Instruction::PushString(value) => self.stack.push(Value::String(value.clone())),
                Instruction::LoadVariable(name) => {
                    let value = locals.get(name).cloned().ok_or_else(|| VmError {
                        message: format!("unknown variable '{name}'"),
                    })?;
                    self.stack.push(value);
                }
                Instruction::StoreVariable(name) => {
                    let value = self.pop_stack()?;
                    locals.insert(name.clone(), value);
                }
                Instruction::MakeArray { length } => {
                    let mut values = Vec::with_capacity(*length);
                    for _ in 0..*length {
                        values.push(self.pop_stack()?);
                    }
                    values.reverse();
                    self.stack.push(Value::Array(values));
                }
                Instruction::MakeStruct { fields } => {
                    let mut values = Vec::with_capacity(fields.len());
                    for _ in fields {
                        values.push(self.pop_stack()?);
                    }
                    values.reverse();
                    let fields = fields.iter().cloned().zip(values).collect();
                    self.stack.push(Value::Struct(fields));
                }
                Instruction::Binary { op } => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let value = match op {
                        BinaryOperator::Add => match (&left, &right) {
                            (Value::Int(left), Value::Int(right)) => Value::Int(left + right),
                            (Value::Float(left), Value::Float(right)) => {
                                Value::Float(render_float(
                                    left.parse::<f64>().map_err(|_| VmError {
                                        message: "invalid Float value".into(),
                                    })? + right.parse::<f64>().map_err(|_| VmError {
                                        message: "invalid Float value".into(),
                                    })?,
                                ))
                            }
                            (Value::String(left), Value::String(right)) => {
                                Value::String(format!("{left}{right}"))
                            }
                            _ => {
                                return Err(VmError {
                                    message: "addition requires matching operands".into(),
                                })
                            }
                        },
                        BinaryOperator::Subtract => match (&left, &right) {
                            (Value::Int(left), Value::Int(right)) => Value::Int(left - right),
                            (Value::Float(left), Value::Float(right)) => {
                                Value::Float(render_float(
                                    left.parse::<f64>().map_err(|_| VmError {
                                        message: "invalid Float value".into(),
                                    })? - right.parse::<f64>().map_err(|_| VmError {
                                        message: "invalid Float value".into(),
                                    })?,
                                ))
                            }
                            _ => {
                                return Err(VmError {
                                    message: "arithmetic requires matching operands".into(),
                                })
                            }
                        },
                        BinaryOperator::Multiply => match (&left, &right) {
                            (Value::Int(left), Value::Int(right)) => Value::Int(left * right),
                            (Value::Float(left), Value::Float(right)) => {
                                Value::Float(render_float(
                                    left.parse::<f64>().map_err(|_| VmError {
                                        message: "invalid Float value".into(),
                                    })? * right.parse::<f64>().map_err(|_| VmError {
                                        message: "invalid Float value".into(),
                                    })?,
                                ))
                            }
                            _ => {
                                return Err(VmError {
                                    message: "arithmetic requires matching operands".into(),
                                })
                            }
                        },
                        BinaryOperator::Divide => {
                            let divisor = match &right {
                                Value::Int(value) => *value as f64,
                                Value::Float(value) => {
                                    value.parse::<f64>().map_err(|_| VmError {
                                        message: "invalid Float value".into(),
                                    })?
                                }
                                _ => {
                                    return Err(VmError {
                                        message: "arithmetic requires numeric operands".into(),
                                    })
                                }
                            };
                            if divisor == 0.0 {
                                return Err(VmError {
                                    message: "division by zero".into(),
                                });
                            }
                            match &left {
                                Value::Int(value) if matches!(&right, Value::Int(_)) => {
                                    Value::Int(*value / divisor as i64)
                                }
                                Value::Float(value) => Value::Float(render_float(
                                    value.parse::<f64>().map_err(|_| VmError {
                                        message: "invalid Float value".into(),
                                    })? / divisor,
                                )),
                                _ => {
                                    return Err(VmError {
                                        message: "arithmetic requires numeric operands".into(),
                                    })
                                }
                            }
                        }
                        BinaryOperator::Modulo => {
                            let left = left.as_int()?;
                            let right = right.as_int()?;
                            if right == 0 {
                                return Err(VmError {
                                    message: "division by zero".into(),
                                });
                            }
                            Value::Int(left % right)
                        }
                        BinaryOperator::Greater => {
                            Value::Bool(compare_numeric(&left, &right, |left, right| left > right)?)
                        }
                        BinaryOperator::GreaterEqual => {
                            Value::Bool(compare_numeric(&left, &right, |left, right| {
                                left >= right
                            })?)
                        }
                        BinaryOperator::Less => {
                            Value::Bool(compare_numeric(&left, &right, |left, right| left < right)?)
                        }
                        BinaryOperator::LessEqual => {
                            Value::Bool(compare_numeric(&left, &right, |left, right| {
                                left <= right
                            })?)
                        }
                        BinaryOperator::Equal => Value::Bool(left == right),
                        BinaryOperator::NotEqual => Value::Bool(left != right),
                        BinaryOperator::And => Value::Bool(left.as_bool()? && right.as_bool()?),
                        BinaryOperator::Or => Value::Bool(left.as_bool()? || right.as_bool()?),
                    };
                    self.stack.push(value);
                }
                Instruction::ShortCircuitAnd { right } => {
                    let left = self.pop_stack()?.as_bool()?;
                    if !left {
                        self.stack.push(Value::Bool(false));
                    } else {
                        match self.execute_block(right, locals)? {
                            Flow::None => {
                                let value = self.pop_stack()?.as_bool()?;
                                self.stack.push(Value::Bool(value));
                            }
                            flow => return Ok(flow),
                        }
                    }
                }
                Instruction::ShortCircuitOr { right } => {
                    let left = self.pop_stack()?.as_bool()?;
                    if left {
                        self.stack.push(Value::Bool(true));
                    } else {
                        match self.execute_block(right, locals)? {
                            Flow::None => {
                                let value = self.pop_stack()?.as_bool()?;
                                self.stack.push(Value::Bool(value));
                            }
                            flow => return Ok(flow),
                        }
                    }
                }
                Instruction::Unary { op } => {
                    let value = self.pop_stack()?;
                    let result =
                        match op {
                            UnaryOperator::Not => Value::Bool(!value.as_bool()?),
                            UnaryOperator::Negate => match value {
                                Value::Int(value) => {
                                    Value::Int(value.checked_neg().ok_or_else(|| VmError {
                                        message: "integer negation overflow".into(),
                                    })?)
                                }
                                Value::Float(value) => Value::Float(render_float(
                                    -value.parse::<f64>().map_err(|_| VmError {
                                        message: "invalid Float value".into(),
                                    })?,
                                )),
                                _ => {
                                    return Err(VmError {
                                        message: "unary '-' requires a numeric operand".into(),
                                    })
                                }
                            },
                        };
                    self.stack.push(result);
                }
                Instruction::Index => {
                    let index = self.pop_stack()?.as_int()?;
                    let array = self.pop_stack()?;
                    let values = array.as_array()?;
                    if index < 0 || index >= values.len() as i64 {
                        return Err(VmError {
                            message: "index out of bounds".into(),
                        });
                    }
                    self.stack.push(values[index as usize].clone());
                }
                Instruction::GetField(field) => {
                    let value = self.pop_stack()?;
                    let Value::Struct(fields) = value else {
                        return Err(VmError {
                            message: "expected struct value".into(),
                        });
                    };
                    self.stack
                        .push(fields.get(field).cloned().ok_or_else(|| VmError {
                            message: format!("unknown field '{field}'"),
                        })?);
                }
                Instruction::StoreField(field) => {
                    let value = self.pop_stack()?;
                    let structure = self.pop_stack()?;
                    let Value::Struct(mut fields) = structure else {
                        return Err(VmError {
                            message: "expected struct value".into(),
                        });
                    };
                    if !fields.contains_key(field) {
                        return Err(VmError {
                            message: format!("unknown field '{field}'"),
                        });
                    }
                    fields.insert(field.clone(), value);
                    self.stack.push(Value::Struct(fields));
                }
                Instruction::StoreIndex => {
                    let value = self.pop_stack()?;
                    let index = self.pop_stack()?.as_int()?;
                    let array = self.pop_stack()?;
                    let values = array.as_array()?.to_vec();
                    if index < 0 || index >= values.len() as i64 {
                        return Err(VmError {
                            message: "index out of bounds".into(),
                        });
                    }
                    let mut updated = values;
                    updated[index as usize] = value;
                    self.stack.push(Value::Array(updated));
                }
                Instruction::Call {
                    name,
                    argument_count,
                } => {
                    let mut arguments = Vec::with_capacity(*argument_count);
                    for _ in 0..*argument_count {
                        arguments.push(self.pop_stack()?);
                    }
                    arguments.reverse();
                    if let Some(value) = self.call_function(name, arguments)? {
                        self.stack.push(value);
                    }
                }
                Instruction::Print => {
                    let value = self.pop_stack()?;
                    self.output.push_str(&value.to_string());
                    self.output.push('\n');
                }
                Instruction::Return => {
                    let value = self.pop_stack()?;
                    return Ok(Flow::Return(value));
                }
                Instruction::If {
                    then_body,
                    else_body,
                } => {
                    let condition = self.pop_stack()?.as_bool()?;
                    let block = if condition { then_body } else { else_body };
                    match self.execute_block(block, locals)? {
                        Flow::None => {}
                        flow => return Ok(flow),
                    }
                }
                Instruction::While { condition, body } => loop {
                    self.execute_block(condition, locals)?;
                    let condition_value = self.pop_stack()?.as_bool()?;
                    if !condition_value {
                        break;
                    }
                    match self.execute_block(body, locals)? {
                        Flow::None | Flow::Continue => {}
                        Flow::Break => break,
                        flow => return Ok(flow),
                    }
                },
                Instruction::For {
                    initializer,
                    condition,
                    update,
                    body,
                } => {
                    match self.execute_block(initializer, locals)? {
                        Flow::None => {}
                        flow => return Ok(flow),
                    }
                    loop {
                        self.execute_block(condition, locals)?;
                        if !self.pop_stack()?.as_bool()? {
                            break;
                        }
                        match self.execute_block(body, locals)? {
                            Flow::None | Flow::Continue => {}
                            Flow::Break => break,
                            flow => return Ok(flow),
                        }
                        match self.execute_block(update, locals)? {
                            Flow::None => {}
                            flow => return Ok(flow),
                        }
                    }
                }
                Instruction::Break => return Ok(Flow::Break),
                Instruction::Continue => return Ok(Flow::Continue),
            }
            index += 1;
        }
        Ok(Flow::None)
    }

    fn pop_stack(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or_else(|| VmError {
            message: "stack underflow".into(),
        })
    }
}

fn call_standard_library(name: &str, arguments: &[Value]) -> Result<Option<Value>, VmError> {
    match name {
        "split_lines" => {
            if arguments.len() != 1 {
                return Err(VmError {
                    message: "split_lines expects one String argument".into(),
                });
            }
            let value = match &arguments[0] {
                Value::String(value) => value,
                _ => {
                    return Err(VmError {
                        message: "split_lines expects a String".into(),
                    })
                }
            };
            Ok(Some(Value::Array(
                value
                    .lines()
                    .map(|line| Value::String(line.to_owned()))
                    .collect(),
            )))
        }
        "split" => {
            if arguments.len() != 2 {
                return Err(VmError {
                    message: "split expects a String value and separator".into(),
                });
            }
            let value = match &arguments[0] {
                Value::String(value) => value,
                _ => {
                    return Err(VmError {
                        message: "split expects a String value".into(),
                    })
                }
            };
            let separator = match &arguments[1] {
                Value::String(value) => value,
                _ => {
                    return Err(VmError {
                        message: "split expects a String separator".into(),
                    })
                }
            };
            Ok(Some(Value::Array(
                value
                    .split(separator)
                    .map(|part| Value::String(part.to_owned()))
                    .collect(),
            )))
        }
        "read_file" => {
            if arguments.len() != 1 {
                return Err(VmError {
                    message: "read_file expects one argument".into(),
                });
            }
            let path = match &arguments[0] {
                Value::String(value) => value,
                _ => {
                    return Err(VmError {
                        message: "read_file expects a String path".into(),
                    })
                }
            };
            let content = std::fs::read_to_string(path).map_err(|error| VmError {
                message: format!("cannot read file '{path}': {error}"),
            })?;
            Ok(Some(Value::String(content)))
        }
        "write_file" => {
            if arguments.len() != 2 {
                return Err(VmError {
                    message: "write_file expects a path and content".into(),
                });
            }
            let path = match &arguments[0] {
                Value::String(value) => value,
                _ => {
                    return Err(VmError {
                        message: "write_file expects a String path".into(),
                    })
                }
            };
            let content = match &arguments[1] {
                Value::String(value) => value,
                _ => {
                    return Err(VmError {
                        message: "write_file expects String content".into(),
                    })
                }
            };
            std::fs::write(path, content).map_err(|error| VmError {
                message: format!("cannot write file '{path}': {error}"),
            })?;
            Ok(Some(Value::Bool(true)))
        }
        "char_at" => {
            if arguments.len() != 2 {
                return Err(VmError {
                    message: "char_at expects a String and an Int index".into(),
                });
            }
            let value = match &arguments[0] {
                Value::String(value) => value,
                _ => {
                    return Err(VmError {
                        message: "char_at expects a String".into(),
                    })
                }
            };
            let index = arguments[1].as_int()?;
            if index < 0 {
                return Err(VmError {
                    message: "char_at index must be non-negative".into(),
                });
            }
            let character = value.chars().nth(index as usize).ok_or_else(|| VmError {
                message: format!("char_at index {index} out of bounds"),
            })?;
            Ok(Some(Value::String(character.to_string())))
        }
        "char_code" => {
            if arguments.len() != 1 {
                return Err(VmError {
                    message: "char_code expects one String character".into(),
                });
            }
            let value = match &arguments[0] {
                Value::String(value) => value,
                _ => {
                    return Err(VmError {
                        message: "char_code expects a String".into(),
                    })
                }
            };
            let mut chars = value.chars();
            let character = chars.next().ok_or_else(|| VmError {
                message: "char_code expects a non-empty String".into(),
            })?;
            if chars.next().is_some() {
                return Err(VmError {
                    message: "char_code expects exactly one character".into(),
                });
            }
            Ok(Some(Value::Int(character as i64)))
        }
        "int_to_string" => {
            if arguments.len() != 1 {
                return Err(VmError {
                    message: "int_to_string expects one Int argument".into(),
                });
            }
            Ok(Some(Value::String(arguments[0].as_int()?.to_string())))
        }
        "len" => {
            if arguments.len() != 1 {
                return Err(VmError {
                    message: "len expects one argument".into(),
                });
            }
            let length = match &arguments[0] {
                Value::String(value) => value.chars().count(),
                Value::Array(values) => values.len(),
                _ => {
                    return Err(VmError {
                        message: "len expects a String or array".into(),
                    })
                }
            };
            Ok(Some(Value::Int(length as i64)))
        }
        "abs" => {
            if arguments.len() != 1 {
                return Err(VmError {
                    message: "abs expects one argument".into(),
                });
            }
            match &arguments[0] {
                Value::Int(value) => Ok(Some(Value::Int(value.abs()))),
                Value::Float(value) => {
                    let value = value.parse::<f64>().map_err(|_| VmError {
                        message: "abs expects a numeric value".into(),
                    })?;
                    Ok(Some(Value::Float(render_float(value.abs()))))
                }
                _ => Err(VmError {
                    message: "abs expects an Int or Float".into(),
                }),
            }
        }
        "min" | "max" => {
            if arguments.len() != 2 {
                return Err(VmError {
                    message: format!("{name} expects two arguments"),
                });
            }
            match (&arguments[0], &arguments[1]) {
                (Value::Int(left), Value::Int(right)) => {
                    let value = if name == "min" {
                        (*left).min(*right)
                    } else {
                        (*left).max(*right)
                    };
                    Ok(Some(Value::Int(value)))
                }
                (Value::Float(left), Value::Float(right)) => {
                    let left = left.parse::<f64>().map_err(|_| VmError {
                        message: "expected Float value".into(),
                    })?;
                    let right = right.parse::<f64>().map_err(|_| VmError {
                        message: "expected Float value".into(),
                    })?;
                    let value = if name == "min" {
                        left.min(right)
                    } else {
                        left.max(right)
                    };
                    Ok(Some(Value::Float(render_float(value))))
                }
                _ => Err(VmError {
                    message: format!("{name} expects two matching numeric arguments"),
                }),
            }
        }
        _ => Ok(None),
    }
}

fn encode_sequence(instructions: &[Instruction]) -> String {
    instructions
        .iter()
        .map(encode_instruction)
        .collect::<Vec<_>>()
        .join(";")
}

fn decode_sequence(raw: &str) -> Result<Vec<Instruction>, VmError> {
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    for part in split_escaped(raw, ';') {
        if !part.is_empty() {
            result.push(decode_instruction(&part)?);
        }
    }
    Ok(result)
}

fn encode_instruction(instruction: &Instruction) -> String {
    match instruction {
        Instruction::PushInt(value) => format!("PushInt:{value}"),
        Instruction::PushFloat(value) => format!("PushFloat:{value}"),
        Instruction::PushBool(value) => {
            format!("PushBool:{}", if *value { "true" } else { "false" })
        }
        Instruction::PushString(value) => format!("PushString:{}", escape_string(value)),
        Instruction::LoadVariable(name) => format!("LoadVariable:{}", escape_string(name)),
        Instruction::StoreVariable(name) => format!("StoreVariable:{}", escape_string(name)),
        Instruction::MakeArray { length } => format!("MakeArray:{length}"),
        Instruction::MakeStruct { fields } => format!(
            "MakeStruct:{}",
            fields
                .iter()
                .map(|field| escape_string(field))
                .collect::<Vec<_>>()
                .join("|")
        ),
        Instruction::Binary { op } => format!("Binary:{}", encode_binary(op)),
        Instruction::ShortCircuitAnd { right } => {
            format!("ShortCircuitAnd[{}]", encode_sequence(right))
        }
        Instruction::ShortCircuitOr { right } => {
            format!("ShortCircuitOr[{}]", encode_sequence(right))
        }
        Instruction::Unary { op } => format!("Unary:{}", encode_unary(op)),
        Instruction::Call {
            name,
            argument_count,
        } => {
            format!("Call:{}|{}", escape_string(name), argument_count)
        }
        Instruction::Index => "Index".to_string(),
        Instruction::StoreIndex => "StoreIndex".to_string(),
        Instruction::GetField(field) => format!("GetField:{}", escape_string(field)),
        Instruction::StoreField(field) => format!("StoreField:{}", escape_string(field)),
        Instruction::Print => "Print".to_string(),
        Instruction::Return => "Return".to_string(),
        Instruction::If {
            then_body,
            else_body,
        } => {
            format!(
                "If[{}][{}]",
                encode_sequence(then_body),
                encode_sequence(else_body)
            )
        }
        Instruction::While { condition, body } => {
            format!(
                "While[{}][{}]",
                encode_sequence(condition),
                encode_sequence(body)
            )
        }
        Instruction::For {
            initializer,
            condition,
            update,
            body,
        } => format!(
            "For[{}][{}][{}][{}]",
            encode_sequence(initializer),
            encode_sequence(condition),
            encode_sequence(update),
            encode_sequence(body)
        ),
        Instruction::Break => "Break".to_string(),
        Instruction::Continue => "Continue".to_string(),
    }
}

fn decode_instruction(token: &str) -> Result<Instruction, VmError> {
    if token == "Print" {
        return Ok(Instruction::Print);
    }
    if token == "Return" {
        return Ok(Instruction::Return);
    }
    if token == "Index" {
        return Ok(Instruction::Index);
    }
    if token == "StoreIndex" {
        return Ok(Instruction::StoreIndex);
    }
    if let Some(field) = token.strip_prefix("GetField:") {
        return Ok(Instruction::GetField(unescape_string(field)));
    }
    if let Some(field) = token.strip_prefix("StoreField:") {
        return Ok(Instruction::StoreField(unescape_string(field)));
    }
    if token == "Break" {
        return Ok(Instruction::Break);
    }
    if token == "Continue" {
        return Ok(Instruction::Continue);
    }
    if let Some(value) = token.strip_prefix("PushInt:") {
        let value = value.parse::<i64>().map_err(|_| VmError {
            message: format!("invalid integer literal '{value}'"),
        })?;
        return Ok(Instruction::PushInt(value));
    }
    if let Some(value) = token.strip_prefix("PushFloat:") {
        value.parse::<f64>().map_err(|_| VmError {
            message: format!("invalid float literal '{value}'"),
        })?;
        return Ok(Instruction::PushFloat(value.to_string()));
    }
    if let Some(value) = token.strip_prefix("PushBool:") {
        let value = match value {
            "true" => true,
            "false" => false,
            _ => {
                return Err(VmError {
                    message: format!("invalid boolean literal '{value}'"),
                })
            }
        };
        return Ok(Instruction::PushBool(value));
    }
    if let Some(value) = token.strip_prefix("PushString:") {
        return Ok(Instruction::PushString(unescape_string(value)));
    }
    if let Some(name) = token.strip_prefix("LoadVariable:") {
        return Ok(Instruction::LoadVariable(unescape_string(name)));
    }
    if let Some(name) = token.strip_prefix("StoreVariable:") {
        return Ok(Instruction::StoreVariable(unescape_string(name)));
    }
    if let Some(raw) = token.strip_prefix("MakeArray:") {
        let length = raw.parse::<usize>().map_err(|_| VmError {
            message: format!("invalid array length '{raw}'"),
        })?;
        return Ok(Instruction::MakeArray { length });
    }
    if let Some(raw) = token.strip_prefix("MakeStruct:") {
        let fields = if raw.is_empty() {
            Vec::new()
        } else {
            split_escaped(raw, '|')
                .into_iter()
                .map(|field| unescape_string(&field))
                .collect()
        };
        return Ok(Instruction::MakeStruct { fields });
    }
    if let Some(raw) = token.strip_prefix("Binary:") {
        return Ok(Instruction::Binary {
            op: decode_binary(raw)?,
        });
    }
    if let Some(raw) = token.strip_prefix("ShortCircuitAnd") {
        let (right, _) = extract_bracketed(raw)?;
        return Ok(Instruction::ShortCircuitAnd {
            right: decode_sequence(&right)?,
        });
    }
    if let Some(raw) = token.strip_prefix("ShortCircuitOr") {
        let (right, _) = extract_bracketed(raw)?;
        return Ok(Instruction::ShortCircuitOr {
            right: decode_sequence(&right)?,
        });
    }
    if let Some(raw) = token.strip_prefix("Unary:") {
        return Ok(Instruction::Unary {
            op: decode_unary(raw)?,
        });
    }
    if let Some(raw) = token.strip_prefix("Call:") {
        let parts = split_escaped(raw, '|');
        if parts.len() != 2 {
            return Err(VmError {
                message: format!("invalid call encoding '{token}'"),
            });
        }
        let name = unescape_string(&parts[0]);
        let argument_count = parts[1].parse::<usize>().map_err(|_| VmError {
            message: format!("invalid call arity '{token}'"),
        })?;
        return Ok(Instruction::Call {
            name,
            argument_count,
        });
    }
    if let Some(raw) = token.strip_prefix("If") {
        let (then_raw, rest) = extract_bracketed(raw)?;
        let (else_raw, _) = extract_bracketed(rest)?;
        return Ok(Instruction::If {
            then_body: decode_sequence(&then_raw)?,
            else_body: decode_sequence(&else_raw)?,
        });
    }
    if let Some(raw) = token.strip_prefix("While") {
        let (condition_raw, rest) = extract_bracketed(raw)?;
        let (body_raw, _) = extract_bracketed(rest)?;
        return Ok(Instruction::While {
            condition: decode_sequence(&condition_raw)?,
            body: decode_sequence(&body_raw)?,
        });
    }
    if let Some(raw) = token.strip_prefix("For") {
        let (initializer, rest) = extract_bracketed(raw)?;
        let (condition, rest) = extract_bracketed(rest)?;
        let (update, rest) = extract_bracketed(rest)?;
        let (body, _) = extract_bracketed(rest)?;
        return Ok(Instruction::For {
            initializer: decode_sequence(&initializer)?,
            condition: decode_sequence(&condition)?,
            update: decode_sequence(&update)?,
            body: decode_sequence(&body)?,
        });
    }

    Err(VmError {
        message: format!("unknown serialized instruction '{token}'"),
    })
}

fn encode_binary(op: &BinaryOperator) -> String {
    match op {
        BinaryOperator::Add => "Add".to_string(),
        BinaryOperator::Subtract => "Subtract".to_string(),
        BinaryOperator::Multiply => "Multiply".to_string(),
        BinaryOperator::Divide => "Divide".to_string(),
        BinaryOperator::Modulo => "Modulo".to_string(),
        BinaryOperator::Greater => "Greater".to_string(),
        BinaryOperator::GreaterEqual => "GreaterEqual".to_string(),
        BinaryOperator::Less => "Less".to_string(),
        BinaryOperator::LessEqual => "LessEqual".to_string(),
        BinaryOperator::Equal => "Equal".to_string(),
        BinaryOperator::NotEqual => "NotEqual".to_string(),
        BinaryOperator::And => "And".to_string(),
        BinaryOperator::Or => "Or".to_string(),
    }
}

fn decode_binary(value: &str) -> Result<BinaryOperator, VmError> {
    match value {
        "Add" => Ok(BinaryOperator::Add),
        "Subtract" => Ok(BinaryOperator::Subtract),
        "Multiply" => Ok(BinaryOperator::Multiply),
        "Divide" => Ok(BinaryOperator::Divide),
        "Modulo" => Ok(BinaryOperator::Modulo),
        "Greater" => Ok(BinaryOperator::Greater),
        "GreaterEqual" => Ok(BinaryOperator::GreaterEqual),
        "Less" => Ok(BinaryOperator::Less),
        "LessEqual" => Ok(BinaryOperator::LessEqual),
        "Equal" => Ok(BinaryOperator::Equal),
        "NotEqual" => Ok(BinaryOperator::NotEqual),
        "And" => Ok(BinaryOperator::And),
        "Or" => Ok(BinaryOperator::Or),
        _ => Err(VmError {
            message: format!("unknown binary operator '{value}'"),
        }),
    }
}

fn encode_unary(op: &UnaryOperator) -> String {
    match op {
        UnaryOperator::Not => "Not".to_string(),
        UnaryOperator::Negate => "Negate".to_string(),
    }
}

fn decode_unary(value: &str) -> Result<UnaryOperator, VmError> {
    match value {
        "Not" => Ok(UnaryOperator::Not),
        "Negate" => Ok(UnaryOperator::Negate),
        _ => Err(VmError {
            message: format!("unknown unary operator '{value}'"),
        }),
    }
}

fn split_escaped(input: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    let mut bracket_depth = 0usize;
    for character in input.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            current.push(character);
            escaped = true;
            continue;
        }
        if delimiter == ';' && character == '[' {
            bracket_depth += 1;
        } else if delimiter == ';' && character == ']' && bracket_depth > 0 {
            bracket_depth -= 1;
        }
        if character == delimiter && bracket_depth == 0 {
            parts.push(current.clone());
            current.clear();
        } else {
            current.push(character);
        }
    }
    parts.push(current);
    parts
}

fn extract_bracketed(input: &str) -> Result<(String, &str), VmError> {
    let mut escaped = false;
    let mut depth = 0usize;
    let mut start = None;
    let mut end = None;

    for (index, character) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character == '[' {
            if depth == 0 {
                start = Some(index + 1);
            }
            depth += 1;
            continue;
        }
        if character == ']' {
            if depth == 0 {
                return Err(VmError {
                    message: "invalid bracketed block".into(),
                });
            }
            depth -= 1;
            if depth == 0 {
                end = Some(index);
                break;
            }
        }
    }

    let start = start.ok_or_else(|| VmError {
        message: "missing block start".into(),
    })?;
    let end = end.ok_or_else(|| VmError {
        message: "missing block end".into(),
    })?;
    let content = input[start..end].to_string();
    let rest = &input[end + 1..];
    Ok((content, rest))
}

fn escape_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace('|', "\\|")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace(':', "\\:")
}

fn unescape_string(value: &str) -> String {
    let mut result = String::new();
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            result.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            result.push(character);
        }
    }
    result
}

fn render_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

fn compare_numeric<F>(left: &Value, right: &Value, compare: F) -> Result<bool, VmError>
where
    F: FnOnce(f64, f64) -> bool,
{
    match (left, right) {
        (Value::Int(left), Value::Int(right)) => Ok(compare(*left as f64, *right as f64)),
        (Value::Float(left), Value::Float(right)) => Ok(compare(
            left.parse::<f64>().map_err(|_| VmError {
                message: "invalid Float value".into(),
            })?,
            right.parse::<f64>().map_err(|_| VmError {
                message: "invalid Float value".into(),
            })?,
        )),
        _ => Err(VmError {
            message: "comparison requires matching numeric operands".into(),
        }),
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
    fn executes_compiled_integer_remainder() {
        let program = parse("fn main() { print(17 % 5) }").unwrap();
        let artifact = compile_program(&program);
        let decoded = Artifact::deserialize(&artifact.serialize()).unwrap();
        assert_eq!(execute_artifact(&decoded).unwrap(), "2\n");
    }

    #[test]
    fn executes_compiled_short_circuit_logic() {
        let program = parse(
            "fn main() { if false && 1 / 0 == 0 { print(\"bad\") } if true || 1 / 0 == 0 { print(\"ok\") } }",
        )
        .unwrap();
        let artifact = compile_program(&program);
        let decoded = Artifact::deserialize(&artifact.serialize()).unwrap();
        assert_eq!(execute_artifact(&decoded).unwrap(), "ok\n");
    }

    #[test]
    fn executes_compiled_struct_field_assignment() {
        let program = parse(
            "struct Point { x: Int, y: Int } fn main() { let point: Point = Point { x: 10, y: 20 }; point.x = 42; print(point.x) }",
        )
        .unwrap();
        let artifact = compile_program(&program);
        let decoded = Artifact::deserialize(&artifact.serialize()).unwrap();
        assert_eq!(execute_artifact(&decoded).unwrap(), "42\n");
    }

    #[test]
    fn executes_compiled_standard_library_calls() {
        let program = parse(
            r#"fn main() { print(len("axiom")); print(abs(-7)); print(min(3, 5)); print(max(3.0, 5.0)) }"#,
        )
        .unwrap();
        let artifact = compile_program(&program);
        let decoded = Artifact::deserialize(&artifact.serialize()).unwrap();
        assert_eq!(
            execute_artifact(&decoded).unwrap(),
            "5
7
3
5.0
"
        );
    }

    #[test]
    fn executes_compiled_struct_field_access() {
        let program = parse(
            "struct Point { x: Int, y: Int } fn main() { let point: Point = Point { x: 10, y: 20 }; print(point.x); print(point.y) }",
        )
        .unwrap();
        let artifact = compile_program(&program);
        let decoded = Artifact::deserialize(&artifact.serialize()).unwrap();
        assert_eq!(execute_artifact(&decoded).unwrap(), "10\n20\n");
    }

    #[test]
    fn executes_compiled_float_arithmetic() {
        let program =
            parse("fn main() { let value: Float = 1.5 + 2.5; print(value); print(value / 2.0) }")
                .unwrap();
        let artifact = compile_program(&program);
        let decoded = Artifact::deserialize(&artifact.serialize()).unwrap();
        assert_eq!(execute_artifact(&decoded).unwrap(), "4.0\n2.0\n");
    }

    #[test]
    fn executes_self_hosted_lexer() {
        let source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../bootstrap/lexer.ax"
        ))
        .unwrap();
        let source = source.replace(
            "bootstrap/lexer_fixture.ax",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../bootstrap/lexer_fixture.ax"),
        );
        let program = parse(&source).unwrap();
        let output = execute_program(&program).unwrap();
        assert!(output.contains("STRUCT|struct\n"));
        assert!(output.contains("ARROW|->\n"));
        assert!(output.contains("LBRACKET|[\n"));
        assert!(output.contains("GREATER_EQUAL|>=\n"));
        assert!(output.contains("AND|&&\n"));
        assert!(output.contains("FLOAT|2.5\n"));
        assert!(output.ends_with("RBRACE|}\n"));
    }

    #[test]
    fn executes_self_hosted_parser() {
        let lexer_source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../bootstrap/lexer.ax"
        ))
        .unwrap()
        .replace(
            "bootstrap/lexer_fixture.ax",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../bootstrap/lexer_fixture.ax"),
        );
        let lexer_program = parse(&lexer_source).unwrap();
        let tokens = execute_program(&lexer_program).unwrap();
        let token_path = std::env::temp_dir().join(format!(
            "axiom-self-hosted-parser-{}.tokens",
            std::process::id()
        ));
        std::fs::write(&token_path, tokens).unwrap();

        let source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../bootstrap/parser.ax"
        ))
        .unwrap()
        .replace("bootstrap/lexer_tokens.txt", token_path.to_str().unwrap());
        let program = parse(&source).unwrap();
        let output = execute_program(&program).unwrap();
        std::fs::remove_file(&token_path).unwrap();
        assert!(output.contains("Program\n"));
        assert!(output.contains("Struct: Point\n"));
        assert!(output.contains("Function: calculate\n"));
        assert!(output.contains("Array\n"));
        assert!(output.contains("Binary: >=\n"));
        assert!(output.contains("Else\n"));
        assert!(output.contains("While\n"));
        assert!(output.contains("Return\n"));
    }

    #[test]
    fn executes_self_hosted_ast_normalizer() {
        let parser_test = {
            let lexer_source = std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../bootstrap/lexer.ax"
            ))
            .unwrap()
            .replace(
                "bootstrap/lexer_fixture.ax",
                concat!(env!("CARGO_MANIFEST_DIR"), "/../bootstrap/lexer_fixture.ax"),
            );
            let lexer_program = parse(&lexer_source).unwrap();
            let tokens = execute_program(&lexer_program).unwrap();

            let token_path = std::env::temp_dir().join(format!(
                "axiom-self-hosted-ast-{}.tokens",
                std::process::id()
            ));
            let ast_input_path = std::env::temp_dir()
                .join(format!("axiom-self-hosted-ast-{}.txt", std::process::id()));
            std::fs::write(&token_path, tokens).unwrap();

            let parser_source = std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../bootstrap/parser.ax"
            ))
            .unwrap()
            .replace("bootstrap/lexer_tokens.txt", token_path.to_str().unwrap());
            let parser_program = parse(&parser_source).unwrap();
            let parser_output = execute_program(&parser_program).unwrap();
            std::fs::write(&ast_input_path, parser_output).unwrap();

            let ast_source = std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../bootstrap/ast.ax"
            ))
            .unwrap()
            .replace("bootstrap/parser_ast.txt", ast_input_path.to_str().unwrap());
            let ast_program = parse(&ast_source).unwrap();
            let ast_output = execute_program(&ast_program).unwrap();

            std::fs::remove_file(token_path).unwrap();
            std::fs::remove_file(ast_input_path).unwrap();
            ast_output
        };

        assert!(parser_test.starts_with("AXIOM_AST_V1\n"));
        assert!(parser_test.contains("NODE|0|Program|"));
        assert!(parser_test.contains("NODE|0|Struct|Point"));
        assert!(parser_test.contains("NODE|0|Function|calculate"));
        assert!(parser_test.contains("NODE|1|Return|"));
    }

    #[test]
    fn executes_self_hosted_semantic_analysis() {
        let lexer_source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../bootstrap/lexer.ax"
        ))
        .unwrap()
        .replace(
            "bootstrap/lexer_fixture.ax",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../bootstrap/lexer_fixture.ax"),
        );
        let lexer_program = parse(&lexer_source).unwrap();
        let tokens = execute_program(&lexer_program).unwrap();

        let token_path = std::env::temp_dir().join(format!(
            "axiom-self-hosted-semantic-{}.tokens",
            std::process::id()
        ));
        let ast_path = std::env::temp_dir().join(format!(
            "axiom-self-hosted-semantic-{}.ast",
            std::process::id()
        ));
        std::fs::write(&token_path, tokens).unwrap();

        let parser_source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../bootstrap/parser.ax"
        ))
        .unwrap()
        .replace("bootstrap/lexer_tokens.txt", token_path.to_str().unwrap());
        let parser_program = parse(&parser_source).unwrap();
        let parser_output = execute_program(&parser_program).unwrap();
        std::fs::write(&ast_path, parser_output).unwrap();

        let ast_source =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../bootstrap/ast.ax"))
                .unwrap()
                .replace("bootstrap/parser_ast.txt", ast_path.to_str().unwrap());
        let ast_program = parse(&ast_source).unwrap();
        let ast_output = execute_program(&ast_program).unwrap();

        let semantic_source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../bootstrap/semantic.ax"
        ))
        .unwrap()
        .replace("bootstrap/ast_output.txt", ast_path.to_str().unwrap());

        let semantic_input_path = std::env::temp_dir().join(format!(
            "axiom-self-hosted-semantic-input-{}.ast",
            std::process::id()
        ));
        std::fs::write(&semantic_input_path, ast_output).unwrap();
        let semantic_source = semantic_source.replace(
            ast_path.to_str().unwrap(),
            semantic_input_path.to_str().unwrap(),
        );
        let semantic_program = parse(&semantic_source).unwrap();
        let semantic_output = execute_program(&semantic_program).unwrap();

        std::fs::remove_file(token_path).unwrap();
        std::fs::remove_file(ast_path).unwrap();
        std::fs::remove_file(semantic_input_path).unwrap();

        assert!(semantic_output.contains("SEMANTIC_OK\n"));
        assert!(semantic_output.contains("STRUCTS|1\n"));
        assert!(semantic_output.contains("FUNCTIONS|2\n"));
    }

    #[test]
    fn self_hosted_semantic_rejects_mixed_literal_arithmetic() {
        let ast_path = std::env::temp_dir().join(format!(
            "axiom-self-hosted-semantic-negative-{}.ast",
            std::process::id()
        ));
        let ast = "AXIOM_AST_V1\nNODE|0|Program|\nNODE|0|Function|main\nNODE|0|ScopeEnter|\nNODE|1|ExprEnter|\nNODE|2|Integer|1\nNODE|2|Binary|+\nNODE|2|Float|2.0\nNODE|1|ExprExit|\nNODE|0|ScopeExit|\n";
        std::fs::write(&ast_path, ast).unwrap();
        let semantic_source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../bootstrap/semantic.ax"
        ))
        .unwrap()
        .replace("bootstrap/ast_output.txt", ast_path.to_str().unwrap());
        let program = parse(&semantic_source).unwrap();
        let output = execute_program(&program).unwrap();
        std::fs::remove_file(ast_path).unwrap();
        assert!(
            output.contains("SEMANTIC_ERROR|binary operator '+' has incompatible operand types")
        );
        assert!(!output.contains("SEMANTIC_OK"));
    }

    #[test]
    fn self_hosted_compiler_rebuilds_itself_reproducibly() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let compiler_source = std::fs::read_to_string(root.join("bootstrap/compiler.ax")).unwrap();
        let lexer_source = std::fs::read_to_string(root.join("bootstrap/lexer.ax")).unwrap();
        let lexer_source = lexer_source.replace(
            "bootstrap/lexer_fixture.ax",
            root.join("bootstrap/compiler.ax").to_str().unwrap(),
        );
        let lexer_program = parse(&lexer_source).unwrap();
        let tokens = execute_program(&lexer_program).unwrap();

        let token_path =
            std::env::temp_dir().join(format!("axiom-self-build-{}.tokens", std::process::id()));
        let ast_path =
            std::env::temp_dir().join(format!("axiom-self-build-{}.ast", std::process::id()));
        let compiler_input =
            std::env::temp_dir().join(format!("axiom-self-build-{}.input", std::process::id()));
        std::fs::write(&token_path, tokens).unwrap();

        let parser_source = std::fs::read_to_string(root.join("bootstrap/parser.ax"))
            .unwrap()
            .replace("bootstrap/lexer_tokens.txt", token_path.to_str().unwrap());
        let parser_program = parse(&parser_source).unwrap();
        let parser_output = execute_program(&parser_program).unwrap();
        std::fs::write(&ast_path, parser_output).unwrap();

        let ast_source = std::fs::read_to_string(root.join("bootstrap/ast.ax"))
            .unwrap()
            .replace("bootstrap/parser_ast.txt", ast_path.to_str().unwrap());
        let ast_program = parse(&ast_source).unwrap();
        let ast_output = execute_program(&ast_program).unwrap();
        std::fs::write(&compiler_input, ast_output).unwrap();

        let compile_source =
            compiler_source.replace("bootstrap/parser_ast.txt", compiler_input.to_str().unwrap());
        let compile_program = parse(&compile_source).unwrap();
        let first = execute_program(&compile_program).unwrap();
        let second = execute_program(&compile_program).unwrap();

        std::fs::remove_file(token_path).unwrap();
        std::fs::remove_file(ast_path).unwrap();
        std::fs::remove_file(compiler_input).unwrap();

        assert_eq!(first, second);
        assert!(first.starts_with("AXIOM_IR_V1\n"));
        assert!(first.contains("FUNCTION|main\n"));
        assert!(first.contains("CALL|"));
        assert!(first.contains("EXPR_ENTER\n"));
    }

    #[test]
    fn builds_ir_artifact() {
        let program = parse("fn main() { print(42) }").unwrap();
        let artifact = build_artifact(&program);
        assert!(artifact.contains("FUNCTION:"));
    }

    #[test]
    fn serializes_and_round_trips_artifact() {
        let program = parse("fn main() { print(42) }").unwrap();
        let artifact = compile_program(&program);
        let encoded = artifact.serialize();
        let decoded = Artifact::deserialize(&encoded).unwrap();
        assert_eq!(decoded.functions[0].name, "main");
        assert!(!decoded.functions[0].instructions.is_empty());
    }

    #[test]
    fn executes_deserialized_artifact() {
        let program = parse("fn main() { print(42) }").unwrap();
        let artifact = compile_program(&program);
        let encoded = artifact.serialize();
        let decoded = Artifact::deserialize(&encoded).unwrap();
        let output = execute_artifact(&decoded).unwrap();
        assert_eq!(output, "42\n");
    }

    #[test]
    fn executes_compiled_function_calls() {
        let program = parse(
            "fn add(a: Int, b: Int) -> Int { return a + b } fn main() { print(add(20, 22)) }",
        )
        .unwrap();
        let artifact = compile_program(&program);
        let encoded = artifact.serialize();
        let decoded = Artifact::deserialize(&encoded).unwrap();
        let output = execute_artifact(&decoded).unwrap();
        assert_eq!(output, "42\n");
    }

    #[test]
    fn executes_compiled_chained_array_indexing() {
        let program = parse("fn main() { print([[10, 20]][0][1]) }").unwrap();
        let artifact = compile_program(&program);
        let encoded = artifact.serialize();
        let decoded = Artifact::deserialize(&encoded).unwrap();
        let output = execute_artifact(&decoded).unwrap();
        assert_eq!(output, "20\n");
    }

    #[test]
    fn executes_compiled_nested_indexed_assignment() {
        let program = parse(
            "fn main() { let matrix = [[10, 20], [30, 40]]; matrix[1][0] = 99; print(matrix[1][0]) }",
        )
        .unwrap();
        let artifact = compile_program(&program);
        let encoded = artifact.serialize();
        let decoded = Artifact::deserialize(&encoded).unwrap();
        let output = execute_artifact(&decoded).unwrap();
        assert_eq!(output, "99\n");
    }

    #[test]
    fn executes_compiled_for_with_break_and_continue() {
        let program = parse(
            "fn main() { for (let i: Int = 0; i < 5; i = i + 1) { if i == 2 { continue } if i == 4 { break } print(i) } }",
        )
        .unwrap();
        let artifact = compile_program(&program);
        let decoded = Artifact::deserialize(&artifact.serialize()).unwrap();
        assert_eq!(execute_artifact(&decoded).unwrap(), "0\n1\n3\n");
    }

    #[test]
    fn writes_artifact_to_custom_output_path() {
        let program = parse("fn main() { print(42) }").unwrap();
        let temp_dir = std::env::temp_dir().join(format!("axiom-artifact-{}", std::process::id()));
        let output = temp_dir.join("custom-output.axm");
        let written = write_artifact_file_with_target(
            std::path::Path::new("examples/hello.ax"),
            Some(&output),
            &program,
        )
        .unwrap();
        assert_eq!(written, output);
        assert!(output.exists());
        let bytes = std::fs::read_to_string(&output).unwrap();
        assert!(bytes.starts_with("AXIOM_ARTIFACT_V1"));
        std::fs::remove_file(output).ok();
        std::fs::remove_dir_all(temp_dir).ok();
    }
}
