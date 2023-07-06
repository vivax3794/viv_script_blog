use crate::IntType;

#[derive(Debug)]
pub struct Module(pub Vec<Statement>);

#[derive(Debug)]
pub enum Statement {
    Print(Expression),
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
}

#[derive(Debug)]
pub enum Literal {
    Integer(IntType),
}
