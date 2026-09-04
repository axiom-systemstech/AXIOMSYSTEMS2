pub mod ir;
pub mod parser;
pub mod runtime;
pub mod semantic;
pub mod vm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Fn,
    Let,
    Return,
    If,
    Else,
    While,
    True,
    False,
    Identifier,
    Integer,
    String,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semicolon,
    Colon,
    Equal,
    Plus,
    Comma,
    Arrow,
    Greater,
    EqualEqual,
    Minus,
    Star,
    Slash,
    Bang,
    And,
    Or,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let characters: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut line = 1;
    let mut column = 1;

    while index < characters.len() {
        let character = characters[index];
        if matches!(character, ' ' | '\t' | '\r') {
            index += 1;
            column += 1;
            continue;
        }
        if character == '\n' {
            index += 1;
            line += 1;
            column = 1;
            continue;
        }

        let token_line = line;
        let token_column = column;
        if let Some((kind, width, lexeme)) = multi_character_token(&characters, index) {
            tokens.push(Token {
                kind,
                lexeme,
                line: token_line,
                column: token_column,
            });
            index += width;
            column += width;
            continue;
        }
        if let Some(kind) = punctuation(character) {
            tokens.push(Token {
                kind,
                lexeme: character.to_string(),
                line: token_line,
                column: token_column,
            });
            index += 1;
            column += 1;
            continue;
        }
        if character == '"' {
            let start = index;
            index += 1;
            column += 1;
            while index < characters.len() && characters[index] != '"' {
                if characters[index] == '\n' {
                    return Err(LexError {
                        message: "unterminated string".into(),
                        line: token_line,
                        column: token_column,
                    });
                }
                index += 1;
                column += 1;
            }
            if index == characters.len() {
                return Err(LexError {
                    message: "unterminated string".into(),
                    line: token_line,
                    column: token_column,
                });
            }
            index += 1;
            column += 1;
            tokens.push(Token {
                kind: TokenKind::String,
                lexeme: characters[start..index].iter().collect(),
                line: token_line,
                column: token_column,
            });
            continue;
        }
        if character.is_ascii_alphabetic() || character == '_' {
            let start = index;
            while index < characters.len()
                && (characters[index].is_ascii_alphanumeric() || characters[index] == '_')
            {
                index += 1;
                column += 1;
            }
            let lexeme: String = characters[start..index].iter().collect();
            tokens.push(Token {
                kind: keyword(&lexeme),
                lexeme,
                line: token_line,
                column: token_column,
            });
            continue;
        }
        if character.is_ascii_digit() {
            let start = index;
            while index < characters.len() && characters[index].is_ascii_digit() {
                index += 1;
                column += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Integer,
                lexeme: characters[start..index].iter().collect(),
                line: token_line,
                column: token_column,
            });
            continue;
        }
        return Err(LexError {
            message: format!("unexpected character '{character}'"),
            line: token_line,
            column: token_column,
        });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        lexeme: String::new(),
        line,
        column,
    });
    Ok(tokens)
}

fn keyword(value: &str) -> TokenKind {
    match value {
        "fn" => TokenKind::Fn,
        "let" => TokenKind::Let,
        "return" => TokenKind::Return,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        _ => TokenKind::Identifier,
    }
}

fn punctuation(value: char) -> Option<TokenKind> {
    Some(match value {
        '(' => TokenKind::LParen,
        ')' => TokenKind::RParen,
        '{' => TokenKind::LBrace,
        '}' => TokenKind::RBrace,
        '[' => TokenKind::LBracket,
        ']' => TokenKind::RBracket,
        ';' => TokenKind::Semicolon,
        ':' => TokenKind::Colon,
        '=' => TokenKind::Equal,
        '+' => TokenKind::Plus,
        ',' => TokenKind::Comma,
        '>' => TokenKind::Greater,
        '-' => TokenKind::Minus,
        '*' => TokenKind::Star,
        '/' => TokenKind::Slash,
        '!' => TokenKind::Bang,
        _ => return None,
    })
}

fn multi_character_token(characters: &[char], index: usize) -> Option<(TokenKind, usize, String)> {
    if index + 1 >= characters.len() {
        return None;
    }
    let pair: String = characters[index..index + 2].iter().collect();
    let kind = match pair.as_str() {
        "->" => TokenKind::Arrow,
        "==" => TokenKind::EqualEqual,
        "&&" => TokenKind::And,
        "||" => TokenKind::Or,
        _ => return None,
    };
    Some((kind, 2, pair))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_minimal_program() {
        let tokens = lex("fn main() { print(42) }").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[5].lexeme, "print");
        assert_eq!(tokens[7].kind, TokenKind::Integer);
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn reports_invalid_character_position() {
        let error = lex("fn main() { @ }").unwrap_err();
        assert_eq!(error.line, 1);
        assert_eq!(error.column, 13);
    }
}
