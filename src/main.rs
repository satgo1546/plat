#[derive(Debug)]
struct Variable(usize);

#[derive(Debug)]
enum Expression<V> {
    Variable(V),
    Integer(i32),
    Function(V, Box<Self>),
    Call(Box<Self>, Box<Self>),
}

impl<V> Expression<V> {
    fn function(parameter: V, body: Self) -> Self {
        Self::Function(parameter, Box::new(body))
    }
    fn call(callee: Self, argument: Self) -> Self {
        Self::Call(Box::new(callee), Box::new(argument))
    }
}

#[derive(Debug)]
struct TypeVariable(u32);

#[derive(Debug)]
enum Type {
    TypeVariable(TypeVariable),
    Integer,
    Function(Box<Self>, Box<Self>),
}

impl Type {
    fn function(parameter: Self, body: Self) -> Self {
        Self::Function(Box::new(parameter), Box::new(body))
    }
}

#[derive(Debug)]
struct VariableAndType(Variable, Type);

fn main() {
    let expr = Expression::function(
        Variable(1),
        Expression::call(
            Expression::function(Variable(0), Expression::Variable(Variable(0))),
            Expression::Integer(114514),
        ),
    );
    let ty = Type::function(Type::TypeVariable(TypeVariable(0)), Type::Integer);
    println!("{:?}: {:?}", expr, ty);
}
