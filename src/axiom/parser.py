"""Recursive-descent parser for the first AXIOM syntax slice."""

from __future__ import annotations

from .ast import ArrayLiteral, Assign, Binary, BooleanLiteral, Break, Call, Continue, FieldAccess, FloatLiteral, For, Function, If, Index, IntegerLiteral, Let, Parameter, Program, Return, StringLiteral, StructDefinition, StructLiteral, Unary, Variable, While
from .lexer import Token, TokenKind, lex


class ParseError(ValueError):
    """Raised when tokens do not match the AXIOM grammar."""


class Parser:
    def __init__(self, tokens: list[Token]):
        self.tokens = tokens
        self.position = 0

    def parse(self) -> Program:
        functions = []
        structs = []
        while not self._check(TokenKind.EOF):
            if self._check(TokenKind.STRUCT):
                structs.append(self._struct())
            else:
                functions.append(self._function())
        return Program(functions, structs)

    def _struct(self) -> StructDefinition:
        self._consume(TokenKind.STRUCT, "expected 'struct'")
        name = self._consume(TokenKind.IDENTIFIER, "expected struct name").lexeme
        self._consume(TokenKind.LBRACE, "expected '{'")
        fields = []
        while not self._check(TokenKind.RBRACE):
            field_name = self._consume(TokenKind.IDENTIFIER, "expected field name").lexeme
            self._consume(TokenKind.COLON, "expected ':'")
            fields.append(Parameter(field_name, self._type_name()))
            if self._check(TokenKind.COMMA) or self._check(TokenKind.SEMICOLON):
                self.position += 1
        self._consume(TokenKind.RBRACE, "expected '}'")
        return StructDefinition(name, fields)

    def _function(self) -> Function:
        self._consume(TokenKind.FN, "expected 'fn'")
        name = self._consume(TokenKind.IDENTIFIER, "expected function name").lexeme
        self._consume(TokenKind.LPAREN, "expected '('")
        parameters = []
        if not self._check(TokenKind.RPAREN):
            parameters.append(self._parameter())
            while self._check(TokenKind.COMMA):
                self.position += 1
                parameters.append(self._parameter())
        self._consume(TokenKind.RPAREN, "expected ')'")
        return_type = None
        if self._check(TokenKind.ARROW):
            self.position += 1
            return_type = self._type_name()
        body = self._block()
        return Function(name, body, parameters, return_type)

    def _block(self):
        self._consume(TokenKind.LBRACE, "expected '{'")
        body = []
        while not self._check(TokenKind.RBRACE):
            if self._check(TokenKind.EOF):
                self._error(self._current(), "expected '}'")
            body.append(self._statement())
            if self._check(TokenKind.SEMICOLON):
                self.position += 1
        self._consume(TokenKind.RBRACE, "expected '}'")
        return body

    def _parameter(self) -> Parameter:
        name = self._consume(TokenKind.IDENTIFIER, "expected parameter name").lexeme
        self._consume(TokenKind.COLON, "expected ':'")
        type_name = self._type_name()
        return Parameter(name, type_name)

    def _type_name(self) -> str:
        type_name = self._consume(TokenKind.IDENTIFIER, "expected type name").lexeme
        while self._check(TokenKind.LBRACKET):
            self.position += 1
            self._consume(TokenKind.RBRACKET, "expected ']' in array type")
            type_name += "[]"
        return type_name

    def _statement(self) -> Call | Let:
        if self._check(TokenKind.LET):
            self.position += 1
            name = self._consume(TokenKind.IDENTIFIER, "expected variable name").lexeme
            type_name = None
            if self._check(TokenKind.COLON):
                self.position += 1
                type_name = self._type_name()
            self._consume(TokenKind.EQUAL, "expected '='")
            return Let(name, type_name, self._expression())
        if self._check(TokenKind.RETURN):
            self.position += 1
            return Return(self._expression())
        if self._check(TokenKind.IF):
            self.position += 1
            condition = self._expression()
            then_body = self._block()
            else_body = []
            if self._check(TokenKind.ELSE):
                self.position += 1
                if self._check(TokenKind.IF):
                    else_body = [self._statement()]
                else:
                    else_body = self._block()
            return If(condition, then_body, else_body)
        if self._check(TokenKind.WHILE):
            self.position += 1
            return While(self._expression(), self._block())
        if self._check(TokenKind.BREAK):
            self.position += 1
            return Break()
        if self._check(TokenKind.CONTINUE):
            self.position += 1
            return Continue()
        if self._check(TokenKind.FOR):
            self.position += 1
            self._consume(TokenKind.LPAREN, "expected '('")
            initializer = None
            if not self._check(TokenKind.SEMICOLON):
                checkpoint = self.position
                if self._check(TokenKind.LET):
                    self.position += 1
                    name = self._consume(TokenKind.IDENTIFIER, "expected variable name").lexeme
                    type_name = None
                    if self._check(TokenKind.COLON):
                        self.position += 1
                        type_name = self._type_name()
                    self._consume(TokenKind.EQUAL, "expected '='")
                    initializer = Let(name, type_name, self._expression())
                else:
                    target = self._primary()
                    if not self._check(TokenKind.EQUAL):
                        self.position = checkpoint
                        self._error(self._current(), "expected loop initializer")
                    self.position += 1
                    initializer = Assign(target, self._expression())
            self._consume(TokenKind.SEMICOLON, "expected ';'")
            condition = self._expression()
            self._consume(TokenKind.SEMICOLON, "expected ';'")
            update = None
            if not self._check(TokenKind.RPAREN):
                checkpoint = self.position
                target = self._primary()
                if not self._check(TokenKind.EQUAL):
                    self.position = checkpoint
                    self._error(self._current(), "expected loop update")
                self.position += 1
                update = Assign(target, self._expression())
            self._consume(TokenKind.RPAREN, "expected ')' ")
            return For(initializer, condition, update, self._block())
        if self._check(TokenKind.IDENTIFIER):
            checkpoint = self.position
            target = self._primary()
            if self._check(TokenKind.EQUAL):
                self.position += 1
                return Assign(target, self._expression())
            self.position = checkpoint
        expression = self._expression()
        if not isinstance(expression, Call):
            self._error(self._current(), "expected statement")
        return expression

    def _call(self) -> Call:
        name = self._consume(TokenKind.IDENTIFIER, "expected call name").lexeme
        self._consume(TokenKind.LPAREN, "expected '('")
        arguments = []
        if not self._check(TokenKind.RPAREN):
            arguments.append(self._expression())
            while self._check(TokenKind.COMMA):
                self.position += 1
                arguments.append(self._expression())
        self._consume(TokenKind.RPAREN, "expected ')'")
        return Call(name, arguments)

    def _string(self) -> StringLiteral:
        token = self._consume(TokenKind.STRING, "expected string literal")
        return StringLiteral(token.lexeme[1:-1])

    def _expression(self):
        return self._or()

    def _or(self):
        expression = self._and()
        while self._check(TokenKind.OR):
            self.position += 1
            expression = Binary(expression, "||", self._and())
        return expression

    def _and(self):
        expression = self._equality()
        while self._check(TokenKind.AND):
            self.position += 1
            expression = Binary(expression, "&&", self._equality())
        return expression

    def _equality(self):
        expression = self._comparison()
        while self._check(TokenKind.EQUAL_EQUAL) or self._check(TokenKind.NOT_EQUAL):
            operator = self._current().lexeme
            self.position += 1
            expression = Binary(expression, operator, self._comparison())
        return expression

    def _comparison(self):
        expression = self._term()
        while (
            self._check(TokenKind.GREATER)
            or self._check(TokenKind.GREATER_EQUAL)
            or self._check(TokenKind.LESS)
            or self._check(TokenKind.LESS_EQUAL)
        ):
            operator = self._current().lexeme
            self.position += 1
            expression = Binary(expression, operator, self._term())
        return expression

    def _term(self):
        expression = self._factor()
        while self._check(TokenKind.PLUS) or self._check(TokenKind.MINUS):
            operator = self._current().lexeme
            self.position += 1
            expression = Binary(expression, operator, self._factor())
        return expression

    def _factor(self):
        expression = self._unary()
        while self._check(TokenKind.STAR) or self._check(TokenKind.SLASH) or self._check(TokenKind.PERCENT):
            operator = self._current().lexeme
            self.position += 1
            expression = Binary(expression, operator, self._unary())
        return expression

    def _unary(self):
        if self._check(TokenKind.BANG) or self._check(TokenKind.MINUS):
            operator = self._current().lexeme
            self.position += 1
            return Unary(operator, self._unary())
        return self._primary()

    def _primary(self):
        token = self._current()
        if token.kind == TokenKind.LPAREN:
            self.position += 1
            expression = self._expression()
            self._consume(TokenKind.RPAREN, "expected ')'")
            return expression
        if token.kind == TokenKind.STRING:
            return self._string()
        if token.kind == TokenKind.INTEGER:
            self.position += 1
            return IntegerLiteral(int(token.lexeme))
        if token.kind == TokenKind.FLOAT:
            self.position += 1
            return FloatLiteral(float(token.lexeme))
        if token.kind in (TokenKind.TRUE, TokenKind.FALSE):
            self.position += 1
            return BooleanLiteral(token.kind == TokenKind.TRUE)
        if token.kind == TokenKind.IDENTIFIER:
            self.position += 1
            if self._check(TokenKind.LBRACE):
                self.position += 1
                fields = []
                while not self._check(TokenKind.RBRACE):
                    field_name = self._consume(TokenKind.IDENTIFIER, "expected field name").lexeme
                    self._consume(TokenKind.COLON, "expected ':'")
                    fields.append((field_name, self._expression()))
                    if self._check(TokenKind.COMMA) or self._check(TokenKind.SEMICOLON):
                        self.position += 1
                self._consume(TokenKind.RBRACE, "expected '}'")
                expression = StructLiteral(token.lexeme, fields)
            elif self._check(TokenKind.LPAREN):
                expression = self._finish_call(token.lexeme)
            else:
                expression = Variable(token.lexeme)
        elif token.kind == TokenKind.LBRACKET:
            self.position += 1
            elements = []
            if not self._check(TokenKind.RBRACKET):
                elements.append(self._expression())
                while self._check(TokenKind.COMMA):
                    self.position += 1
                    elements.append(self._expression())
            self._consume(TokenKind.RBRACKET, "expected ']'")
            expression = ArrayLiteral(elements)
        else:
            self._error(token, "expected expression")
        while self._check(TokenKind.LBRACKET):
            self.position += 1
            index = self._expression()
            self._consume(TokenKind.RBRACKET, "expected ']'")
            expression = Index(expression, index)
        while self._check(TokenKind.DOT):
            self.position += 1
            field = self._consume(TokenKind.IDENTIFIER, "expected field name").lexeme
            expression = FieldAccess(expression, field)
        return expression

    def _finish_call(self, name: str) -> Call:
        self._consume(TokenKind.LPAREN, "expected '('")
        arguments = []
        if not self._check(TokenKind.RPAREN):
            arguments.append(self._expression())
            while self._check(TokenKind.COMMA):
                self.position += 1
                arguments.append(self._expression())
        self._consume(TokenKind.RPAREN, "expected ')'")
        return Call(name, arguments)

    def _consume(self, kind: TokenKind, message: str) -> Token:
        if self._check(kind):
            token = self._current()
            self.position += 1
            return token
        self._error(self._current(), message)

    def _check(self, kind: TokenKind) -> bool:
        return self._current().kind == kind

    def _current(self) -> Token:
        return self.tokens[self.position]

    def _peek(self) -> Token:
        return self.tokens[min(self.position + 1, len(self.tokens) - 1)]

    @staticmethod
    def _error(token: Token, message: str) -> None:
        raise ParseError(f"{message} at {token.line}:{token.column}")


def parse(source: str) -> Program:
    return Parser(lex(source)).parse()