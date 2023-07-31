#[derive(Debug)]
pub struct Module(pub Vec<ToplevelStatement>);

#[derive(Debug)]
pub enum ToplevelStatement {
    Function {
        name: String,
        body: Vec<Statement>,
        locals: Vec<(VariableIdentifier, VarType)>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarType {
    Int,
    Boolean,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct VariableIdentifier(pub usize);

#[derive(Debug)]
pub enum Statement {
    Print(PrintStatement),
    Assert(BooleanExpression, Option<String>),
    Assignment(VariableIdentifier, AssignmentStatement),
}

#[derive(Debug)]
pub enum AssignmentStatement {
    Int(IntExpression),
    Boolean(BooleanExpression),
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
    Var(VariableIdentifier),
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
        VariableIdentifier,
        Box<BooleanExpression>,
        BooleanOperator,
        Box<BooleanExpression>,
    ),
    Var(VariableIdentifier),
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
