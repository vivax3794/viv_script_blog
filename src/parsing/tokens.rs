use crate::{parsing::StreamConsumer, IntType};
use anyhow::Context;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Integer(IntType),
    Identifier(String),
    Print,
    SemiColon,
    Eof,
}

pub struct Token {
    pub _type: TokenType,
    pub line: usize,
    pub char: usize,
}

#[derive(Error, Debug)]
#[error("Tokenizer error on line {line}, char {char}: {message}")]
pub struct TokenizerError {
    pub line: usize,
    pub char: usize,
    pub message: String,
}

pub struct Tokenizer {
    code: StreamConsumer<char>,
    line: usize,
    char: usize,
}

impl Tokenizer {
    pub fn new(code: &str) -> Self {
        Self {
            code: StreamConsumer::new(code.chars().collect()),
            line: 1,
            char: 1,
        }
    }

    fn error(&self, msg: String) -> Result<(), TokenizerError> {
        Err(TokenizerError {
            line: self.line,
            char: self.char,
            message: msg,
        })
    }

    fn token(&self, _type: TokenType) -> Token {
        Token {
            _type,
            line: self.line,
            char: self.char,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.code.advance().ok()?;
        if c == '\n' {
            self.line += 1;
            self.char = 1;
        } else {
            self.char += 1;
        }
        Some(c)
    }

    fn void(&mut self) {
        self.advance();
    }

    fn consume_number(&mut self) -> Token {
        let mut number = String::new();
        while let Ok(c) = self.code.peek() {
            if c.is_ascii_digit() {
                number.push(*c);
                self.void();
            } else {
                break;
            }
        }
        self.token(TokenType::Integer(number.parse().unwrap()))
    }

    fn consume_identifier(&mut self) -> Token {
        let mut identifier = String::new();
        while let Ok(c) = self.code.peek() {
            if c.is_ascii_alphanumeric() {
                identifier.push(*c);
                self.void();
            } else {
                break;
            }
        }

        self.token(match identifier.as_str() {
            "print" => TokenType::Print,
            _ => TokenType::Identifier(identifier),
        })
    }

    fn consume_whitespace(&mut self) {
        while let Ok(c) = self.code.peek() {
            if c.is_ascii_whitespace() {
                self.void();
            } else {
                break;
            }
        }
    }

    fn consume_comment(&mut self) {
        while let Ok(c) = self.code.peek() {
            if c == &'\n' {
                self.void();
                break;
            } else {
                self.void();
            }
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, TokenizerError> {
        let mut tokens = Vec::new();
        while let Ok(c) = self.code.peek() {
            match c {
                c if c.is_ascii_digit() => tokens.push(self.consume_number()),
                c if c.is_ascii_alphabetic() => tokens.push(self.consume_identifier()),
                ';' => {
                    tokens.push(self.token(TokenType::SemiColon));
                    self.void();
                }
                c if c.is_ascii_whitespace() => self.consume_whitespace(),
                '#' => self.consume_comment(),
                _ => {
                    self.error(format!("Unexpected character: {}", c))?;
                }
            }
        }
        tokens.push(self.token(TokenType::Eof));
        Ok(tokens)
    }
}
