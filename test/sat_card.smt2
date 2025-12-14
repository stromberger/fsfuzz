(declare-const s (FiniteSet Int))

(assert (set.in 4 s:set):bool)
(assert (= (set.size s) 4))

(check-sat)