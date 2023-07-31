use std::{collections::HashMap, process::id};

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

    fn to_var_type(&self) -> ir::VarType {
        match self {
            TypedExpression::Int(_) => ir::VarType::Int,
            TypedExpression::Boolean(_) => ir::VarType::Boolean,
        }
    }
}

struct VarInfo {
    identifier: ir::VariableIdentifier,
    var_type: ir::VarType,
}

struct VarScope {
    parent: Option<Box<VarScope>>,
    variables: HashMap<String, VarInfo>,
}

struct FunctionMetadata {
    locals: Vec<(ir::VariableIdentifier, ir::VarType)>,
}

#[derive(Debug, Error)]
#[error("Type error: {0}")]
struct TypeError(String);

pub struct Analyzer {
    scopes: Vec<VarScope>,
    function_metadata: Option<FunctionMetadata>,
    current_identifier: usize,
}

impl Analyzer {
    fn get_free_identifier(&mut self) -> ir::VariableIdentifier {
        self.current_identifier += 1;
        ir::VariableIdentifier(self.current_identifier)
    }

    fn resolve_literal(&mut self, literal: &ast::Literal) -> anyhow::Result<TypedExpression> {
        match literal {
            ast::Literal::Integer(int) => {
                Ok(TypedExpression::Int(ir::IntExpression::Literal(*int)))
            }
            ast::Literal::Boolean(boolean) => Ok(TypedExpression::Boolean(
                ir::BooleanExpression::Literal(*boolean),
            )),
            ast::Literal::Variable(name) => {
                let var_info = self
                    .scopes
                    .last()
                    .unwrap()
                    .variables
                    .get(name)
                    .ok_or(TypeError(format!("variable {name} not found")))?;

                Ok(match var_info.var_type {
                    ir::VarType::Int => {
                        TypedExpression::Int(ir::IntExpression::Var(var_info.identifier))
                    }
                    ir::VarType::Boolean => {
                        TypedExpression::Boolean(ir::BooleanExpression::Var(var_info.identifier))
                    }
                })
            }
        }
    }

    fn resolve_prefix(
        &mut self,
        expression: &Box<ast::Expression>,
        op: &ast::PrefixOp,
    ) -> anyhow::Result<TypedExpression> {
        let expression = self.resolve_expression(expression)?;

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
        &mut self,
        left: &Box<ast::Expression>,
        op: &ast::BinaryOp,
        right: &Box<ast::Expression>,
    ) -> anyhow::Result<TypedExpression> {
        let left = self.resolve_expression(left)?;
        let right = self.resolve_expression(right)?;

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

                let result_identifier = self.get_free_identifier();
                self.function_metadata
                    .as_mut()
                    .unwrap()
                    .locals
                    .push((result_identifier, ir::VarType::Boolean));

                Ok(TypedExpression::Boolean(ir::BooleanExpression::Operator(
                    result_identifier,
                    Box::new(left),
                    op,
                    Box::new(right),
                )))
            }
            _ => Err(TypeError("Operator not supported for type".to_string()))?,
        }
    }

    fn resolve_comparison(
        &mut self,
        left_side: &Box<ast::Expression>,
        chains: &Vec<(ast::ComparisonOp, ast::Expression)>,
    ) -> anyhow::Result<TypedExpression> {
        let left_side = self.resolve_expression(left_side)?;
        let left_side = left_side.is_int()?;
        let chains = chains
            .iter()
            .map(|(op, expression)| {
                let expression = self.resolve_expression(expression)?;
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

    fn resolve_expression(
        &mut self,
        expression: &ast::Expression,
    ) -> anyhow::Result<TypedExpression> {
        match expression {
            ast::Expression::Literal(literal) => self.resolve_literal(literal),
            ast::Expression::Prefix(op, expression) => self.resolve_prefix(expression, op),
            ast::Expression::BinaryOp(left, op, right) => self.resolve_binary(left, op, right),
            ast::Expression::Comparison(left_side, chains) => {
                self.resolve_comparison(left_side, chains)
            }
        }
    }

    fn resolve_print_statement(
        &mut self,
        expression: &ast::Expression,
    ) -> anyhow::Result<ir::PrintStatement> {
        let typed_expression = self.resolve_expression(expression)?;

        match typed_expression {
            TypedExpression::Int(int_expression) => Ok(ir::PrintStatement::Int(int_expression)),
            TypedExpression::Boolean(boolean_expression) => {
                Ok(ir::PrintStatement::Boolean(boolean_expression))
            }
        }
    }

    fn resolve_statement(&mut self, statement: &ast::Statement) -> anyhow::Result<ir::Statement> {
        match statement {
            ast::Statement::Print(expression) => {
                let print_statement = self.resolve_print_statement(expression)?;

                Ok(ir::Statement::Print(print_statement))
            }
            ast::Statement::Assert(expression, message) => {
                let expression = self.resolve_expression(expression)?;
                let expression = expression.is_boolean()?;

                Ok(ir::Statement::Assert(expression, message.clone()))
            }
            ast::Statement::Declaration(name, expression) => {
                let identifier = self.get_free_identifier();
                let typed_expression = self.resolve_expression(expression)?;

                let var_type = typed_expression.to_var_type();
                self.function_metadata
                    .as_mut()
                    .unwrap()
                    .locals
                    .push((identifier, var_type));

                let assignment = match typed_expression {
                    TypedExpression::Int(int_expression) => {
                        ir::AssignmentStatement::Int(int_expression)
                    }
                    TypedExpression::Boolean(boolean_expression) => {
                        ir::AssignmentStatement::Boolean(boolean_expression)
                    }
                };

                self.scopes.last_mut().unwrap().variables.insert(
                    name.clone(),
                    VarInfo {
                        identifier,
                        var_type,
                    },
                );

                Ok(ir::Statement::Assignment(identifier, assignment))
            }
            ast::Statement::Assignment(name, expression) => {
                let typed_expression = self.resolve_expression(expression)?;
                let var_info = self
                    .scopes
                    .last()
                    .unwrap()
                    .variables
                    .get(name)
                    .ok_or(TypeError(format!("variable {name} not found")))?;

                let var_type = typed_expression.to_var_type();

                if var_type != var_info.var_type {
                    Err(TypeError(format!(
                        "expression is of type {var_type:?}, but variable {name} is not."
                    )))?;
                }

                let assignment = match typed_expression {
                    TypedExpression::Int(int_expression) => {
                        ir::AssignmentStatement::Int(int_expression)
                    }
                    TypedExpression::Boolean(boolean_expression) => {
                        ir::AssignmentStatement::Boolean(boolean_expression)
                    }
                };
                Ok(ir::Statement::Assignment(var_info.identifier, assignment))
            }
        }
    }

    pub fn resolve_top_level_statement(
        &mut self,
        statement: &ast::ToplevelStatement,
    ) -> anyhow::Result<ir::ToplevelStatement> {
        match statement {
            ast::ToplevelStatement::MainFunction(statements) => {
                self.function_metadata = Some(FunctionMetadata { locals: Vec::new() });
                self.scopes.push(VarScope {
                    parent: None,
                    variables: HashMap::new(),
                });

                let mut ir_statements = Vec::new();

                for statement in statements {
                    let ir_statement = self.resolve_statement(statement)?;

                    ir_statements.push(ir_statement);
                }

                Ok(ir::ToplevelStatement::Function {
                    name: String::from("main"),
                    body: ir_statements,
                    locals: self.function_metadata.as_ref().unwrap().locals.clone(),
                })
            }
        }
    }

    pub fn resolve_module(&mut self, module: &ast::Module) -> anyhow::Result<ir::Module> {
        let mut ir_statements = Vec::new();

        for statement in &module.0 {
            let ir_statement = self.resolve_top_level_statement(statement)?;

            ir_statements.push(ir_statement);
        }

        Ok(ir::Module(ir_statements))
    }

    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            function_metadata: None,
            current_identifier: 0,
        }
    }
}
