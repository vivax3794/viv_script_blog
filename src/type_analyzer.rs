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

fn resolve_prefix(
    expression: &Box<ast::Expression>,
    op: &ast::PrefixOp,
) -> anyhow::Result<TypedExpression> {
    let expression = resolve_expression(expression)?;

    match op {
        ast::PrefixOp::Negate => {
            let expression = expression.is_int()?;

            Ok(TypedExpression::Int(ir::IntExpression::Negate(Box::new(
                expression,
            ))))
        }
        ast::PrefixOp::Not => {
            let expression = expression.is_boolean()?;

            Ok(TypedExpression::Boolean(ir::BooleanExpression::Not(
                Box::new(expression),
            )))
        }
    }
}

fn resolve_binary(
    left: &Box<ast::Expression>,
    op: &ast::BinaryOp,
    right: &Box<ast::Expression>,
) -> anyhow::Result<TypedExpression> {
    let left = resolve_expression(left)?;
    let right = resolve_expression(right)?;

    match left {
        TypedExpression::Int(left) => {
            let right = right.is_int()?;
            let op = match op {
                ast::BinaryOp::Plus => ir::IntBinaryOp::Plus,
                ast::BinaryOp::Minus => ir::IntBinaryOp::Minus,
                ast::BinaryOp::Multiply => ir::IntBinaryOp::Multiply,
                ast::BinaryOp::Divide => ir::IntBinaryOp::Divide,
                _ => Err(TypeError("Operator not supported for int".to_string()))?,
            };
            Ok(TypedExpression::Int(ir::IntExpression::BinaryOperation(
                Box::new(left),
                op,
                Box::new(right),
            )))
        }
        TypedExpression::Boolean(left) => {
            let right = right.is_boolean()?;
            let op = match op {
                ast::BinaryOp::And => ir::BooleanOperator::And,
                ast::BinaryOp::Or => ir::BooleanOperator::Or,
                _ => Err(TypeError("Operator not supported for boolean".to_string()))?,
            };
            Ok(TypedExpression::Boolean(ir::BooleanExpression::Operator(
                Box::new(left),
                op,
                Box::new(right),
            )))
        }
        _ => Err(TypeError("Operator not supported for type".to_string()))?,
    }
}

fn resolve_comparison(
    left_side: &Box<ast::Expression>,
    chains: &Vec<(ast::ComparisonOp, ast::Expression)>,
) -> anyhow::Result<TypedExpression> {
    let left_side = resolve_expression(left_side)?;
    let left_side = left_side.is_int()?;
    let chains = chains
        .iter()
        .map(|(op, expression)| {
            let expression = resolve_expression(expression)?;
            let expression = expression.is_int()?;

            let op = match op {
                ast::ComparisonOp::Equals => ir::IntComparisonOp::Equal,
                ast::ComparisonOp::NotEquals => ir::IntComparisonOp::NotEquals,
                ast::ComparisonOp::LessThan => ir::IntComparisonOp::LessThan,
                ast::ComparisonOp::LessThanEquals => ir::IntComparisonOp::LessThanEquals,
                ast::ComparisonOp::GreaterThan => ir::IntComparisonOp::GreaterThan,
                ast::ComparisonOp::GreaterThanEquals => ir::IntComparisonOp::GreaterThanEquals,
            };

            Ok((op, expression))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(TypedExpression::Boolean(ir::BooleanExpression::Comparison(
        ir::ComparisonExpression::IntComparison(Box::new(left_side), chains),
    )))
}

fn resolve_expression(expression: &ast::Expression) -> anyhow::Result<TypedExpression> {
    match expression {
        ast::Expression::Literal(literal) => resolve_literal(literal),
        ast::Expression::Prefix(op, expression) => resolve_prefix(expression, op),
        ast::Expression::BinaryOp(left, op, right) => resolve_binary(left, op, right),
        ast::Expression::Comparison(left_side, chains) => resolve_comparison(left_side, chains),
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
