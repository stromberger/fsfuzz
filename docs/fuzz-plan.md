# Fuzz Command Plan

## Overview
Add a CLI command `fsfuzz fuzz` that repeatedly mutates an SMT2 formula and runs z3 to detect solver bugs via metamorphic testing.

## CLI Interface

```bash
fsfuzz fuzz <input-file> --rules <rules-file> [--iterations <n>] [--max-mutations <m>] [--seed <seed>]
```

### Arguments
- `input-file`: Path to seed SMT2 file (must be SAT)
- `--rules`: Path to transforms file
- `--iterations` (optional): Number of fuzzing iterations (default: 100)
- `--max-mutations` (optional): Max mutations per iteration (default: 5)
- `--seed` (optional): Random seed for reproducibility

## Implementation Steps

### 1. Add `Fuzz` subcommand to `main.rs`

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

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
```

### 2. Implement `run_z3()` helper

```rust
use std::process::{Command, Stdio};
use std::io::Write;

const Z3_PATH: &str = "/Users/alex/Desktop/unistuff/smt/z3/build/z3";

fn run_z3(formula: &str) -> Result<String, String> {
    let mut child = Command::new(Z3_PATH)
        .arg("-in")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn z3: {}", e))?;

    child.stdin.take().unwrap()
        .write_all(formula.as_bytes())
        .map_err(|e| format!("Failed to write to z3 stdin: {}", e))?;

    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait for z3: {}", e))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

### 3. Implement `fuzz_loop()` function

```rust
fn fuzz_loop(
    file: &str,
    rules_file: &str,
    iterations: usize,
    max_mutations: usize,
    seed: Option<u64>,
) {
    // 1. Parse input file
    // 2. Load rules
    // 3. Initialize RNG

    let mut bugs_found = 0;

    for i in 0..iterations {
        // Pick random mutation count: 1..=max_mutations
        let count = rng.gen_range(1..=max_mutations);

        // Apply mutations
        let (mutated, applied_rules) = mutate_n(&expr, &rules, count, &mut rng);

        // Convert to string
        let formula = SExprPrinter::print_untyped(&mutated);

        // Run z3
        let result = run_z3(&formula);

        match result.as_str() {
            "usnsat" => {
                // Expected result, print progress
                eprint!(".");
            }
            other => {
                // Bug found!
                bugs_found += 1;
                eprintln!("\n[BUG #{}] iteration {}: expected sat, got '{}'", bugs_found, i, other);
                eprintln!("Applied rules: {}", applied_rules.join(", "));
                eprintln!("Formula:\n{}\n", formula);
            }
        }
    }

    eprintln!("\n\nCompleted {} iterations, found {} bugs", iterations, bugs_found);
}
```

## Output Format

```
$ fsfuzz fuzz test/fs1.smt2 --rules transforms.lisp --iterations 1000

.....................................[BUG #1] iteration 37: expected sat, got 'unsat'
Applied rules: union-self, inter-self-union-singleton
Formula:
(declare-const s (Set Int))
(assert (= s (set.inter s (set.union s (set.singleton 347)))))
(check-sat)

...............................................................
Completed 1000 iterations, found 1 bugs
```

## Future Enhancements

- Save bugs to `bugs/` directory with timestamp
- Timeout for z3 execution
- Support for different expected results (unsat seeds)
- Parallel fuzzing
- Shrinking/minimization of bug-triggering formulas
