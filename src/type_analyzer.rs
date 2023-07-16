use crate::{ir, parsing::ast};
use thiserror::Error;

enum TypedExpression {
    Int(ir::IntExpression),
    Boolean(ir::BooleanExpression),
}

impl TypedExpression {
    fn is_int(self) -> anyhow::Result<ir::IntExpression> {
        match self {
            TypedExpression::Int(exp) => Ok(exp),
            _ => Err(TypeError("Expected int".to_string()))?,
        }
    }

    fn is_boolean(self) -> anyhow::Result<ir::BooleanExpression> {
        match self {
            TypedExpression::Boolean(exp) => Ok(exp),
            _ => Err(TypeError("Expected bool".to_string()))?,
        }
    }
}

#[derive(Debug, Error)]
#[error("Type error: {0}")]
struct TypeError(String);

fn resolve_literal(literal: &ast::Literal) -> anyhow::Result<TypedExpression> {
    match literal {
        ast::Literal::Integer(int) => Ok(TypedExpression::Int(ir::IntExpression::Literal(*int))),
        ast::Literal::Boolean(boolean) => Ok(TypedExpression::Boolean(
            ir::BooleanExpression::Literal(*boolean),
        )),
    }
}

fn resolve_expression(expression: &ast::Expression) -> anyhow::Result<TypedExpression> {
    match expression {
        ast::Expression::Literal(literal) => resolve_literal(literal),
        ast::Expression::BinaryOp(left, op, right) => {
            let left = resolve_expression(left)?;
            let right = resolve_expression(right)?;

            match op {
                ast::BinaryOp::Equals => {
                    let left = left.is_int()?;
                    let right = right.is_int()?;

                    Ok(TypedExpression::Boolean(ir::BooleanExpression::Comparison(
                        ir::ComparisonExpression::IntComparison(
                            Box::new(left),
                            ir::IntComparisonOp::Equal,
                            Box::new(right),
                        ),
                    )))
                }
            }
        }
    }
}

fn resolve_print_statement(expression: &ast::Expression) -> anyhow::Result<ir::PrintStatement> {
    let typed_expression = resolve_expression(expression)?;

    match typed_expression {
        TypedExpression::Int(int_expression) => Ok(ir::PrintStatement::Int(int_expression)),
        TypedExpression::Boolean(boolean_expression) => {
            Ok(ir::PrintStatement::Boolean(boolean_expression))
        }
    }
}

fn resolve_statement(statement: &ast::Statement) -> anyhow::Result<ir::Statement> {
    match statement {
        ast::Statement::Print(expression) => {
            let print_statement = resolve_print_statement(expression)?;

            Ok(ir::Statement::Print(print_statement))
        }
        ast::Statement::Assert(expression, message) => {
            let expression = resolve_expression(expression)?;
            let expression = expression.is_boolean()?;

            Ok(ir::Statement::Assert(expression, message.clone()))
        }
    }
}

pub fn resolve_top_level_statement(
    statement: &ast::ToplevelStatement,
) -> anyhow::Result<ir::ToplevelStatement> {
    match statement {
        ast::ToplevelStatement::MainFunction(statements) => {
            let mut ir_statements = Vec::new();

            for statement in statements {
                let ir_statement = resolve_statement(statement)?;

                ir_statements.push(ir_statement);
            }

            Ok(ir::ToplevelStatement::MainFunction(ir_statements))
        }
    }
}

pub fn resolve_module(module: &ast::Module) -> anyhow::Result<ir::Module> {
    let mut ir_statements = Vec::new();

    for statement in &module.0 {
        let ir_statement = resolve_top_level_statement(statement)?;

        ir_statements.push(ir_statement);
    }

    Ok(ir::Module(ir_statements))
}
