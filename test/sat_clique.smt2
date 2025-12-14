(define-const edge-12 (FiniteSet Int) (set.range 1 2))
(define-const edge-23 (FiniteSet Int) (set.range 2 3))
(define-const edge-13 (FiniteSet Int) (set.union (set.singleton 1):set (set.singleton 3):set):set)
(define-const edge-34 (FiniteSet Int) (set.range 3 4):set)

(declare-const clique (FiniteSet Int))
(assert (= (set.size clique) 3))
(assert (set.subset clique (set.range 1 4):set))

(declare-const v1 Int)
(declare-const v2 Int)
(declare-const v3 Int)

(assert (= clique (set.union (set.singleton v1):set (set.singleton v2):set (set.singleton v3):set):set))
; ensure ordered solution for is-edge
(assert (< v1 v2))
(assert (< v2 v3))

; Check each pair is an edge
(define-fun is-edge ((a Int) (b Int)) Bool
    (or (and (= a 1) (= b 2))
        (and (= a 2) (= b 3))
        (and (= a 1) (= b 3))
        (and (= a 3) (= b 4))))

(assert (is-edge v1 v2))
(assert (is-edge v2 v3))
(assert (is-edge v1 v3))

(check-sat)