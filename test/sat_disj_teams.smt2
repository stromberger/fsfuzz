(declare-const team-A (FiniteSet Int))
(declare-const team-B (FiniteSet Int))
(define-const people (FiniteSet Int) (set.range 1 10))

; Both teams from same pool
(assert (set.subset team-A people:set):bool)
(assert (set.subset team-B people:set):bool)

; No overlap
(assert (= (set.intersect team-A:set team-B:set):set (as set.empty (FiniteSet Int)):set))

; Team sizes
(assert (= (set.size team-A:set) 3))
(assert (= (set.size team-B:set) 4))

(check-sat)