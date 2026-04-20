#[derive(Debug, Clone)]
pub struct Program {
    pub function_definition: FunctionDefinition,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    #[allow(unused)]
    pub function_type: FunctionType,
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct FunctionType {}

#[derive(Debug, Clone)]
pub struct BasicType {}

#[derive(Debug, Clone)]
pub enum Statement {
    Constant {
        constant_type: BasicType,
        name: String,
        value: Box<Expression>,
    },
    Variable {
        variable_type: BasicType,
        name: String,
        value: Option<Box<Expression>>,
    },
    Assign {
        target: Box<Expression>,
        value: Box<Expression>,
    },
    Expression(Box<Expression>),
    Block(Vec<Statement>),
    If {
        condition: Box<Expression>,
        then: Box<Statement>,
        otherwise: Option<Box<Statement>>,
    },
    While {
        condition: Box<Expression>,
        body: Box<Statement>,
    },
    Return(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Variable(String),
    Number(i32),
    Unary(UnaryOperator, Box<Expression>),
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    Plus,
    Minus,
    BooleanNot,
    BitNot,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    BooleanAnd,
    BooleanOr,
}
