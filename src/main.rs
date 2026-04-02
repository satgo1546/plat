use making_a_language::{Expression, Variable, infer};

fn main() {
    let expr = Expression::function(
        Variable(1),
        Expression::call(
            Expression::function(Variable(0), Expression::Variable(Variable(0))),
            Expression::Integer(114514),
        ),
    );
    print!("{:?}: ", expr);
    let ty = infer(expr);
    println!("{:?}", ty);
}
