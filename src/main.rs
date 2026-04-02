use making_a_language::{Expression, Type, TypeVariable, Variable};

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
