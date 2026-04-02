use rpds::HashTrieMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVariable(pub Uuid);

#[derive(Debug, Clone)]
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
enum Constraint {
    Equal((), Type, Type),
}

struct Inferrer {
    // unification_table: InPlaceUnificationTable<TypeVariable>,
}

impl Inferrer {
    fn new_type_variable(&mut self) -> TypeVariable {
        TypeVariable(Uuid::new_v4())
    }
    fn infer(
        &mut self,
        scope: HashTrieMap<Variable, Type>,
        expression: Expression<Variable>,
    ) -> (Vec<Constraint>, Expression<(Variable, Type)>, Type) {
        match expression {
            Expression::Variable(variable) => {
                let type_ = &scope[&variable];
                (
                    vec![],
                    Expression::Variable((variable, type_.clone())),
                    type_.clone(),
                )
            }
            Expression::Integer(value) => (vec![], Expression::Integer(value), Type::Integer),
            Expression::Function(parameter, body) => {
                let parameter_type = Type::TypeVariable(self.new_type_variable());
                let scope = scope.insert(parameter, parameter_type.clone());
                let (constraints, body, body_type) = self.infer(scope, *body);
                (
                    constraints,
                    Expression::function((parameter, parameter_type.clone()), body),
                    Type::function(parameter_type, body_type),
                )
            }
            Expression::Call(callee, argument) => {
                let (mut constraints0, argument, argument_type) =
                    self.infer(scope.clone(), *argument);
                let return_type = Type::TypeVariable(self.new_type_variable());
                let function_type = Type::function(argument_type, return_type.clone());
                let (mut constraints1, callee) = self.check(scope, *callee, function_type);
                constraints0.append(&mut constraints1);
                (
                    constraints0,
                    Expression::call(callee, argument),
                    return_type,
                )
            }
        }
    }
    fn check(
        &mut self,
        scope: HashTrieMap<Variable, Type>,
        expression: Expression<Variable>,
        type_: Type,
    ) -> (Vec<Constraint>, Expression<(Variable, Type)>) {
        match (expression, type_) {
            (Expression::Integer(value), Type::Integer) => (vec![], Expression::Integer(value)),
            (
                Expression::Function(parameter, body),
                Type::Function(parameter_type, return_type),
            ) => {
                let scope = scope.insert(parameter, *parameter_type.clone());
                let (constraints, body) = self.check(scope, *body, *return_type);
                (
                    constraints,
                    Expression::function((parameter, *parameter_type), body),
                )
            }
            (expression, type_) => {
                let (mut constraints, expression, actual_type) = self.infer(scope, expression);
                constraints.push(Constraint::Equal((), type_, actual_type));
                (constraints, expression)
            }
        }
    }
}

pub fn infer(expression: Expression<Variable>) -> Type {
    let mut inferrer = Inferrer {};
    let (constraints, expression, type_) = inferrer.infer(HashTrieMap::new(), expression);
    println!("Constraints:");
    for constraint in constraints {
        println!("• {:?}", constraint);
    }
    println!("Typed expression: {:?}", expression);
    return type_;
}
