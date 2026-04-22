#[derive(Debug, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
    pub functions: Vec<FunctionDefinition>,
}

#[derive(Debug, Clone)]
pub enum TopLevelItem {
    Declaration(Vec<Declaration>),
    Function(FunctionDefinition),
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub parameters: Vec<Parameter>,
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    #[allow(unused)]
    pub parameter_type: BasicType,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct BasicType {}

#[derive(Debug, Clone)]
pub enum Declaration {
    Constant {
        constant_type: BasicType,
        name: String,
        value: Box<Expression>,
    },
    Variable {
        variable_type: BasicType,
        name: String,
        initial_value: Option<Box<Expression>>,
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Declaration(Declaration),
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
    Break,
    Continue,
    Return(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Variable(String),
    Number(i32),
    Unary(UnaryOperator, Box<Expression>),
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
    Call {
        function_name: String,
        arguments: Vec<Expression>,
    },
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
