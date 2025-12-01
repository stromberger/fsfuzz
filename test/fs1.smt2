(declare-const s (FiniteSet Int))

(assert (= s (set.union s (empty-set)))
(check-sat)