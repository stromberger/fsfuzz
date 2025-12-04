use crate::{Expression, Type};
use rand::seq::SliceRandom;
use rand::Rng;
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

    pub fn apply_with_rng<R: Rng>(&self, expr: &Expression, rng: &mut R) -> Option<Expression> {
        transform_with_rng(&self.pattern, &self.replacement, expr, &mut Some(rng))
    }
}

fn is_var(s: &str) -> bool {
    s.starts_with('?')
}

fn matches(pattern: &Expression, expr: &Expression, bindings: &mut Bindings) -> bool {
    match (pattern, expr) {
        // Typed pattern variable - only matches typed expressions with same type
        (Expression::Typed(p_expr, p_type), Expression::Typed(e_expr, e_type)) if matches_var(p_expr) => {
            if p_type != e_type {
                return false;
            }
            if let Expression::Symbol(v) = p_expr.as_ref() {
                bindings.insert(v.clone(), (expr.clone(), Some(p_type.clone())));
                true
            } else {
                false
            }
        }
        // Typed pattern variable with untyped expression - no match
        (Expression::Typed(p_expr, _), _) if matches_var(p_expr) => {
            false
        }
        // Untyped pattern variable - matches anything
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
        // Typed expressions - match underlying expression and require same type
        (Expression::Typed(p_expr, p_type), Expression::Typed(e_expr, e_type)) => {
            p_type == e_type && matches(p_expr, e_expr, bindings)
        }
        // Untyped pattern can match typed expression (strips type)
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

fn is_special(s: &str) -> bool {
    s.starts_with('$')
}

fn replace<R: Rng>(template: &Expression, bindings: &Bindings, rng: &mut Option<&mut R>) -> Expression {
    match template {
        Expression::Symbol(s) if is_special(s) => {
            match s.as_str() {
                "$random-uint" => {
                    let n = rng.as_mut().map(|r| r.gen_range(0i32..=1000)).unwrap_or(0);
                    Expression::Number(n)
                }
                _ => panic!("Unknown special operator: {}", s),
            }
        }
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
            Expression::List(ts.iter().map(|t| replace(t, bindings, rng)).collect())
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
            Expression::Typed(Box::new(replace(expr, bindings, rng)), typ.clone())
        }
        e => e.clone(),
    }
}

pub fn transform(pattern: &Expression, replacement: &Expression, expr: &Expression) -> Option<Expression> {
    transform_with_rng::<rand::rngs::ThreadRng>(pattern, replacement, expr, &mut None)
}

pub fn transform_with_rng<R: Rng>(
    pattern: &Expression,
    replacement: &Expression,
    expr: &Expression,
    rng: &mut Option<&mut R>,
) -> Option<Expression> {
    let mut bindings = HashMap::new();
    if matches(pattern, expr, &mut bindings) {
        Some(replace(replacement, &bindings, rng))
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

/// Represents a mutation point: a path to a subexpression and applicable rules
struct MutationPoint {
    path: Vec<usize>,
    rule_indices: Vec<usize>,
}

/// Collect all possible mutation points in an expression
fn collect_mutation_points(
    expr: &Expression,
    rules: &[Rule],
    path: &mut Vec<usize>,
    points: &mut Vec<MutationPoint>,
) {
    // Check which rules apply at this node
    let applicable: Vec<usize> = rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| rule.apply(expr).is_some())
        .map(|(i, _)| i)
        .collect();

    if !applicable.is_empty() {
        points.push(MutationPoint {
            path: path.clone(),
            rule_indices: applicable,
        });
    }

    // Recurse into children
    match expr {
        Expression::List(children) => {
            for (i, child) in children.iter().enumerate() {
                path.push(i);
                collect_mutation_points(child, rules, path, points);
                path.pop();
            }
        }
        Expression::Typed(inner, _) => {
            path.push(0);
            collect_mutation_points(inner, rules, path, points);
            path.pop();
        }
        _ => {}
    }
}

/// Get a subexpression at a given path
fn get_at_path<'a>(expr: &'a Expression, path: &[usize]) -> &'a Expression {
    if path.is_empty() {
        return expr;
    }

    match expr {
        Expression::List(children) => get_at_path(&children[path[0]], &path[1..]),
        Expression::Typed(inner, _) => get_at_path(inner, &path[1..]),
        _ => panic!("Invalid path"),
    }
}

/// Replace a subexpression at a given path
fn replace_at_path(expr: &Expression, path: &[usize], replacement: Expression) -> Expression {
    if path.is_empty() {
        return replacement;
    }

    match expr {
        Expression::List(children) => {
            let mut new_children = children.clone();
            new_children[path[0]] = replace_at_path(&children[path[0]], &path[1..], replacement);
            Expression::List(new_children)
        }
        Expression::Typed(inner, typ) => {
            Expression::Typed(
                Box::new(replace_at_path(inner, &path[1..], replacement)),
                typ.clone(),
            )
        }
        _ => panic!("Invalid path"),
    }
}

/// Apply a single random mutation to an expression
/// Returns the mutated expression and the name of the rule applied, or None if no mutation possible
pub fn mutate_once<R: Rng>(
    expr: &Expression,
    rules: &[Rule],
    rng: &mut R,
) -> Option<(Expression, String)> {
    let mut points = Vec::new();
    let mut path = Vec::new();
    collect_mutation_points(expr, rules, &mut path, &mut points);

    if points.is_empty() {
        return None;
    }

    // Pick a random mutation point
    let point = points.choose(rng)?;

    // Pick a random applicable rule
    let rule_idx = *point.rule_indices.choose(rng)?;
    let rule = &rules[rule_idx];

    // Get the subexpression and apply the rule
    let subexpr = get_at_path(expr, &point.path);
    let mutated_subexpr = rule.apply_with_rng(subexpr, rng)?;

    // Replace in the original expression
    let result = replace_at_path(expr, &point.path, mutated_subexpr);
    Some((result, rule.name.clone()))
}

/// Apply multiple random mutations to an expression
/// Returns the mutated expression and the list of rules applied
pub fn mutate_n<R: Rng>(
    expr: &Expression,
    rules: &[Rule],
    count: usize,
    rng: &mut R,
) -> (Expression, Vec<String>) {
    let mut current = expr.clone();
    let mut applied = Vec::new();

    for _ in 0..count {
        match mutate_once(&current, rules, rng) {
            Some((mutated, rule_name)) => {
                current = mutated;
                applied.push(rule_name);
            }
            None => break,
        }
    }

    (current, applied)
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

        // Test applying the first rule: (add 0 ?x:num) -> ?x
        // Expression must be typed to match the typed pattern
        let expr = Expression::List(vec![
            Expression::Symbol("add".to_string()),
            Expression::Number(0),
            Expression::Typed(
                Box::new(Expression::Number(42)),
                Type::Named("num".to_string())
            ),
        ]);
        let result = rules[0].apply(&expr);
        assert_eq!(result, Some(Expression::Typed(
            Box::new(Expression::Number(42)),
            Type::Named("num".to_string())
        )));
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

        // Expression: (union empty (set 1 2 3):set) - must be typed to match
        let expr = Expression::List(vec![
            Expression::Symbol("union".to_string()),
            Expression::Symbol("empty".to_string()),
            Expression::Typed(
                Box::new(Expression::List(vec![
                    Expression::Symbol("set".to_string()),
                    Expression::Number(1),
                    Expression::Number(2),
                    Expression::Number(3),
                ])),
                Type::Named("set".to_string())
            ),
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

    #[test]
    fn test_typed_pattern_rejects_untyped_expr() {
        // Pattern: ?x:set should NOT match untyped expression
        let pattern = Expression::Typed(
            Box::new(Expression::Symbol("?x".to_string())),
            Type::Named("set".to_string())
        );
        let replacement = Expression::Symbol("?x".to_string());

        // Untyped expression
        let expr = Expression::Symbol("s".to_string());

        let result = transform(&pattern, &replacement, &expr);
        assert_eq!(result, None);
    }

    #[test]
    fn test_typed_pattern_rejects_wrong_type() {
        // Pattern: ?x:set should NOT match expression with different type
        let pattern = Expression::Typed(
            Box::new(Expression::Symbol("?x".to_string())),
            Type::Named("set".to_string())
        );
        let replacement = Expression::Symbol("?x".to_string());

        // Expression with wrong type
        let expr = Expression::Typed(
            Box::new(Expression::Symbol("s".to_string())),
            Type::Named("num".to_string())
        );

        let result = transform(&pattern, &replacement, &expr);
        assert_eq!(result, None);
    }
}
