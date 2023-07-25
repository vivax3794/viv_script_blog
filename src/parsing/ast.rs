use crate::IntType;

#[derive(Debug)]
pub struct Module(pub Vec<ToplevelStatement>);

#[derive(Debug)]
pub enum ToplevelStatement {
    MainFunction(Vec<Statement>),
}

#[derive(Debug)]
pub enum Statement {
    Print(Expression),
    Assert(Expression, Option<String>),
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    BinaryOp(Box<Expression>, BinaryOp, Box<Expression>),
    Prefix(PrefixOp, Box<Expression>),
    Comparison(Box<Expression>, Vec<(ComparisonOp, Expression)>),
}

#[derive(Debug, Copy, Clone)]
pub enum PrefixOp {
    Negate,
    Not,
}

#[derive(Debug, Copy, Clone)]
pub enum BinaryOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    And,
    Or,
}

#[derive(Debug, Copy, Clone)]
pub enum ComparisonOp {
    Equals,
    NotEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
}

#[derive(Debug)]
pub enum Literal {
    Integer(IntType),
    Boolean(bool),
}
