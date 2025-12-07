(declare-const s (FiniteSet Int))
(declare-const t (FiniteSet Int))
(declare-const u (FiniteSet Int))
(declare-const x Int)

(assert (= s (set.union t u):set))
(assert (set.in x t:set):bool)
(assert (not (set.in x s:set):bool))
(check-sat)