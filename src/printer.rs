use crate::{Expression, Type};
use crate::visitor::Visitor;

pub struct SExprPrinter {
    output: String,
    include_types: bool,
}

impl SExprPrinter {
    pub fn new() -> Self {
        SExprPrinter {
            output: String::new(),
            include_types: true,
        }
    }

    pub fn without_types() -> Self {
        SExprPrinter {
            output: String::new(),
            include_types: false,
        }
    }

    pub fn print(expr: &Expression) -> String {
        let mut printer = SExprPrinter::new();
        expr.accept(&mut printer);
        printer.output
    }

    pub fn print_untyped(expr: &Expression) -> String {
        let mut printer = SExprPrinter::without_types();
        expr.accept(&mut printer);
        printer.output
    }

    pub fn result(self) -> String {
        self.output
    }
}
impl Visitor<()> for SExprPrinter {
    fn visit_symbol(&mut self, s: &String) {
        self.output.push_str(s.to_string().as_str());
    }

    fn visit_number(&mut self, n: i32) {
        self.output.push_str(n.to_string().as_str());
    }

    fn visit_list(&mut self, l: &Vec<Expression>)  {
        self.output.push_str("(");

        for e in l {
            e.accept(self);
            self.output.push_str(" ");
        }

        self.output.pop();
        self.output.push_str(")");
    }

    fn visit_typed(&mut self, expression: &Expression, t: &Type) {
        expression.accept(self);
        if self.include_types {
            self.output.push_str(":");
            match t {
                Type::Named(name) => self.output.push_str(name.as_str()),
            }
        }
    }
}