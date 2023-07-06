use std::collections::VecDeque;
use thiserror::Error;

use crate::CompilerOptions;

pub mod ast;
mod parser;
mod tokens;

#[derive(Error, Debug)]
#[error("Stream ran out of elements")]
struct StreamerError;

struct StreamConsumer<T>(VecDeque<T>);

impl<T> StreamConsumer<T> {
    fn new(stream: VecDeque<T>) -> Self {
        Self(stream)
    }

    fn advance(&mut self) -> Result<T, StreamerError> {
        self.0.pop_front().ok_or(StreamerError)
    }

    fn void(&mut self) {
        self.0.pop_front();
    }

    fn peek(&self) -> Result<&T, StreamerError> {
        self.0.front().ok_or(StreamerError)
    }
}

pub fn parse(code: &str, compiler_options: &CompilerOptions) -> anyhow::Result<ast::Module> {
    let tokenizer = tokens::Tokenizer::new(code);
    let tokens = tokenizer.tokenize()?;

    if compiler_options.output_tokens {
        // we dont care about position info in this output
        let token_types = tokens
            .iter()
            .map(|t| format!("{:?}", t._type))
            // This is inefficient! itertools has a more efficient join method, but I dont want to add a dependency just for that
            // You however can :D
            .collect::<Vec<String>>()
            .join(", ");

        println!("TOKENS: {token_types}");
    }

    let parser = parser::Parser::new(tokens);
    let ast = parser.module()?;

    if compiler_options.output_ast {
        println!("AST: {ast:#?}");
    }

    Ok(ast)
}
