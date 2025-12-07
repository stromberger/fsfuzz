(declare-const lo Int)
(declare-const hi Int)
(declare-const a (FiniteSet Int))

(assert (= a (set.range lo hi):set))
(assert (not (set.in hi a:set):bool))
(check-sat)