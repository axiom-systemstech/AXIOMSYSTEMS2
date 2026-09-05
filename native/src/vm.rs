use std::collections::HashMap;

use crate::ir::{lower_program, Instruction, LoweredFunction};
use crate::parser::{BinaryOperator, Program, UnaryOperator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
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
                Instruction::Binary { op } => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let value = match op {
                        BinaryOperator::Add => Value::Int(left.as_int()? + right.as_int()?),
                        BinaryOperator::Subtract => Value::Int(left.as_int()? - right.as_int()?),
                        BinaryOperator::Multiply => Value::Int(left.as_int()? * right.as_int()?),
                        BinaryOperator::Divide => {
                            let divisor = right.as_int()?;
                            if divisor == 0 {
                                return Err(VmError {
                                    message: "division by zero".into(),
                                });
                            }
                            Value::Int(left.as_int()? / divisor)
                        }
                        BinaryOperator::Greater => Value::Bool(left.as_int()? > right.as_int()?),
                        BinaryOperator::Less => Value::Bool(left.as_int()? < right.as_int()?),
                        BinaryOperator::Equal => Value::Bool(left == right),
                        BinaryOperator::And => Value::Bool(left.as_bool()? && right.as_bool()?),
                        BinaryOperator::Or => Value::Bool(left.as_bool()? || right.as_bool()?),
                    };
                    self.stack.push(value);
                }
                Instruction::Unary { op } => {
                    let value = self.pop_stack()?;
                    let result = match op {
                        UnaryOperator::Not => Value::Bool(!value.as_bool()?),
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
                Instruction::While { condition, body } => loop {
                    self.execute_block(condition, locals)?;
                    let condition_value = self.pop_stack()?.as_bool()?;
                    if !condition_value {
                        break;
                    }
                    if let Some(value) = self.execute_block(body, locals)? {
                        return Ok(Some(value));
                    }
                },
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
        Instruction::PushBool(value) => {
            format!("PushBool:{}", if *value { "true" } else { "false" })
        }
        Instruction::PushString(value) => format!("PushString:{}", escape_string(value)),
        Instruction::LoadVariable(name) => format!("LoadVariable:{}", escape_string(name)),
        Instruction::StoreVariable(name) => format!("StoreVariable:{}", escape_string(name)),
        Instruction::MakeArray { length } => format!("MakeArray:{length}"),
        Instruction::Binary { op } => format!("Binary:{}", encode_binary(op)),
        Instruction::Unary { op } => format!("Unary:{}", encode_unary(op)),
        Instruction::Call {
            name,
            argument_count,
        } => {
            format!("Call:{}|{}", escape_string(name), argument_count)
        }
        Instruction::Index => "Index".to_string(),
        Instruction::StoreIndex => "StoreIndex".to_string(),
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
    if let Some(value) = token.strip_prefix("PushInt:") {
        let value = value.parse::<i64>().map_err(|_| VmError {
            message: format!("invalid integer literal '{value}'"),
        })?;
        return Ok(Instruction::PushInt(value));
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
    if let Some(raw) = token.strip_prefix("Binary:") {
        return Ok(Instruction::Binary {
            op: decode_binary(raw)?,
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
        BinaryOperator::Greater => "Greater".to_string(),
        BinaryOperator::Less => "Less".to_string(),
        BinaryOperator::Equal => "Equal".to_string(),
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
        "Greater" => Ok(BinaryOperator::Greater),
        "Less" => Ok(BinaryOperator::Less),
        "Equal" => Ok(BinaryOperator::Equal),
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
    }
}

fn decode_unary(value: &str) -> Result<UnaryOperator, VmError> {
    match value {
        "Not" => Ok(UnaryOperator::Not),
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
