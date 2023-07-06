#[derive(Debug)]
pub struct Module(pub Vec<Statement>);

#[derive(Debug)]
pub enum Statement {
    Print(PrintStatement),
}

#[derive(Debug)]
pub enum PrintStatement {
    Int(IntExpression),
}

#[derive(Debug)]
pub enum IntExpression {
    Literal(i32),
}
