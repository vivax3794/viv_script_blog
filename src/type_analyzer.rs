use std::any;

use crate::{ir, parsing::ast};
use thiserror::Error;

enum TypedExpression {
    Int(ir::IntExpression),
}

impl TypedExpression {
    fn is_int(&self) -> bool {
        match self {
            TypedExpression::Int(_) => true,
        }
    }
}

#[derive(Debug, Error)]
#[error("Type error: {0}")]
struct TypeError(String);

fn resolve_literal(literal: &ast::Literal) -> anyhow::Result<TypedExpression> {
    match literal {
        ast::Literal::Integer(int) => Ok(TypedExpression::Int(ir::IntExpression::Literal(*int))),
    }
}

fn resolve_expression(expression: &ast::Expression) -> anyhow::Result<TypedExpression> {
    match expression {
        ast::Expression::Literal(literal) => resolve_literal(literal),
    }
}

fn resolve_print_statement(expression: &ast::Expression) -> anyhow::Result<ir::PrintStatement> {
    let typed_expression = resolve_expression(expression)?;

    match typed_expression {
        TypedExpression::Int(int_expression) => Ok(ir::PrintStatement::Int(int_expression)),
    }
}

fn resolve_statement(statement: &ast::Statement) -> anyhow::Result<ir::Statement> {
    match statement {
        ast::Statement::Print(expression) => {
            let print_statement = resolve_print_statement(expression)?;

            Ok(ir::Statement::Print(print_statement))
        }
    }
}

pub fn resolve_module(module: &ast::Module) -> anyhow::Result<ir::Module> {
    let mut ir_statements = Vec::new();

    for statement in &module.0 {
        let ir_statement = resolve_statement(statement)?;

        ir_statements.push(ir_statement);
    }

    Ok(ir::Module(ir_statements))
}
