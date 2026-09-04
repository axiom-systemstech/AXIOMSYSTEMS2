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

    for statement in &main.body {
        match statement {
            Statement::Call(call) if call.name == "print" && call.arguments.len() == 1 => {
                if !matches!(
                    call.arguments[0],
                    Expression::String(_) | Expression::Integer(_) | Expression::Boolean(_)
                ) {
                    return Err(SemanticError {
                        message: "print expects a literal argument".into(),
                    });
                }
            }
            Statement::Call(call) if call.name == "print" => {
                return Err(SemanticError {
                    message: "print expects exactly one argument".into(),
                });
            }
            Statement::Call(call) => {
                return Err(SemanticError {
                    message: format!("unknown function '{}'", call.name),
                });
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
