use crate::{Expression, Type};

pub trait Visitor<T> {
    fn visit_symbol(&mut self, s: &String) -> T;
    fn visit_number(&mut self, n: i32) -> T;
    fn visit_list(&mut self, l: &Vec<Expression>) -> T;
    fn visit_typed(&mut self, expression: &Expression, t: &Type) -> T;
}

impl Expression {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Expression::Symbol(s) => visitor.visit_symbol(s),
            Expression::Number(n) => visitor.visit_number(*n),
            Expression::List(exprs) => visitor.visit_list(exprs),
            Expression::Typed(expr, t) => {
                let expression: &Expression = &**expr;
                visitor.visit_typed(expression, t)
            }
        }
    }
}