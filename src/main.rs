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
    /// Mutate an SMT file by applying random transforms
    Mutate {
        /// Input file path
        file: String,
        /// Rules file path
        #[arg(short, long)]
        rules: String,
        /// Number of mutations to apply per expression
        #[arg(short, long, default_value = "3")]
        count: usize,
        /// Random seed for reproducibility
        #[arg(short, long)]
        seed: Option<u64>,
    },
    /// Start the interactive REPL
    Repl,
    /// Fuzz z3 with metamorphic mutations
    Fuzz {
        /// Input file path (seed formula, must be SAT)
        file: String,
        /// Rules file path
        #[arg(short, long)]
        rules: String,
        /// Number of fuzzing iterations
        #[arg(short, long, default_value = "100")]
        iterations: usize,
        /// Max mutations per iteration (actual count is random 1..=max)
        #[arg(short = 'm', long, default_value = "5")]
        max_mutations: usize,
        /// Random seed for reproducibility
        #[arg(short, long)]
        seed: Option<u64>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Strip { file }) => {
            strip_types(&file);
        }
        Some(Commands::Mutate { file, rules, count, seed }) => {
            mutate_file(&file, &rules, count, seed);
        }
        Some(Commands::Repl) | None => {
            run_repl();
        }
        Some(Commands::Fuzz { file, rules, iterations, max_mutations, seed }) => {
            fuzz_loop(&file, &rules, iterations, max_mutations, seed);
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

fn mutate_file(file: &str, rules_file: &str, count: usize, seed: Option<u64>) {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    // Read input file
    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    // Parse expressions
    let exprs = match Expression::parse_multiple(&content) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    // Load rules
    let rules = match transform::load_rules(rules_file) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading rules: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize RNG
    let mut rng: StdRng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    // Mutate the program
    let (mutated_exprs, applied) = transform::mutate_program(&exprs, &rules, count, &mut rng);
    if !applied.is_empty() {
        eprintln!("; Applied: {}", applied.join(", "));
    }
    for expr in mutated_exprs {
        println!("{}", SExprPrinter::print_untyped(&expr));
    }
}

const Z3_PATH: &str = "/Users/alex/Desktop/unistuff/smt/z3/build/z3";
const Z3_TIMEOUT_SECS: u64 = 10;

enum Z3Result {
    Success(String),
    Timeout,
    Error(String),
}

fn run_z3(formula: &str) -> Z3Result {
    use std::process::{Command, Stdio};
    use std::io::{Write, Read};
    use std::time::{Duration, Instant};
    use std::thread;

    let mut child = match Command::new(Z3_PATH)
        .arg("-in")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return Z3Result::Error(format!("Failed to spawn z3: {}", e)),
    };

    // Write formula to stdin
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(formula.as_bytes()) {
            return Z3Result::Error(format!("Failed to write to z3 stdin: {}", e));
        }
    }

    // Wait with timeout
    let start = Instant::now();
    let timeout = Duration::from_secs(Z3_TIMEOUT_SECS);

    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Process finished, read output
                if let Some(mut stdout) = child.stdout.take() {
                    let mut output = String::new();
                    if let Err(e) = stdout.read_to_string(&mut output) {
                        return Z3Result::Error(format!("Failed to read z3 output: {}", e));
                    }
                    return Z3Result::Success(output.trim().to_string());
                }
                return Z3Result::Success(String::new());
            }
            Ok(None) => {
                // Still running
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Z3Result::Timeout;
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Z3Result::Error(format!("Failed to wait for z3: {}", e)),
        }
    }
}

fn fuzz_loop(file: &str, rules_file: &str, iterations: usize, max_mutations: usize, seed: Option<u64>) {
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use rand::Rng;
    use std::io::{self, Write};

    // Read input file
    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let expected_result = if file.contains("unsat") { "unsat" } else { "sat" };

    // Parse expressions
    let exprs = match Expression::parse_multiple(&content) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    // Load rules
    let rules = match transform::load_rules(rules_file) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading rules: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize RNG
    let mut rng: StdRng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    eprintln!("Fuzzing with {} iterations, max {} mutations per iteration", iterations, max_mutations);
    eprintln!("Loaded {} rules, {} seed expressions", rules.len(), exprs.len());
    eprintln!();

    let mut bugs_found = 0;

    for i in 0..iterations {
        // Pick random mutation count: 1..=max_mutations
        let count = rng.gen_range(1..=max_mutations);

        // Mutate the program
        let (mutated_exprs, all_applied_rules) = transform::mutate_program(&exprs, &rules, count, &mut rng);

        // Convert all expressions to string
        let formula: String = mutated_exprs
            .iter()
            .map(|e| SExprPrinter::print_untyped(e))
            .collect::<Vec<_>>()
            .join("\n");

        // Run z3
        match run_z3(&formula) {
            Z3Result::Success(result) => {
                if result == expected_result {
                    eprint!(".");
                    io::stderr().flush().unwrap();
                } else {
                    bugs_found += 1;
                    eprintln!("\n[BUG #{}] iteration {}: expected {}, got '{}'", bugs_found, i, expected_result, result);
                    eprintln!("Applied rules: {}", all_applied_rules.join(", "));
                    eprintln!("Formula:\n{}\n", formula);
                }
            }
            Z3Result::Timeout => {
                bugs_found += 1;
                eprintln!("\n[BUG #{}] iteration {}: TIMEOUT (>{}s)", bugs_found, i, Z3_TIMEOUT_SECS);
                eprintln!("Applied rules: {}", all_applied_rules.join(", "));
                eprintln!("Formula:\n{}\n", formula);
            }
            Z3Result::Error(e) => {
                eprintln!("\n[ERROR] iteration {}: {}", i, e);
            }
        }
    }

    eprintln!("\n\nCompleted {} iterations, found {} bugs (wrong result or timeout)", iterations, bugs_found);
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