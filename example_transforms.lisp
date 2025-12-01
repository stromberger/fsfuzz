(deft add-simpl
    (add 0 ?x:num)
    ?x)
(deft add-zero
    ?x:num
    (add 0 ?x):num)

(deft union-empty-set
    ?x:set
    (union (set) ?x):set)

(deft intersection-self
    ?x:set
    (intersection ?x ?x):set)

(deft union-self
    ?x:set
    (union ?x ?x):set)