# Mutate Command Plan

## Overview
Add a CLI command `fsfuzz mutate` that reads an SMT file, randomly applies transformation rules, and outputs the mutated result.

## CLI Interface

```bash
fsfuzz mutate <input-file> --rules <rules-file> [--count <n>] [--seed <seed>]
```

### Arguments
- `input-file`: Path to the SMT file to mutate
- `--rules`: Path to the transforms file (e.g., `transforms.lisp`)
- `--count` (optional): Number of mutations to apply (default: 1)
- `--seed` (optional): Random seed for reproducibility

## Implementation Steps

### 1. Add `rand` dependency to Cargo.toml
```toml
rand = "0.8"
```

### 2. Create mutation logic in `src/transform.rs`

Add a function to recursively try applying rules at random positions:

```rust
pub fn mutate_random(
    expr: &Expression,
    rules: &[Rule],
    rng: &mut impl Rng,
) -> (Expression, bool)
```

Strategy:
- Walk the expression tree
- At each node, collect all applicable rules
- With some probability, apply a random matching rule
- If no rule matches at current node, recurse into children
- Return the mutated expression and whether a mutation occurred

### 3. Add `Mutate` subcommand to `main.rs`

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Mutate an SMT file by applying random transforms
    Mutate {
        /// Input file path
        file: String,

        /// Rules file path
        #[arg(short, long)]
        rules: String,

        /// Number of mutations to apply
        #[arg(short, long, default_value = "1")]
        count: usize,

        /// Random seed for reproducibility
        #[arg(short, long)]
        seed: Option<u64>,
    },
}
```

### 4. Implement `mutate_file` function in `main.rs`

```rust
fn mutate_file(file: &str, rules_file: &str, count: usize, seed: Option<u64>) {
    // 1. Read and parse input file
    // 2. Load rules from rules file
    // 3. Initialize RNG (with seed if provided)
    // 4. For each expression in file:
    //    - Apply `count` random mutations
    // 5. Print mutated expressions
}
```

## Mutation Strategy Options

### Option A: Single-point mutation
- Pick one random subexpression
- Try all rules, pick one that matches
- Apply it

### Option B: Walk-and-mutate
- Walk the tree depth-first
- At each node, with probability p, try to apply a rule
- Continue until `count` mutations are made

### Option C: Targeted mutation (recommended)
- Collect all (position, applicable_rules) pairs
- Randomly select `count` of them
- Apply the mutations

## Example Usage

```bash
# Apply 1 random mutation
fsfuzz mutate test/sat_simple_eq.smt2 --rules transforms.lisp

# Apply 3 mutations with a fixed seed
fsfuzz mutate test/sat_simple_eq.smt2 --rules transforms.lisp --count 3 --seed 42

# Pipe to file
fsfuzz mutate test/sat_simple_eq.smt2 --rules transforms.lisp > mutated.smt2
```

## Output Format
- Print each mutated top-level expression on its own line
- Use `SExprPrinter::print_untyped()` to strip type annotations from output
