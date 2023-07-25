use crate::{parsing::StreamConsumer, IntType};
use anyhow::Context;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Integer(IntType),
    Identifier(String),
    String(String),
    Print,
    SemiColon,
    Dollar,
    CurlyOpen,
    CurlyClose,
    ParenOpen,
    ParenClose,
    Comma,
    True,
    False,
    Assert,
    Eq,
    Bang,
    EqEq,
    BangEq,
    Lt,
    Gt,
    GtEq,
    LtEq,
    Minus,
    Plus,
    Star,
    Slash,
    AndAnd,
    OrOr,
    Or,
    And,
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
            "true" => TokenType::True,
            "false" => TokenType::False,
            "assert" => TokenType::Assert,
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

    fn consume_string(&mut self) -> Result<Token, TokenizerError> {
        let mut string = String::new();
        self.void();
        loop {
            match self.code.peek() {
                Ok('"') => {
                    self.void();
                    break;
                }
                Ok('\n') => self.error("Unexpected newline in string".to_string())?,
                Ok(c) => {
                    string.push(*c);
                    self.void();
                }
                Err(_) => self.error("Unexpected end of file".to_string())?,
            }
        }
        Ok(self.token(TokenType::String(string)))
    }

    fn consume_double_symbol(
        &mut self,
        next_char: char,
        single_token: TokenType,
        double_token: TokenType,
    ) -> Token {
        self.void();
        if let Ok(c) = self.code.peek() {
            if c == &next_char {
                self.void();
                self.token(double_token)
            } else {
                self.token(single_token)
            }
        } else {
            self.token(single_token)
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, TokenizerError> {
        let mut tokens = Vec::new();
        while let Ok(&c) = self.code.peek() {
            match c {
                '#' => self.consume_comment(),
                '"' => tokens.push(self.consume_string()?),
                c if c.is_ascii_digit() => tokens.push(self.consume_number()),
                c if c.is_ascii_alphabetic() => tokens.push(self.consume_identifier()),
                c if c.is_ascii_whitespace() => self.consume_whitespace(),
                '=' => tokens.push(self.consume_double_symbol('=', TokenType::Eq, TokenType::EqEq)),
                '!' => {
                    tokens.push(self.consume_double_symbol('=', TokenType::Bang, TokenType::BangEq))
                }
                '<' => tokens.push(self.consume_double_symbol('=', TokenType::Lt, TokenType::LtEq)),
                '>' => tokens.push(self.consume_double_symbol('=', TokenType::Gt, TokenType::GtEq)),
                '&' => {
                    tokens.push(self.consume_double_symbol('&', TokenType::And, TokenType::AndAnd))
                }
                '|' => tokens.push(self.consume_double_symbol('|', TokenType::Or, TokenType::OrOr)),
                _ => {
                    self.void();

                    match c {
                        ';' => tokens.push(self.token(TokenType::SemiColon)),
                        '$' => tokens.push(self.token(TokenType::Dollar)),
                        ',' => tokens.push(self.token(TokenType::Comma)),
                        '{' => tokens.push(self.token(TokenType::CurlyOpen)),
                        '}' => tokens.push(self.token(TokenType::CurlyClose)),
                        '(' => tokens.push(self.token(TokenType::ParenOpen)),
                        ')' => tokens.push(self.token(TokenType::ParenClose)),
                        '-' => tokens.push(self.token(TokenType::Minus)),
                        '+' => tokens.push(self.token(TokenType::Plus)),
                        '*' => tokens.push(self.token(TokenType::Star)),
                        '/' => tokens.push(self.token(TokenType::Slash)),
                        _ => self.error(format!("Unexpected character: {}", c))?,
                    }
                }
            }
        }
        tokens.push(self.token(TokenType::Eof));
        Ok(tokens)
    }
}
