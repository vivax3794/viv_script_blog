use anyhow::Context;
use thiserror::Error;

use crate::parsing::{
    ast,
    tokens::{Token, TokenType},
    StreamConsumer,
};

#[derive(Error, Debug)]
#[error("Parser error on line {line}, char {char}: expected one of {expected}, got {got:?}")]
pub struct ParsingError {
    pub line: usize,
    pub char: usize,
    pub got: TokenType,
    pub expected: String,
}

fn error(got: Token, expected: String) -> ParsingError {
    ParsingError {
        line: got.line,
        char: got.char,
        got: got._type,
        expected,
    }
}

pub struct Parser(StreamConsumer<Token>);

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self(StreamConsumer::new(tokens.into_iter().collect()))
    }

    fn peek(&self) -> anyhow::Result<&TokenType> {
        self.0
            .peek()
            .context("Unexpected End of Tokens")
            .map(|t| &t._type)
    }

    fn advance(&mut self) -> anyhow::Result<Token> {
        self.0.advance().context("Unexpected End of Tokens")
    }

    fn expect(&mut self, expected_token: TokenType) -> anyhow::Result<TokenType> {
        let token = self.advance()?;

        if token._type == expected_token {
            Ok(token._type)
        } else {
            Err(error(token, format!("{expected_token:?}")))?
        }
    }

    fn literal(&mut self) -> anyhow::Result<ast::Literal> {
        let token = self.advance()?;
        match token._type {
            TokenType::Integer(i) => Ok(ast::Literal::Integer(i)),
            TokenType::True => Ok(ast::Literal::Boolean(true)),
            TokenType::False => Ok(ast::Literal::Boolean(false)),
            _ => Err(error(token, "Literal".to_string()))?,
        }
    }

    fn prefix(&mut self) -> anyhow::Result<ast::Expression> {
        match self.peek()? {
            _ => Ok(ast::Expression::Literal(self.literal()?)),
        }
    }

    fn expression(&mut self) -> anyhow::Result<ast::Expression> {
        let left_side = self.prefix()?;

        match self.peek()? {
            TokenType::EqEq => {
                self.0.void();
                let right_side = self.prefix()?;
                Ok(ast::Expression::BinaryOp(
                    Box::new(left_side),
                    ast::BinaryOp::Equals,
                    Box::new(right_side),
                ))
            }
            _ => Ok(left_side),
        }
    }

    fn statement(&mut self) -> anyhow::Result<ast::Statement> {
        let token = self.advance()?;
        match token._type {
            TokenType::Print => {
                let result = ast::Statement::Print(self.expression()?);
                self.expect(TokenType::SemiColon)?;
                Ok(result)
            }
            TokenType::Assert => {
                let expression = self.expression()?;
                let message = match self.peek()? {
                    TokenType::Comma => {
                        self.0.void();
                        let should_be_string = self.advance()?;
                        match should_be_string._type {
                            TokenType::String(msg) => Some(msg),
                            _ => Err(error(should_be_string, "String".to_string()))?,
                        }
                    }
                    _ => None,
                };

                self.expect(TokenType::SemiColon)?;

                Ok(ast::Statement::Assert(expression, message))
            }
            _ => Err(error(token, "Statement".to_string()))?,
        }
    }

    fn main_function(&mut self) -> anyhow::Result<ast::ToplevelStatement> {
        self.expect(TokenType::CurlyOpen)?;

        let mut statements = Vec::new();
        while self.peek()? != &TokenType::CurlyClose {
            statements.push(self.statement()?);
        }
        self.expect(TokenType::CurlyClose)?;

        Ok(ast::ToplevelStatement::MainFunction(statements))
    }

    fn top_level_statement(&mut self) -> anyhow::Result<ast::ToplevelStatement> {
        match self.advance()?._type {
            TokenType::Dollar => self.main_function(),
            _ => Err(error(self.advance()?, "Top Level Statement".to_string()))?,
        }
    }

    pub fn module(mut self) -> anyhow::Result<ast::Module> {
        let mut statements = Vec::new();

        while self.peek()? != &TokenType::Eof {
            statements.push(self.top_level_statement()?);
        }

        Ok(ast::Module(statements))
    }
}
