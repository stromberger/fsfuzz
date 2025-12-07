(declare-const s (FiniteSet Int))

(assert (not (set.subset (set.range 1 2) s)))
(assert (not (set.subset (set.range 2 2) s)))

(assert (not (set.in 1 s)))

(check-sat)