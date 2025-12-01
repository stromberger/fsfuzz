use crate::{Expression, Type};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type Bindings = HashMap<String, (Expression, Option<Type>)>;

pub struct Rule {
    pub name: String,
    pub pattern: Expression,
    pub replacement: Expression,
}

impl Rule {
    pub fn new(name: String, pattern: Expression, replacement: Expression) -> Self {
        Rule {
            name,
            pattern,
            replacement,
        }
    }

    pub fn apply(&self, expr: &Expression) -> Option<Expression> {
        transform(&self.pattern, &self.replacement, expr)
    }
}

fn is_var(s: &str) -> bool {
    s.starts_with('?')
}

fn matches(pattern: &Expression, expr: &Expression, bindings: &mut Bindings) -> bool {
    match (pattern, expr) {
        // Typed pattern variable - store the type!
        (Expression::Typed(p_expr, p_type), _) if matches_var(p_expr) => {
            if let Expression::Symbol(v) = p_expr.as_ref() {
                bindings.insert(v.clone(), (expr.clone(), Some(p_type.clone())));
                true
            } else {
                false
            }
        }
        // Untyped pattern variable - no type info
        (Expression::Symbol(v), _) if is_var(v) => {
            bindings.insert(v.clone(), (expr.clone(), None));
            true
        }
        // Exact matches
        (Expression::Symbol(a), Expression::Symbol(b)) => a == b,
        (Expression::Number(a), Expression::Number(b)) => a == b,
        (Expression::List(ps), Expression::List(es)) if ps.len() == es.len() => {
            ps.iter().zip(es.iter()).all(|(p, e)| matches(p, e, bindings))
        }
        // Typed expressions - match underlying expression and ignore type for now
        (Expression::Typed(p_expr, _), Expression::Typed(e_expr, _)) => {
            matches(p_expr, e_expr, bindings)
        }
        (p, Expression::Typed(e_expr, _)) => matches(p, e_expr, bindings),
        _ => false,
    }
}

fn matches_var(expr: &Expression) -> bool {
    match expr {
        Expression::Symbol(s) => is_var(s),
        _ => false,
    }
}

fn replace(template: &Expression, bindings: &Bindings) -> Expression {
    match template {
        Expression::Symbol(v) if is_var(v) => {
            let (expr, typ) = &bindings[v];

            // If pattern had type info, add it ONLY if expression isn't already typed
            if let Some(t) = typ {
                match expr {
                    Expression::Typed(_, _) => {
                        // Already typed, return as-is to avoid stacking
                        expr.clone()
                    }
                    _ => {
                        // Not typed, add the type from pattern
                        Expression::Typed(Box::new(expr.clone()), t.clone())
                    }
                }
            } else {
                // No type info, return as-is
                expr.clone()
            }
        }
        Expression::List(ts) => {
            Expression::List(ts.iter().map(|t| replace(t, bindings)).collect())
        }
        // Typed variable in template - explicitly apply the template's type
        Expression::Typed(expr, typ) => {
            if let Expression::Symbol(v) = expr.as_ref() {
                if is_var(v) {
                    let (bound_expr, _) = &bindings[v];
                    // Strip any existing type and apply template's type
                    let untyped = match bound_expr {
                        Expression::Typed(inner, _) => (**inner).clone(),
                        other => other.clone(),
                    };
                    return Expression::Typed(Box::new(untyped), typ.clone());
                }
            }
            Expression::Typed(Box::new(replace(expr, bindings)), typ.clone())
        }
        e => e.clone(),
    }
}

pub fn transform(pattern: &Expression, replacement: &Expression, expr: &Expression) -> Option<Expression> {
    let mut bindings = HashMap::new();
    if matches(pattern, expr, &mut bindings) {
        Some(replace(replacement, &bindings))
    } else {
        None
    }
}

/// Load rules from a file
/// Format: (deft name pattern replacement)
pub fn load_rules<P: AsRef<Path>>(path: P) -> Result<Vec<Rule>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let exprs = Expression::parse_multiple(&content)
        .map_err(|e| format!("Parse error: {}", e))?;

    let mut rules = Vec::new();

    for expr in exprs {
        if let Expression::List(items) = expr {
            if items.len() == 4 {
                if let Expression::Symbol(cmd) = &items[0] {
                    if cmd == "deft" {
                        let name = match &items[1] {
                            Expression::Symbol(s) => s.clone(),
                            _ => return Err("Rule name must be a symbol".to_string()),
                        };
                        rules.push(Rule::new(name, items[2].clone(), items[3].clone()));
                        continue;
                    }
                }
            }
            return Err(format!("Invalid rule format: expected (deft name pattern replacement)"));
        }
    }

    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_transform() {
        // Pattern: (+ 0 ?x) -> ?x
        let pattern = Expression::List(vec![
            Expression::Symbol("+".to_string()),
            Expression::Number(0),
            Expression::Symbol("?x".to_string()),
        ]);
        let replacement = Expression::Symbol("?x".to_string());
        let expr = Expression::List(vec![
            Expression::Symbol("+".to_string()),
            Expression::Number(0),
            Expression::Number(42),
        ]);

        let result = transform(&pattern, &replacement, &expr);
        assert_eq!(result, Some(Expression::Number(42)));
    }

    #[test]
    fn test_no_match() {
        // Pattern: (+ 0 ?x)
        let pattern = Expression::List(vec![
            Expression::Symbol("+".to_string()),
            Expression::Number(0),
            Expression::Symbol("?x".to_string()),
        ]);
        let replacement = Expression::Symbol("?x".to_string());
        // Expression: (+ 1 42) - should not match
        let expr = Expression::List(vec![
            Expression::Symbol("+".to_string()),
            Expression::Number(1),
            Expression::Number(42),
        ]);

        let result = transform(&pattern, &replacement, &expr);
        assert_eq!(result, None);
    }

    #[test]
    fn test_nested_transform() {
        // Pattern: (union ?a (empty)) -> ?a
        let pattern = Expression::List(vec![
            Expression::Symbol("union".to_string()),
            Expression::Symbol("?a".to_string()),
            Expression::List(vec![Expression::Symbol("empty".to_string())]),
        ]);
        let replacement = Expression::Symbol("?a".to_string());
        let expr = Expression::List(vec![
            Expression::Symbol("union".to_string()),
            Expression::List(vec![
                Expression::Symbol("set".to_string()),
                Expression::Number(1),
                Expression::Number(2),
            ]),
            Expression::List(vec![Expression::Symbol("empty".to_string())]),
        ]);

        let result = transform(&pattern, &replacement, &expr);
        assert_eq!(result, Some(Expression::List(vec![
            Expression::Symbol("set".to_string()),
            Expression::Number(1),
            Expression::Number(2),
        ])));
    }

    #[test]
    fn test_load_rules() {
        let rules = load_rules("example_transforms.lisp").unwrap();
        assert_eq!(rules[0].name, "add-simpl");
        assert_eq!(rules[1].name, "add-zero");

        // Test applying the first rule: (add 0 ?x) -> ?x
        let expr = Expression::List(vec![
            Expression::Symbol("add".to_string()),
            Expression::Number(0),
            Expression::Number(42),
        ]);
        let result = rules[0].apply(&expr);
        assert_eq!(result, Some(Expression::Number(42)));
    }

    #[test]
    fn test_type_propagation() {
        // Pattern: (union empty ?x:set) -> ?x
        let pattern = Expression::List(vec![
            Expression::Symbol("union".to_string()),
            Expression::Symbol("empty".to_string()),
            Expression::Typed(
                Box::new(Expression::Symbol("?x".to_string())),
                Type::Named("set".to_string())
            ),
        ]);
        let replacement = Expression::Symbol("?x".to_string());

        // Expression: (union empty (set 1 2 3))
        let expr = Expression::List(vec![
            Expression::Symbol("union".to_string()),
            Expression::Symbol("empty".to_string()),
            Expression::List(vec![
                Expression::Symbol("set".to_string()),
                Expression::Number(1),
                Expression::Number(2),
                Expression::Number(3),
            ]),
        ]);

        let result = transform(&pattern, &replacement, &expr);

        // Result should be: (set 1 2 3):set
        // The type from the pattern should be propagated!
        assert_eq!(result, Some(Expression::Typed(
            Box::new(Expression::List(vec![
                Expression::Symbol("set".to_string()),
                Expression::Number(1),
                Expression::Number(2),
                Expression::Number(3),
            ])),
            Type::Named("set".to_string())
        )));
    }
}
