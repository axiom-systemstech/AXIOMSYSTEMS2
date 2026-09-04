use crate::{lex, LexError, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub type_name: Option<Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Call(Call),
    Let {
        name: String,
        type_name: Option<Type>,
        value: Expression,
    },
    Return(Expression),
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    Assign {
        name: String,
        value: Expression,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub name: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    String(String),
    Integer(i64),
    Boolean(bool),
    Variable(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Call(Call),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Greater,
    Equal,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = lex(source).map_err(ParseError::from)?;
    Parser {
        tokens,
        position: 0,
    }
    .parse()
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn parse(mut self) -> Result<Program, ParseError> {
        let mut functions = Vec::new();
        while !self.check(TokenKind::Eof) {
            functions.push(self.function()?);
        }
        Ok(Program { functions })
    }

    fn function(&mut self) -> Result<Function, ParseError> {
        self.consume(TokenKind::Fn, "expected 'fn'")?;
        let name = self
            .consume(TokenKind::Identifier, "expected function name")?
            .lexeme;
        self.consume(TokenKind::LParen, "expected '('")?;
        let mut parameters = Vec::new();
        if !self.check(TokenKind::RParen) {
            parameters.push(self.parameter()?);
            while self.check(TokenKind::Comma) {
                self.position += 1;
                parameters.push(self.parameter()?);
            }
        }
        self.consume(TokenKind::RParen, "expected ')'")?;
        let return_type = if self.check(TokenKind::Arrow) {
            self.position += 1;
            Some(self.type_name()?)
        } else {
            None
        };
        let body = self.block()?;
        Ok(Function {
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn parameter(&mut self) -> Result<Parameter, ParseError> {
        let name = self
            .consume(TokenKind::Identifier, "expected parameter name")?
            .lexeme;
        let type_name = if self.check(TokenKind::Colon) {
            self.position += 1;
            Some(self.type_name()?)
        } else {
            None
        };
        Ok(Parameter { name, type_name })
    }

    fn type_name(&mut self) -> Result<Type, ParseError> {
        let token = self.consume(TokenKind::Identifier, "expected type name")?;
        match token.lexeme.as_str() {
            "Int" => Ok(Type::Int),
            "Bool" => Ok(Type::Bool),
            "String" => Ok(Type::String),
            _ => Err(ParseError {
                message: format!("unknown type '{}'", token.lexeme),
                line: token.line,
                column: token.column,
            }),
        }
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.check(TokenKind::If) {
            self.position += 1;
            let condition = self.expression()?;
            let then_body = self.block()?;
            let else_body = if self.check(TokenKind::Else) {
                self.position += 1;
                self.block()?
            } else {
                Vec::new()
            };
            return Ok(Statement::If {
                condition,
                then_body,
                else_body,
            });
        }
        if self.check(TokenKind::While) {
            self.position += 1;
            let condition = self.expression()?;
            let body = self.block()?;
            return Ok(Statement::While { condition, body });
        }
        if self.check(TokenKind::Let) {
            self.position += 1;
            let name = self
                .consume(TokenKind::Identifier, "expected variable name")?
                .lexeme;
            let type_name = if self.check(TokenKind::Colon) {
                self.position += 1;
                Some(self.type_name()?)
            } else {
                None
            };
            self.consume(TokenKind::Equal, "expected '='")?;
            return Ok(Statement::Let {
                name,
                type_name,
                value: self.expression()?,
            });
        }
        if self.check(TokenKind::Return) {
            self.position += 1;
            return Ok(Statement::Return(self.expression()?));
        }
        if self.check(TokenKind::Identifier)
            && self
                .tokens
                .get(self.position + 1)
                .is_some_and(|token| token.kind == TokenKind::Equal)
        {
            let name = self.current().lexeme.clone();
            self.position += 2;
            return Ok(Statement::Assign {
                name,
                value: self.expression()?,
            });
        }
        self.call()
    }

    fn block(&mut self) -> Result<Vec<Statement>, ParseError> {
        self.consume(TokenKind::LBrace, "expected '{'")?;
        let mut body = Vec::new();
        while !self.check(TokenKind::RBrace) {
            if self.check(TokenKind::Eof) {
                return Err(self.error("expected '}'"));
            }
            body.push(self.statement()?);
            if self.check(TokenKind::Semicolon) {
                self.position += 1;
            }
        }
        self.consume(TokenKind::RBrace, "expected '}'")?;
        Ok(body)
    }

    fn call(&mut self) -> Result<Statement, ParseError> {
        let name = self
            .consume(TokenKind::Identifier, "expected call name")?
            .lexeme;
        self.consume(TokenKind::LParen, "expected '('")?;
        let mut arguments = Vec::new();
        if !self.check(TokenKind::RParen) {
            arguments.push(self.expression()?);
            while self.check(TokenKind::Comma) {
                self.position += 1;
                arguments.push(self.expression()?);
            }
        }
        self.consume(TokenKind::RParen, "expected ')'")?;
        Ok(Statement::Call(Call { name, arguments }))
    }

    fn expression(&mut self) -> Result<Expression, ParseError> {
        self.or()
    }

    fn or(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.and()?;
        while self.check(TokenKind::Or) {
            self.position += 1;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Or,
                right: Box::new(self.and()?),
            };
        }
        Ok(expression)
    }

    fn and(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.equality()?;
        while self.check(TokenKind::And) {
            self.position += 1;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::And,
                right: Box::new(self.equality()?),
            };
        }
        Ok(expression)
    }

    fn equality(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.comparison()?;
        while self.check(TokenKind::EqualEqual) {
            self.position += 1;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Equal,
                right: Box::new(self.comparison()?),
            };
        }
        Ok(expression)
    }

    fn comparison(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.term()?;
        while self.check(TokenKind::Greater) {
            self.position += 1;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Greater,
                right: Box::new(self.term()?),
            };
        }
        Ok(expression)
    }

    fn term(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.factor()?;
        while self.check(TokenKind::Plus) || self.check(TokenKind::Minus) {
            let operator = if self.check(TokenKind::Plus) {
                BinaryOperator::Add
            } else {
                BinaryOperator::Subtract
            };
            self.position += 1;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(self.factor()?),
            };
        }
        Ok(expression)
    }

    fn factor(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.unary()?;
        while self.check(TokenKind::Star) || self.check(TokenKind::Slash) {
            let operator = if self.check(TokenKind::Star) {
                BinaryOperator::Multiply
            } else {
                BinaryOperator::Divide
            };
            self.position += 1;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(self.unary()?),
            };
        }
        Ok(expression)
    }

    fn unary(&mut self) -> Result<Expression, ParseError> {
        if self.check(TokenKind::Bang) {
            self.position += 1;
            return Ok(Expression::Unary {
                operator: UnaryOperator::Not,
                operand: Box::new(self.unary()?),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expression, ParseError> {
        let token = self.current().clone();
        match token.kind {
            TokenKind::String => {
                self.position += 1;
                Ok(Expression::String(
                    token.lexeme[1..token.lexeme.len() - 1].to_owned(),
                ))
            }
            TokenKind::Integer => {
                self.position += 1;
                Ok(Expression::Integer(
                    token.lexeme.parse().expect("lexer emitted integer"),
                ))
            }
            TokenKind::True => {
                self.position += 1;
                Ok(Expression::Boolean(true))
            }
            TokenKind::False => {
                self.position += 1;
                Ok(Expression::Boolean(false))
            }
            TokenKind::Identifier => {
                let name = token.lexeme.clone();
                self.position += 1;
                if self.check(TokenKind::LParen) {
                    self.position += 1;
                    let mut arguments = Vec::new();
                    if !self.check(TokenKind::RParen) {
                        arguments.push(self.expression()?);
                        while self.check(TokenKind::Comma) {
                            self.position += 1;
                            arguments.push(self.expression()?);
                        }
                    }
                    self.consume(TokenKind::RParen, "expected ')'")?;
                    Ok(Expression::Call(Call { name, arguments }))
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            _ => Err(self.error("expected literal")),
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<Token, ParseError> {
        if self.check(kind) {
            let token = self.current().clone();
            self.position += 1;
            Ok(token)
        } else {
            Err(self.error(message))
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current().kind == kind
    }

    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_owned(),
            line: self.current().line,
            column: self.current().column,
        }
    }
}

impl From<LexError> for ParseError {
    fn from(error: LexError) -> Self {
        Self {
            message: error.message,
            line: error.line,
            column: error.column,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_program() {
        let program = parse("fn main() { print(\"Hello AXIOM\") }").unwrap();
        assert_eq!(program.functions[0].name, "main");
        assert_eq!(program.functions[0].body.len(), 1);
    }

    #[test]
    fn reports_missing_body() {
        let error = parse("fn main()").unwrap_err();
        assert_eq!(error.message, "expected '{'");
        assert_eq!(error.line, 1);
    }

    #[test]
    fn parses_explicit_types() {
        let program = parse("fn add(a: Int, b: Int) -> Int { return a + b }").unwrap();
        assert_eq!(
            program.functions[0].parameters[0].type_name,
            Some(Type::Int)
        );
        assert_eq!(program.functions[0].return_type, Some(Type::Int));
    }

    #[test]
    fn rejects_unknown_type() {
        let error = parse("fn main(value: Any) { print(value) }").unwrap_err();
        assert_eq!(error.message, "unknown type 'Any'");
    }
}
