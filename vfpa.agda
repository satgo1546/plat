{-
  Welcome to Agda! :-)

  If you are new to Agda, you could play The HoTT Game, a tutorial for learning
  Agda and homotopy type theory. You can start the game using the "Help" menu
  and then navigating to a file such as 1FundamentalGroup/Quest0.agda. You
  will also need to open the accompanying guide in your browser:
  https://thehottgameguide.readthedocs.io/

  This editor runs on agdapad.quasicoherent.io. Your Agda code is stored on
  this server and should be available when you revisit the same Agdapad session.
  However, absolutely no guarantees are made. You should make backups by
  downloading (see the clipboard icon in the lower right corner).

  C-c C-l          check file
  C-c C-SPC        check hole
  C-c C-,          display goal and context
  C-c C-c          split cases
  C-c C-r          fill in boilerplate from goal
  C-c C-d          display type of expression
  C-c C-v          evaluate expression (normally this is C-c C-n)
  C-c C-a          try to find proof automatically
  C-z              enable Vi keybindings
  C-x C-+          increase font size
  \bN \alpha \to   math symbols

  "C-c" means "<Ctrl key> + c". In case your browser is intercepting C-c,
  you can also use C-o. In case your browser in intercepting C-SPC, you can
  also use C-p. For pasting code into the Agdapad, see the clipboard
  icon in the lower right corner.

  In text mode, use <F10> to access the menu bar, not the mouse.
-}

open import Relation.Binary.PropositionalEquality using (_≡_; refl; sym; inspect; [_])
open import Agda.Primitive using (Level; _⊔_)
open import Agda.Builtin.Bool using (Bool; false; true)
open import Agda.Builtin.Nat using (Nat; zero; suc; _+_; _*_; _-_; _<_; _==_)
open import Agda.Builtin.List using (List; []; _∷_)
open import Agda.Builtin.Maybe using (Maybe; nothing; just)
open import Agda.Builtin.Sigma

infix 7 ~_
infixr 6 _&&_
infixr 5 _||_
infixl 6 _++_
infixl 6 _++v_
infixl 6 _<=_

~_ : Bool -> Bool
~ true = false
~ false = true

_&&_ : Bool -> Bool -> Bool
true && b = b
false && _ = false

_||_ : Bool -> Bool -> Bool
true || _ = true
false || b = b

ifttt : ∀ {l} {A : Set l} -> Bool -> A -> A -> A
ifttt true t f = t
ifttt false t f = f

~~true : ~ ~ true ≡ true
~~true = refl

~~false : ~ ~ false ≡ false
~~false = refl

~~b : ∀ {b} -> ~ ~ b ≡ b
~~b {false} = refl
~~b {true} = refl

b&&b : ∀ {b} -> b && b ≡ b
b&&b {false} = refl
b&&b {true} = refl

b||b : ∀ {b} -> b || b ≡ b
b||b {false} = refl
b||b {true} = refl

||≡false1 : ∀ {b1 b2} -> b1 || b2 ≡ false -> b1 ≡ false
||≡false1 {false} _ = refl
||≡false1 {true} ()

my-impossible-theorem : true ≡ false -> false ≡ true
my-impossible-theorem p = sym p

||-cong1 : ∀ {b1 b1' b2} -> b1 ≡ b1' -> b1 || b2 ≡ b1' || b2
||-cong1 {b1} {b1} {bbb222} refl = refl

||-cong2 : ∀ {b1 b2 b2'} -> b2 ≡ b2' -> b1 || b2 ≡ b1 || b2'
||-cong2 p rewrite p = refl


ifttt-same : ∀ {a} {A : Set a} -> ∀ (b : Bool) (x : A) -> (ifttt b x x) ≡ x
ifttt-same false _ = refl
ifttt-same true _ = refl

Bool-contra : false ≡ true -> ∀ {P : Set} -> P
Bool-contra ()

demorgan1 : ∀ {b1 b2} -> b1 || b2 ≡ ~ (~ b1 && ~ b2)
demorgan1 {false} {false} = refl
demorgan1 {false} {true} = refl
demorgan1 {true} {false} = refl
demorgan1 {true} {true} = refl

demorgan2 : ∀ {b1 b2} -> ~ (b1 || ~ b2) ≡ ~ b1 && b2
demorgan2 {false} {false} = refl
demorgan2 {false} {true} = refl
demorgan2 {true} {false} = refl
demorgan2 {true} {true} = refl

&&true : ∀ {x} -> x && true ≡ x
&&true {false} = refl
&&true {true} = refl

||false : ∀ {x} -> x || false ≡ x
||false {false} = refl
||false {true} = refl

pred : Nat -> Nat
pred zero = zero
pred (suc x) = x

0+ : ∀ (x : Nat) -> 0 + x ≡ x
0+ x = refl

+0 : ∀ (x : Nat) -> x + 0 ≡ x
+0 zero = refl
+0 (suc w) rewrite +0 w = refl

+suc : ∀ (x y : Nat) -> x + suc y ≡ suc (x + y)
+suc zero y = refl
+suc (suc w) y rewrite +suc w y = refl

+assoc : ∀ (x y z : Nat) -> x + (y + z) ≡ (x + y) + z
+assoc zero y z = refl
+assoc (suc w) y z rewrite +assoc w y z = refl

+comm : ∀ (x y : Nat) -> x + y ≡ y + x
+comm zero y = sym (+0 y)
+comm (suc w) y rewrite +comm w y rewrite +suc y w = refl

*dist : ∀ (x y z : Nat) -> (x + y) * z ≡ x * z + y * z
*dist zero y z = refl
*dist (suc w) y z rewrite *dist w y z = +assoc z (w * z) (y * z)

*0 : ∀ (x : Nat) -> x * 0 ≡ 0
*0 zero = refl
*0 (suc w) = *0 w

*suc : ∀ (x y : Nat) -> x * (suc y) ≡ x + x * y
*suc zero y = refl
*suc (suc w) y rewrite *suc w y | +assoc y w (w * y) | +assoc w y (w * y) | +comm y w = refl

*comm : ∀ (x y : Nat) -> x * y ≡ y * x
*comm zero y rewrite *0 y = refl
*comm (suc w) y rewrite *comm w y | *suc y w = refl

*assoc : ∀ (x y z : Nat) -> x * (y * z) ≡ (x * y) * z
*assoc zero y z = refl
*assoc (suc w) y z rewrite *assoc w y z | *dist y (w * y) z = refl

<-0 : ∀ (x : Nat) -> (x < 0) ≡ false
<-0 zero = refl
<-0 (suc x) = refl

<-trans : ∀ {x y z : Nat} -> (x < y) ≡ true -> (y < z) ≡ true -> (x < z) ≡ true
<-trans {x} {0} p1 p2 = Bool-contra p1
<-trans {0} {suc y} {0} p1 ()
<-trans {0} {suc y} {suc z} p1 p2 = refl
<-trans {suc x} {suc y} {0} p1 ()
<-trans {suc x} {suc y} {suc z} p1 p2 = <-trans {x} {y} {z} p1 p2

_<=_ : Nat -> Nat -> Bool
zero <= _ = true
(suc _) <= 0 = false
(suc a) <= (suc b) = a <= b

<=-trans : ∀ {x y z : Nat} -> (x <= y) ≡ true -> (y <= z) ≡ true -> (x <= z) ≡ true
<=-trans {zero} p1 p2 = refl
<=-trans {suc x} {zero} p1 = Bool-contra p1
<=-trans {suc x} {suc y} {zero} p1 ()
<=-trans {suc x} {suc y} {suc z} p1 p2 = <=-trans {x} {y} {z} p1 p2

<=-suc : ∀ (x : Nat) -> x <= (suc x) ≡ true
<=-suc zero = refl
<=-suc (suc w) rewrite <=-suc w = refl

==-refl : ∀ (x : Nat) -> (x == x) ≡ true
==-refl 0 = refl
==-refl (suc x) = ==-refl x

==-to-≡ : ∀ {x y : Nat} -> (x == y) ≡ true -> x ≡ y
==-to-≡ {zero} {zero} u = refl
==-to-≡ {suc x} {suc y} u rewrite ==-to-≡ {x} {y} u = refl

==-from-≡ : ∀ {x y : Nat} -> x ≡ y -> (x == y) ≡ true
==-from-≡ {x} refl = ==-refl x

is-even : Nat -> Bool
is-odd : Nat -> Bool
is-even zero = true
is-even (suc x) = is-odd x
is-odd zero = false
is-odd (suc x) = is-even x

even~odd : ∀ (x : Nat) -> is-even x ≡ ~ is-odd x
odd~even : ∀ (x : Nat) -> is-odd x ≡ ~ is-even x
even~odd zero = refl
even~odd (suc x) = odd~even x
odd~even zero = refl
odd~even (suc x) = even~odd x


length : ∀ {l} {A : Set l} -> List A -> Nat
length [] = zero
length (x ∷ xs) = suc (length xs)

append : ∀ {l} {A : Set l} -> List A -> List A -> List A
append [] ys = ys
append (x ∷ xs) ys = x ∷ (append xs ys)

_++_ : ∀ {l} {A : Set l} -> List A -> List A -> List A
_++_ = append

map : ∀ {l} {l'} {A : Set l} {A' : Set l'} -> (A -> A') -> List A -> List A'
map f [] = []
map f (x ∷ xs) = (f x) ∷ (map f xs)

filter : ∀ {l} {A : Set l} -> (A -> Bool) -> List A -> List A
filter f [] = []
filter f (x ∷ xs) = ifttt (f x) (x ∷ filter f xs) (filter f xs)

remove : ∀ {l} {A : Set l} -> (A -> A -> Bool) -> A -> List A -> List A
remove eq a l = filter (λ x -> ~ (eq a x)) l

nth : ∀ {l} {A : Set l} -> Nat -> List A -> Maybe A
nth _ [] = nothing
nth zero (x ∷ _) = just x
nth (suc w) (_ ∷ xs) = nth w xs

reverse-helper : ∀ {l} {A : Set l} -> List A -> List A -> List A
reverse-helper h [] = h
reverse-helper h (x ∷ xs) = reverse-helper (x ∷ h) xs

reverse : ∀ {l} {A : Set l} -> List A -> List A
reverse l = reverse-helper [] l

length-++ : ∀ {l} {A : Set l} (l1 l2 : List A) -> length (append l1 l2) ≡ (length l1) + (length l2)
length-++ [] l2 = refl
length-++ (x ∷ l1) l2 rewrite length-++ l1 l2 = refl

++-assoc : ∀ {l} {A : Set l} (l1 l2 l3 : List A) -> (l1 ++ l2) ++ l3 ≡ l1 ++ (l2 ++ l3)
++-assoc [] l2 l3 = refl
++-assoc (x ∷ l1) l2 l3 rewrite ++-assoc l1 l2 l3 = refl

filter-fewer : ∀ {l} {A : Set l} (p : A -> Bool) (l : List A) -> length (filter p l) <= length l ≡ true
filter-fewer p [] = refl
filter-fewer p (x ∷ l) with p x
filter-fewer p (x ∷ l) | true = filter-fewer p l
filter-fewer p (x ∷ l) | false = <=-trans {length (filter p l)} (filter-fewer p l) (<=-suc (length l))

filter-idem : ∀ {l} {A : Set l} (p : A -> Bool) (l : List A) -> (filter p (filter p l)) ≡ (filter p l)
filter-idem p [] = refl
filter-idem p (x ∷ l) with p x | inspect p x
filter-idem p (x ∷ l) | true | [ p' ] rewrite p' | filter-idem p l = refl
filter-idem p (x ∷ l) | false | [ _ ] = filter-idem p l

length-reverse-helper : ∀ {l} {A : Set l} (h l : List A) -> length (reverse-helper h l) ≡ length h + length l
length-reverse-helper h [] = sym (+0 (length h))
length-reverse-helper h (x ∷ l) rewrite length-reverse-helper (x ∷ h) l | +suc (length h) (length l) = refl

length-reverse : ∀ {l} {A : Set l} (l : List A) -> length (reverse l) ≡ length l
length-reverse l rewrite length-reverse-helper [] l = refl

length-map : ∀ {l1 l2} {A : Set l1} {A' : Set l2} (f : A -> A') (l : List A) -> length (map f l) ≡ length l
length-map f [] = refl
length-map f (x ∷ l) rewrite length-map f l = refl

repeat : ∀ {l} {A : Set l} -> Nat -> A -> List A
repeat zero _ = []
repeat (suc n) a = a ∷ (repeat n a)

take-while : ∀ {l} {A : Set l} -> (A -> Bool) -> List A -> List A
take-while p [] = []
take-while p (x ∷ l) = ifttt (p x) (x ∷ (take-while p l)) []

take-while-repeat : ∀ {l} {A : Set l} (n : Nat) (a : A) (p : A -> Bool) -> p a ≡ true -> take-while p (repeat n a) ≡ repeat n a
take-while-repeat zero a p p1 = refl
take-while-repeat (suc w) a p p1 rewrite p1 | take-while-repeat w a p p1 = refl

take : ∀ {l} {A : Set l} -> Nat -> List A -> List A
take zero _ = []
take (suc n) [] = []
take (suc n) (x ∷ xs) = x ∷ (take n xs)

nth-tail : ∀ {l} {A : Set l} -> Nat -> List A -> List A
nth-tail zero l = l
nth-tail (suc n) [] = []
nth-tail (suc n) (x ∷ xs) = nth-tail n xs

∷-++-assoc : ∀ {l} {A : Set l} (x : A) (l1 : List A) (l2 : List A) -> (x ∷ l1) ++ l2 ≡ x ∷ (l1 ++ l2)
∷-++-assoc x l1 l2 = refl

take-++-nth-tail : ∀ {lv} {A : Set lv} (n : Nat) (l : List A) -> (take n l) ++ (nth-tail n l) ≡ l
take-++-nth-tail zero l = refl
take-++-nth-tail (suc n) [] = refl
take-++-nth-tail (suc n) (x ∷ l) rewrite ∷-++-assoc x (take n l) (nth-tail n l) | take-++-nth-tail n l = refl

data Vec {l} (A : Set l) : Nat -> Set l where
  [] : Vec A 0
  _∷_ : {n : Nat} (x : A) (xs : Vec A n) -> Vec A (suc n)

test-vec : Vec Bool 4
test-vec = false ∷ true ∷ false ∷ false ∷ []

test-list : List (Vec Bool 2)
test-list = (false ∷ true ∷ []) ∷ (true ∷ false ∷ []) ∷ (true ∷ false ∷ []) ∷ []

test-vec3 : Vec (Vec Bool 3) 2
test-vec3 = (true ∷ true ∷ true ∷ []) ∷ (false ∷ false ∷ false ∷ []) ∷ []

_++v_ : ∀ {l} {A : Set l} {n m : Nat} -> Vec A n -> Vec A m -> Vec A (n + m)
[] ++v ys = ys
(x ∷ xs) ++v ys = x ∷ (xs ++v ys)

test-vec-++v : Vec Bool 8
test-vec-++v = test-vec ++v test-vec

head-v : ∀ {l} {A : Set l} {n : Nat} -> Vec A (suc n) -> A
head-v (x ∷ _) = x

tail-v : ∀ {l} {A : Set l} {n : Nat} -> Vec A n -> Vec A (pred n)
tail-v [] = []
tail-v (_ ∷ xs) = xs

map-v : ∀ {l l'} {A : Set l} {A' : Set l'} {n : Nat} -> (A -> A') -> Vec A n -> Vec A' n
map-v f [] = []
map-v f (x ∷ xs) = (f x) ∷ map-v f xs

concat-v : ∀ {l} {A : Set l} {n m : Nat} -> Vec (Vec A n) m -> Vec A (m * n)
concat-v [] = []
concat-v (x ∷ xs) = x ++v (concat-v xs)

nth-v : ∀ {l} {A : Set l} {m : Nat} -> (n : Nat) -> (n < m) ≡ true -> Vec A m -> A
nth-v zero p (x ∷ _) = x
nth-v (suc n) p (_ ∷ xs) = nth-v n p xs

repeat-v : ∀ {l} {A : Set l} -> (a : A) (n : Nat) -> Vec A n
repeat-v a 0 = []
repeat-v a (suc m) = a ∷ repeat-v a m

module relations {l l' : Level} {A : Set l} (_>=A_ : A -> A -> Set l') where
  reflexive : Set (l ⊔ l')
  reflexive = ∀ {a : A} -> a >=A a

  transitive : Set (l ⊔ l')
  transitive = ∀ {a b c : A} -> a >=A b -> b >=A c -> a >=A c

module bool-relations {l : Level} {A : Set l} (_<=A_ : A -> A -> Bool) where
  open relations (λ a a' -> a <=A a' ≡ true) public using (reflexive; transitive)

  total : Set l
  total = ∀ {a b : A} -> a <=A b ≡ false -> b <=A a ≡ true

  total-reflexive : total -> reflexive
  total-reflexive tot {a} with (a <=A a) | inspect (λ x -> x <=A x) a
  total-reflexive tot {a} | true | [ p ] = refl
  total-reflexive tot {a} | false | [ p ] rewrite sym (tot p) | sym p = refl

  _isoB_ : A -> A -> Bool
  d isoB d' = d <=A d' && d' <=A d

  isoB-intro : ∀ {x y : A} -> x <=A y ≡ true -> y <=A x ≡ true -> x isoB y ≡ true
  isoB-intro p1 p2 rewrite p1 | p2 = refl

module minmax where

module bst (A : Set) (_<=A_ : A -> A -> Bool) (<=A-trans : bool-relations.transitive _<=A_) (<=A-total : bool-relations.total _<=A_) where
  open bool-relations _<=A_ hiding (transitive; total)
  --open minmax _<=A_ <=A-trans <=A-total
  data bst : A -> A -> Set where
    bst-leaf : ∀ {l u : A} -> l <=A u ≡ true -> bst l u
    bst-node : ∀ {l l' u' u : A} (d : A) -> bst l' d -> bst d u' -> l <=A l' ≡ true -> u' <=A u ≡ true -> bst l u

  bst-search : ∀ {l u : A} (d : A) -> bst l u -> Maybe (Σ A (λ d' -> d isoB d' ≡ true))
  bst-search d (bst-leaf p) = nothing
  bst-search d (bst-node d' L R _ _) with d <=A d' | inspect (_<=A_ d) d'
  bst-search d (bst-node d' L R _ _) | true | [ p1 ] with d' <=A d | inspect (_<=A_ d') d
  bst-search d (bst-node d' L R _ _) | true | [ p1 ] | true | [ p2 ] = just (d' , isoB-intro p1 p2)
  bst-search d (bst-node d' L R _ _) | true | [ p1 ] | false | [ p2 ] = bst-search d L
  bst-search d (bst-node d' L R _ _) | false | [ p1 ] = bst-search d R
