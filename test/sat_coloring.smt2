(define-const colors (FiniteSet Int) (set.range 1 3):set)

(declare-const v1 Int)
(declare-const v2 Int)
(declare-const v3 Int)

(assert (set.in v1 colors:set):bool)
(assert (set.in v2 colors:set):bool)
(assert (set.in v3 colors:set):bool)

; adjacent vertices differ
(assert (not (= v1 v2)))
(assert (not (= v2 v3)))
(assert (not (= v1 v3)))

(check-sat)