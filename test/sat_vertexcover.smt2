(define-const V (FiniteSet Int) (set.range 1 4):set)

(declare-const cover (FiniteSet Int))
(assert (set.subset cover V:set):bool)

; Every edge has at least one endpoint in cover
(assert (or (set.in 1 cover:set):bool (set.in 2 cover:set):bool))
(assert (or (set.in 2 cover:set):bool (set.in 3 cover:set):bool))
(assert (or (set.in 1 cover:set):bool (set.in 3 cover:set):bool))
(assert (or (set.in 3 cover:set):bool (set.in 4 cover:set):bool))

; minimum size
(assert (= (set.size cover) 2))

(check-sat)