use pest::Parser;
use pest_derive::Parser;
use crate::{Expression, Type};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct SExprParser;

impl Expression {
    pub fn parse(input: &str) -> Result<Self, pest::error::Error<Rule>> {
        let mut pairs = SExprParser::parse(Rule::sexpr, input)?;
        Ok(parse_sexpr(pairs.next().unwrap()))
    }

    pub fn parse_multiple(input: &str) -> Result<Vec<Self>, pest::error::Error<Rule>> {
        let mut pairs = SExprParser::parse(Rule::file, input)?;
        let file_pair = pairs.next().unwrap();
        Ok(file_pair.into_inner()
            .filter(|p| p.as_rule() == Rule::sexpr)
            .map(parse_sexpr)
            .collect())
    }
}

fn parse_sexpr(pair: pest::iterators::Pair<Rule>) -> Expression {
    match pair.as_rule() {
        Rule::sexpr => {
            let inner = pair.into_inner().next().unwrap();
            parse_sexpr(inner)
        }
        Rule::atom => {
            let inner = pair.into_inner().next().unwrap();
            parse_sexpr(inner)
        }
        Rule::symbol => Expression::Symbol(pair.as_str().to_string()),
        Rule::number => Expression::Number(pair.as_str().parse().unwrap()),
        Rule::list => {
            let elements = pair.into_inner()
                .map(parse_sexpr)
                .collect();
            Expression::List(elements)
        }
        Rule::typed => {
            let mut inner = pair.into_inner();
            let expr = parse_sexpr(inner.next().unwrap());
            let type_name = inner.next().unwrap().as_str().to_string();
            Expression::Typed(Box::new(expr), Type::Named(type_name))
        }
        _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
    }
}
