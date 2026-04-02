#[derive(Debug)]
pub struct Variable(pub usize);

#[derive(Debug)]
pub enum Expression<V> {
    Variable(V),
    Integer(i32),
    Function(V, Box<Self>),
    Call(Box<Self>, Box<Self>),
}

impl<V> Expression<V> {
    pub fn function(parameter: V, body: Self) -> Self {
        Self::Function(parameter, Box::new(body))
    }
    pub fn call(callee: Self, argument: Self) -> Self {
        Self::Call(Box::new(callee), Box::new(argument))
    }
}

#[derive(Debug)]
pub struct TypeVariable(pub u32);

#[derive(Debug)]
pub enum Type {
    TypeVariable(TypeVariable),
    Integer,
    Function(Box<Self>, Box<Self>),
}

impl Type {
    pub fn function(parameter: Self, return_type: Self) -> Self {
        Self::Function(Box::new(parameter), Box::new(return_type))
    }
}

#[derive(Debug)]
pub struct VariableAndType(pub Variable, pub Type);
