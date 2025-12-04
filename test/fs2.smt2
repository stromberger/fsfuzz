(declare-const s (FiniteSet Int))

(assert (= s (set.union s (as set.empty (FiniteSet Int)))))
(check-sat)