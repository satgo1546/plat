#[derive(Debug, Clone)]
pub struct Program {
    pub function_definition: FunctionDefinition,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    #[allow(unused)]
    pub function_type: FunctionType,
    pub name: String,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct FunctionType {}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Number(i32),
    Unary(UnaryOperator, Box<Expression>),
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    Plus,
    Minus,
    BooleanNot,
    BitNot,
}
