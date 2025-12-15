# fsfuzz
A simple adaptable metamorphic fuzzer for generating SMT-LIB2 programs.

## Implementation
I want to give a short overview of some implementation details of `fsfuzz`. It is implemented in Rust and supports defining arbitrary transformation rules in a SMT-LIB2 like domain specific language. In line with this guest lecture's topic, I provided about a dozen metamorphic relations  and a corpus of seed programs for the theory of finite sets.

### Transformation Rules
I implemented the rules in a lisp-ish DSL that is similar to the SMT-LIB2 format, albeit with a simple type system. Expressions can have types denoted with a `:` suffix. For example, `:set` denotes that the expression is of type `set`. By default expressions are untyped and have an implicit `any` type.
Transformation rules are defined by `deft` with the first argument the pattern and the second one the replacement. In the pattern and replacement expressions can be captured by variables prefixed with `?` (e.g. `?x`). The types in the expression considered for replacement have to match the types in the pattern.

For example a rule that encodes $x \in S \iff (\{x\} \cup S) = S$ can be defined as:

```lisp
(deft in-to-union-absorb
    (set.in ?x ?t:set):bool
    (= (set.union (set.singleton ?x):set ?t):set ?t):bool)
```

This rule applied to the (type annotated) SMT-LIB2 program
```lisp
(declare-const s (FiniteSet Int))

(assert (set.in 4 s:set):bool)
(assert (= (set.size s) 4))

(check-sat)
```

results in the equisatisfiable program
```lisp
(declare-const s (FiniteSet Int))

(assert (= (set.union (set.singleton 4) s) s))
(assert (= (set.size s) 4))

(check-sat)
```

### Fuzzer
The fuzzer takes a directory of (type annotated) SMT-LIB2 programs and a set of transformation rules. Every iteration one of the seed programs is selected and mutated multiple times. The resulting program is then stripped of types and checked for satisfiability by `z3` (or any SMT solver that accepts SMT-LIB2 input). If the mutated program does not agree on the satisfiability of the seed program or does not terminate within a reasonable time an error is emitted.

```
fsfuzz --rules <rules>.lisp -i <iterations> -m <max amount of mutations per iteration> <seed program dir>
```