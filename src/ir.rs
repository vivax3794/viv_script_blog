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
    Negate(Box<IntExpression>),
    BinaryOperation(Box<IntExpression>, IntBinaryOp, Box<IntExpression>),
}

#[derive(Debug)]
pub enum IntBinaryOp {
    Plus,
    Minus,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum BooleanExpression {
    Literal(bool),
    Not(Box<BooleanExpression>),
    Comparison(ComparisonExpression),
    Operator(
        Box<BooleanExpression>,
        BooleanOperator,
        Box<BooleanExpression>,
    ),
}

#[derive(Debug)]
pub enum BooleanOperator {
    And,
    Or,
}

#[derive(Debug)]
pub enum ComparisonExpression {
    IntComparison(Box<IntExpression>, Vec<(IntComparisonOp, IntExpression)>),
}

#[derive(Debug)]
pub enum IntComparisonOp {
    Equal,
    NotEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
}
