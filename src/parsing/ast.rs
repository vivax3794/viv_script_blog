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
}

#[derive(Debug)]
pub enum BinaryOp {
    Equals,
}

#[derive(Debug)]
pub enum Literal {
    Integer(IntType),
    Boolean(bool),
}
