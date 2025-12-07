(deft union-empty-set
    ?x:set
    (set.union (as set.empty (FiniteSet Int)):set ?x):set)

(deft union-self
    ?x:set
    (set.union ?x ?x):set)

(deft intersect-self
    ?x:set
    (set.intersect ?x ?x):set)

(deft union-empty-set
    ?x:set
     (set.union ?x (as set.empty (FiniteSet Int)):set):set)

(deft intersect-self-union-singleton
    ?x:set
    (set.intersect ?x (set.union ?x (set.singleton $random-uint):set):set):set)

(deft difference-singleton-difference-self
    ?x:set
    (set.difference ?x (set.difference (set.singleton $random-uint):set ?x):set):set)

(deft true-empty-subset-singleton
    true
    (set.subset (as set.empty (FiniteSet Int)):set (set.singleton $random-uint):set))

(deft true-not-member-empty
    true
    (not (set.member $random-uint (as set.empty (FiniteSet Int)):set)))

(deft false-not-true
    false
   (not true))

(deft false-singleton-subset-empty
    false
    (set.subset (set.singleton $random-uint):set (as set.empty (FiniteSet Int)):set))

(deft in-to-subset
     (set.in ?x ?t:set):bool
     (set.subset (set.singleton ?x):set ?t):bool)

(deft in-to-union-absorb
    (set.in ?x ?t:set):bool
    (= (set.union (set.singleton ?x):set ?t):set ?t):bool)

(deft in-to-intersect-nonempty
    (set.in ?x ?t:set):bool
    (not (= (set.intersect (set.singleton ?x) ?t):set (as set.empty (FiniteSet Int)):set)))

(deft in-to-difference-changes
    (set.in ?x ?t:set):bool
    (not (= (set.difference ?t (set.singleton ?x)) ?t)))