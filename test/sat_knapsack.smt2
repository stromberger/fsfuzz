(declare-const selected (FiniteSet Int))

(assert (set.subset selected (set.range 1 3):set))

 (define-const total-value Int
    (+ (ite (set.in 1 selected:set):bool 1 0)
       (ite (set.in 2 selected:set):bool 2 0)
       (ite (set.in 3 selected:set):bool 3 0)))


(assert (<= (set.size selected) 2))
(assert (<= total-value 3))

(check-sat)