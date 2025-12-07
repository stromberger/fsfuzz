(declare-const s (FiniteSet Int))

(assert (set.in 4 s):bool)
(assert (= (set.size s) 4))
(assert (= s (set.union s (as set.empty (FiniteSet Int)))))
(check-sat)