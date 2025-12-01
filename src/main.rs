use clap::{Parser, Subcommand};
use crate::printer::SExprPrinter;

mod parser;
mod transform;
mod visitor;
mod printer;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named(String)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Symbol(String),
    Number(i32),
    List(Vec<Expression>),
    Typed(Box<Expression>, Type)
}

#[derive(Parser)]
#[command(name = "fsfuzz")]
#[command(about = "S-Expression toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Read an SMT file and print without type annotations
    Strip {
        /// Input file path
        file: String,
    },
    /// Start the interactive REPL
    Repl,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Strip { file }) => {
            strip_types(&file);
        }
        Some(Commands::Repl) | None => {
            run_repl();
        }
    }
}

fn strip_types(file: &str) {
    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let exprs = match Expression::parse_multiple(&content) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    for expr in exprs {
        println!("{}", SExprPrinter::print_untyped(&expr));
    }
}

fn run_repl() {
    use std::io::{self, Write};

    println!("S-Expression Stack REPL");
    println!("Commands:");
    println!("  <expr>          - Push expression onto stack");
    println!("  :a <rulename>   - Apply rule to top of stack");
    println!("  :show           - Show stack");
    println!("  :pop            - Pop top element");
    println!("  :load <file>    - Load rules from file");
    println!("  quit/exit       - Exit REPL\n");

    let mut stack: Vec<Expression> = Vec::new();
    let mut rules = Vec::new();

    // Try to load default rules
    if let Ok(loaded_rules) = transform::load_rules("example_transforms.lisp") {
        println!("Loaded {} rules from example_transforms.lisp", loaded_rules.len());
        for rule in &loaded_rules {
            println!("  - {}", rule.name);
        }
        rules = loaded_rules;
        println!();
    }

    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("({}) > ", stack.len());
        io::stdout().flush().unwrap();

        input.clear();
        match stdin.read_line(&mut input) {
            Ok(_) => {
                let trimmed = input.trim();

                if trimmed == "quit" || trimmed == "exit" {
                    println!("Goodbye!");
                    break;
                }

                if trimmed.is_empty() {
                    continue;
                }

                // Handle commands
                if trimmed.starts_with(':') {
                    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
                    let cmd = parts[0];

                    match cmd {
                        ":show" => {
                            if stack.is_empty() {
                                println!("Stack is empty");
                            } else {
                                for (i, expr) in stack.iter().enumerate().rev() {
                                    println!("[{}] {}", i, SExprPrinter::print(expr));
                                }
                            }
                        }
                        ":pop" => {
                            if let Some(expr) = stack.pop() {
                                println!("Popped: {:?}", expr);
                            } else {
                                println!("Stack is empty");
                            }
                        }
                        ":a" => {
                            if parts.len() < 2 {
                                println!("Usage: :a <rulename>");
                                continue;
                            }
                            let rulename = parts[1];

                            if stack.is_empty() {
                                println!("Stack is empty");
                                continue;
                            }

                            let rule = rules.iter().find(|r| r.name == rulename);
                            if let Some(rule) = rule {
                                let top = stack.last().unwrap();
                                if let Some(result) = rule.apply(top) {
                                    stack.pop();
                                    stack.push(result.clone());
                                    println!("Applied {} -> {}", rulename, SExprPrinter::print(&result));
                                } else {
                                    println!("Rule {} did not match", rulename);
                                }
                            } else {
                                println!("Rule '{}' not found", rulename);
                            }
                        }
                        ":load" => {
                            if parts.len() < 2 {
                                println!("Usage: :load <file>");
                                continue;
                            }
                            let filename = parts[1];
                            match transform::load_rules(filename) {
                                Ok(loaded) => {
                                    println!("Loaded {} rules from {}", loaded.len(), filename);
                                    rules = loaded;
                                }
                                Err(e) => {
                                    println!("Error loading rules: {}", e);
                                }
                            }
                        }
                        _ => {
                            println!("Unknown command: {}", cmd);
                        }
                    }
                } else {
                    // Parse and push expression
                    match Expression::parse(trimmed) {
                        Ok(expr) => {
                            println!("Pushed: {}", SExprPrinter::print(&expr));
                            stack.push(expr);
                        }
                        Err(e) => {
                            println!("Parse error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error reading input: {}", e);
                break;
            }
        }
    }
}