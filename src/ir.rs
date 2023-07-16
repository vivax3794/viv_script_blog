#[derive(Debug)]
pub struct Module(pub Vec<ToplevelStatement>);

#[derive(Debug)]
pub enum ToplevelStatement {
    MainFunction(Vec<Statement>),
}

#[derive(Debug)]
pub enum Statement {
    Print(PrintStatement),
    Assert(BooleanExpression, Option<String>),
}

#[derive(Debug)]
pub enum PrintStatement {
    Int(IntExpression),
    Boolean(BooleanExpression),
}

#[derive(Debug)]
pub enum IntExpression {
    Literal(i32),
}

#[derive(Debug)]
pub enum BooleanExpression {
    Literal(bool),
    Comparison(ComparisonExpression),
}

#[derive(Debug)]
pub enum ComparisonExpression {
    IntComparison(Box<IntExpression>, IntComparisonOp, Box<IntExpression>),
}

#[derive(Debug)]
pub enum IntComparisonOp {
    Equal,
}
