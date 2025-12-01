(deft union-empty-set
    ?x:set
    (set.union (set) ?x):set)

(deft intersection-self
    ?x:set
    (set.intersection ?x ?x):set)

(deft union-self
    ?x:set
    (set.union ?x ?x):set)