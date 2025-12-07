(declare-const s (FiniteSet Int))

(assert (not (set.subset (set.range 3 5):set s:set):bool):bool)
(assert (not (set.subset (set.range 10 14):set s:set):bool):bool)
(assert (not (set.subset (set.range 16 20):set s:set):bool):bool)
(assert (not (set.subset (set.range 12 18) s)))

(assert (not (set.in 5 s:set):bool))
(assert (not (set.in 11 s:set):bool))
(assert (not (set.in 17 s:set):bool))

(check-sat)