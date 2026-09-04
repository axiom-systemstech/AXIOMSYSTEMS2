use crate::{lex, LexError, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Call(Call),
    Let {
        name: String,
        value: Expression,
    },
    Return(Expression),
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
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
    Call(Call),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Greater,
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
            parameters.push(
                self.consume(TokenKind::Identifier, "expected parameter name")?
                    .lexeme,
            );
            while self.check(TokenKind::Comma) {
                self.position += 1;
                parameters.push(
                    self.consume(TokenKind::Identifier, "expected parameter name")?
                        .lexeme,
                );
            }
        }
        self.consume(TokenKind::RParen, "expected ')'")?;
        let body = self.block()?;
        Ok(Function {
            name,
            parameters,
            body,
        })
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
        if self.check(TokenKind::Let) {
            self.position += 1;
            let name = self
                .consume(TokenKind::Identifier, "expected variable name")?
                .lexeme;
            self.consume(TokenKind::Equal, "expected '='")?;
            return Ok(Statement::Let {
                name,
                value: self.expression()?,
            });
        }
        if self.check(TokenKind::Return) {
            self.position += 1;
            return Ok(Statement::Return(self.expression()?));
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
        let mut expression = self.primary()?;
        while self.check(TokenKind::Plus) {
            self.position += 1;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Add,
                right: Box::new(self.primary()?),
            };
        }
        if self.check(TokenKind::Greater) {
            self.position += 1;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Greater,
                right: Box::new(self.primary()?),
            };
        }
        Ok(expression)
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
}
