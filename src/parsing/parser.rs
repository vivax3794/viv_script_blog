use anyhow::Context;
use lazy_static::lazy_static;
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

enum OperatorType {
    Binary(Vec<(TokenType, ast::BinaryOp)>),
    Prefix(TokenType, ast::PrefixOp),
    Comparison(Vec<(TokenType, ast::ComparisonOp)>),
}

lazy_static! {
    static ref OPERATORS: Vec<OperatorType> = vec![
        OperatorType::Prefix(TokenType::Bang, ast::PrefixOp::Not),
        OperatorType::Binary(vec![
            (TokenType::AndAnd, ast::BinaryOp::And),
            (TokenType::OrOr, ast::BinaryOp::Or)
        ]),
        OperatorType::Comparison(vec![
            (TokenType::EqEq, ast::ComparisonOp::Equals),
            (TokenType::BangEq, ast::ComparisonOp::NotEquals),
            (TokenType::Lt, ast::ComparisonOp::LessThan),
            (TokenType::LtEq, ast::ComparisonOp::LessThanEquals),
            (TokenType::Gt, ast::ComparisonOp::GreaterThan),
            (TokenType::GtEq, ast::ComparisonOp::GreaterThanEquals)
        ]),
        OperatorType::Binary(vec![
            (TokenType::Plus, ast::BinaryOp::Plus),
            (TokenType::Minus, ast::BinaryOp::Minus),
        ]),
        OperatorType::Binary(vec![
            (TokenType::Star, ast::BinaryOp::Multiply),
            (TokenType::Slash, ast::BinaryOp::Divide),
        ]),
        OperatorType::Prefix(TokenType::Minus, ast::PrefixOp::Negate),
    ];
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
            TokenType::Identifier(name) => Ok(ast::Literal::Variable(name)),
            _ => Err(error(token, "Literal".to_string()))?,
        }
    }

    fn group(&mut self) -> anyhow::Result<ast::Expression> {
        if self.peek()? == &TokenType::ParenOpen {
            self.0.void();
            let expression = self.expression()?;
            self.expect(TokenType::ParenClose)?;
            Ok(expression)
        } else {
            Ok(ast::Expression::Literal(self.literal()?))
        }
    }

    fn expression_precedence(&mut self, precedence: usize) -> anyhow::Result<ast::Expression> {
        let level = match OPERATORS.get(precedence) {
            Some(level) => level,
            None => return self.group(),
        };

        match level {
            OperatorType::Prefix(token, op) => {
                if self.peek()? == token {
                    self.0.void();
                    let expression = self.expression_precedence(precedence)?;
                    Ok(ast::Expression::Prefix(*op, Box::new(expression)))
                } else {
                    self.expression_precedence(precedence + 1)
                }
            }
            OperatorType::Comparison(comparisons) => {
                let left_side = self.expression_precedence(precedence + 1)?;
                let mut chains = Vec::new();

                loop {
                    let token = self.peek()?;
                    if let Some((_, op)) = comparisons.iter().find(|(t, _)| t == token) {
                        self.0.void();
                        chains.push((*op, self.expression_precedence(precedence + 1)?));
                    } else {
                        break;
                    }
                }

                if chains.is_empty() {
                    Ok(left_side)
                } else {
                    Ok(ast::Expression::Comparison(Box::new(left_side), chains))
                }
            }
            OperatorType::Binary(mappings) => {
                let mut left_side = self.expression_precedence(precedence + 1)?;
                loop {
                    let token = self.peek()?;
                    if let Some((_, op)) = mappings.iter().find(|(t, _)| t == token) {
                        self.0.void();
                        let right_side = self.expression_precedence(precedence + 1)?;
                        left_side = ast::Expression::BinaryOp(
                            Box::new(left_side),
                            *op,
                            Box::new(right_side),
                        );
                    } else {
                        break;
                    }
                }

                Ok(left_side)
            }
        }
    }

    fn expression(&mut self) -> anyhow::Result<ast::Expression> {
        self.expression_precedence(0)
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
            TokenType::Let | TokenType::Set => {
                let identifier = self.advance()?;
                match identifier._type {
                    TokenType::Identifier(name) => {
                        self.expect(TokenType::Eq)?;
                        let expression = self.expression()?;
                        self.expect(TokenType::SemiColon)?;
                        Ok(match token._type {
                            TokenType::Let => ast::Statement::Declaration(name, expression),
                            TokenType::Set => ast::Statement::Assignment(name, expression),
                            _ => unreachable!(),
                        })
                    }
                    _ => Err(error(identifier, "Identifier".to_string()))?,
                }
            }
            _ => Err(error(token, "Statement".to_string()))?,
        }
    }

    // let x = 123; -> Declaration
    // set x = 12313; -> Assignment
    // x(); -> Expression

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
