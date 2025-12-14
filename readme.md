# fsfuzz
Simple metamorphic fuzzer for SMT solvers.

## Transformation Rules


```lisp
(deft in-to-union-absorb
    (set.in ?x ?t:set):bool
    (= (set.union (set.singleton ?x):set ?t):set ?t):bool)
```
## Let the fuzzing begin

```
fsfuzz --rules <rules>.lisp -i <iterations> -m <max amount of mutations per tieration> <seed program dir>
```
