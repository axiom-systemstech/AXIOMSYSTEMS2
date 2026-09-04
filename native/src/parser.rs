use crate::{lex, LexError, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Call(Call),
    Let { name: String, value: Expression },
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
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
        self.consume(TokenKind::RParen, "expected ')'")?;
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
        Ok(Function { name, body })
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
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
        self.call()
    }

    fn call(&mut self) -> Result<Statement, ParseError> {
        let name = self
            .consume(TokenKind::Identifier, "expected call name")?
            .lexeme;
        self.consume(TokenKind::LParen, "expected '('")?;
        let mut arguments = Vec::new();
        if !self.check(TokenKind::RParen) {
            arguments.push(self.expression()?);
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
        Ok(expression)
    }

    fn primary(&mut self) -> Result<Expression, ParseError> {
        let token = self.current();
        match token.kind {
            TokenKind::String => Ok(Expression::String(
                token.lexeme[1..token.lexeme.len() - 1].to_owned(),
            )),
            TokenKind::Integer => Ok(Expression::Integer(
                token.lexeme.parse().expect("lexer emitted integer"),
            )),
            TokenKind::True => Ok(Expression::Boolean(true)),
            TokenKind::False => Ok(Expression::Boolean(false)),
            TokenKind::Identifier => Ok(Expression::Variable(token.lexeme.clone())),
            _ => Err(self.error("expected literal")),
        }
        .inspect(|_| self.position += 1)
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
