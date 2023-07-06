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
            _ => Err(error(token, "Literal".to_string()))?,
        }
    }

    fn expression(&mut self) -> anyhow::Result<ast::Expression> {
        match self.peek()? {
            TokenType::Integer(_) => Ok(ast::Expression::Literal(self.literal()?)),
            _ => Err(error(self.advance()?, "Expression".to_string()))?,
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
            _ => Err(error(token, "Statement".to_string()))?,
        }
    }

    pub fn module(mut self) -> anyhow::Result<ast::Module> {
        let mut statements = Vec::new();

        while self.peek()? != &TokenType::Eof {
            statements.push(self.statement()?);
        }

        Ok(ast::Module(statements))
    }
}
