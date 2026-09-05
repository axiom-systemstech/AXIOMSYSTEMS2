"""Lexer for the first AXIOM language syntax slice."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum, auto


class TokenKind(Enum):
    FN = auto()
    LET = auto()
    RETURN = auto()
    IF = auto()
    ELSE = auto()
    WHILE = auto()
    FOR = auto()
    BREAK = auto()
    CONTINUE = auto()
    STRUCT = auto()
    TRUE = auto()
    FALSE = auto()
    IDENTIFIER = auto()
    INTEGER = auto()
    FLOAT = auto()
    STRING = auto()
    LPAREN = auto()
    RPAREN = auto()
    LBRACE = auto()
    RBRACE = auto()
    LBRACKET = auto()
    RBRACKET = auto()
    SEMICOLON = auto()
    COLON = auto()
    EQUAL = auto()
    PLUS = auto()
    COMMA = auto()
    ARROW = auto()
    GREATER = auto()
    GREATER_EQUAL = auto()
    EQUAL_EQUAL = auto()
    NOT_EQUAL = auto()
    MINUS = auto()
    STAR = auto()
    PERCENT = auto()
    SLASH = auto()
    BANG = auto()
    AND = auto()
    OR = auto()
    LESS = auto()
    LESS_EQUAL = auto()
    DOT = auto()
    EOF = auto()


@dataclass(frozen=True)
class Token:
    kind: TokenKind
    lexeme: str
    line: int
    column: int


class LexError(ValueError):
    """Raised when source text cannot be converted into tokens."""


_KEYWORDS = {"fn": TokenKind.FN, "let": TokenKind.LET, "return": TokenKind.RETURN, "if": TokenKind.IF, "else": TokenKind.ELSE, "while": TokenKind.WHILE, "for": TokenKind.FOR, "break": TokenKind.BREAK, "continue": TokenKind.CONTINUE, "struct": TokenKind.STRUCT, "true": TokenKind.TRUE, "false": TokenKind.FALSE}


def lex(source: str) -> list[Token]:
    tokens: list[Token] = []
    index = 0
    line = 1
    column = 1

    while index < len(source):
        character = source[index]
        if character in " \t\r":
            index += 1
            column += 1
            continue
        if character == "\n":
            index += 1
            line += 1
            column = 1
            continue
        if character == "/" and source.startswith("//", index):
            index += 2
            column += 2
            while index < len(source) and source[index] != "\n":
                index += 1
                column += 1
            continue

        token_line, token_column = line, column
        if source.startswith("->", index):
            tokens.append(Token(TokenKind.ARROW, "->", token_line, token_column))
            index += 2
            column += 2
            continue
        if source.startswith("==", index):
            tokens.append(Token(TokenKind.EQUAL_EQUAL, "==", token_line, token_column))
            index += 2
            column += 2
            continue
        if source.startswith("!=", index):
            tokens.append(Token(TokenKind.NOT_EQUAL, "!=", token_line, token_column))
            index += 2
            column += 2
            continue
        if source.startswith(">=", index):
            tokens.append(Token(TokenKind.GREATER_EQUAL, ">=", token_line, token_column))
            index += 2
            column += 2
            continue
        if source.startswith("<=", index):
            tokens.append(Token(TokenKind.LESS_EQUAL, "<=", token_line, token_column))
            index += 2
            column += 2
            continue
        if source.startswith("&&", index):
            tokens.append(Token(TokenKind.AND, "&&", token_line, token_column))
            index += 2
            column += 2
            continue
        if source.startswith("||", index):
            tokens.append(Token(TokenKind.OR, "||", token_line, token_column))
            index += 2
            column += 2
            continue
        punctuation = {
            "(": TokenKind.LPAREN,
            ")": TokenKind.RPAREN,
            "{": TokenKind.LBRACE,
            "}": TokenKind.RBRACE,
            "[": TokenKind.LBRACKET,
            "]": TokenKind.RBRACKET,
            ";": TokenKind.SEMICOLON,
            ":": TokenKind.COLON,
            "=": TokenKind.EQUAL,
            "+": TokenKind.PLUS,
            ",": TokenKind.COMMA,
            ">": TokenKind.GREATER,
            "<": TokenKind.LESS,
            "-": TokenKind.MINUS,
            "*": TokenKind.STAR,
            "%": TokenKind.PERCENT,
            "/": TokenKind.SLASH,
            "!": TokenKind.BANG,
            ".": TokenKind.DOT,
        }
        if character in punctuation:
            tokens.append(Token(punctuation[character], character, token_line, token_column))
            index += 1
            column += 1
            continue

        if character == '"':
            start = index
            index += 1
            column += 1
            while index < len(source) and source[index] != '"':
                if source[index] == "\n":
                    raise LexError(f"unterminated string at {token_line}:{token_column}")
                index += 1
                column += 1
            if index == len(source):
                raise LexError(f"unterminated string at {token_line}:{token_column}")
            index += 1
            column += 1
            tokens.append(Token(TokenKind.STRING, source[start:index], token_line, token_column))
            continue

        if character.isalpha() or character == "_":
            start = index
            while index < len(source) and (source[index].isalnum() or source[index] == "_"):
                index += 1
                column += 1
            lexeme = source[start:index]
            tokens.append(Token(_KEYWORDS.get(lexeme, TokenKind.IDENTIFIER), lexeme, token_line, token_column))
            continue

        if character.isdigit():
            start = index
            while index < len(source) and source[index].isdigit():
                index += 1
                column += 1
            kind = TokenKind.INTEGER
            if index < len(source) and source[index] == "." and index + 1 < len(source) and source[index + 1].isdigit():
                kind = TokenKind.FLOAT
                index += 1
                column += 1
                while index < len(source) and source[index].isdigit():
                    index += 1
                    column += 1
            tokens.append(Token(kind, source[start:index], token_line, token_column))
            continue

        raise LexError(f"unexpected character {character!r} at {token_line}:{token_column}")

    tokens.append(Token(TokenKind.EOF, "", line, column))
    return tokens