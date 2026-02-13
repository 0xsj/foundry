# Haskell Reference — Variables and Types

> Extracted from [Haskell 2010 Language Report](https://www.haskell.org/onlinereport/haskell2010/)
> and [GHC User's Guide](https://downloads.haskell.org/ghc/latest/docs/users_guide/)
> for the `variables-and-types` module. Covers: declarations, basic types, type
> annotations, lists, tuples, type synonyms, newtype, Maybe.

---

## Declarations and Bindings

Source: [Report §4](https://www.haskell.org/onlinereport/haskell2010/haskellch4.html)

### Top-Level Declarations

A module consists of a collection of declarations. Declarations define values, types,
classes, and other entities.

```haskell
-- Value binding
x = 42

-- Function binding
add x y = x + y

-- Type signature (optional, inferred if omitted)
add :: Int -> Int -> Int
```

### Where Clauses

`where` clauses introduce local declarations scoped to the enclosing binding:

```haskell
f x = y + z
  where
    y = x * 2
    z = x * 3
```

### Let Expressions

`let` expressions introduce local declarations scoped to the `in` expression:

```haskell
f x = let y = x * 2
          z = x * 3
      in y + z
```

### Pattern Bindings

A pattern binding binds variables by matching a pattern against an expression:

```haskell
(x, y) = (1, 2)        -- x = 1, y = 2
(a:bs) = [1, 2, 3]     -- a = 1, bs = [2, 3]
```

---

## Type Expressions

Source: [Report §4.1](https://www.haskell.org/onlinereport/haskell2010/haskellch4.html#x10-620004.1)

### Type Signatures

```haskell
-- Variable type signature
x :: Int

-- Function type signature
map :: (a -> b) -> [a] -> [b]

-- Constrained type signature
sort :: Ord a => [a] -> [a]
```

The function arrow `->` is **right-associative**:

```haskell
a -> b -> c    -- is the same as:
a -> (b -> c)  -- a function that returns a function
```

### Type Variables

- Lowercase identifiers in type expressions are **type variables**: `a`, `b`, `elem`
- Uppercase identifiers are **concrete types**: `Int`, `Bool`, `String`
- Type variables are implicitly universally quantified:

```haskell
id :: a -> a
-- means: forall a. a -> a (for any type a)
```

### Type Constraints (Context)

```haskell
-- Single constraint
show :: Show a => a -> String

-- Multiple constraints
compare :: (Eq a, Ord a) => a -> a -> Ordering
```

The `=>` separates constraints from the type. Read `Show a =>` as "given that `a` is an instance of `Show`."

---

## Basic Types

Source: [Report §6.1](https://www.haskell.org/onlinereport/haskell2010/haskellch6.html)

### Bool

```haskell
data Bool = False | True
```

- Two values: `True` and `False`
- Operators: `&&` (and), `||` (or), `not`
- Instance of: `Eq`, `Ord`, `Show`, `Read`, `Enum`, `Bounded`

### Char

```haskell
-- Single Unicode character
'a' :: Char
'λ' :: Char
'\n' :: Char    -- newline
'\97' :: Char   -- 'a' by decimal code
```

- Instance of: `Eq`, `Ord`, `Show`, `Read`, `Enum`, `Bounded`
- Operations in `Data.Char`: `isAlpha`, `isDigit`, `toUpper`, `toLower`, `ord`, `chr`

### String

```haskell
type String = [Char]
```

- A `String` is a type synonym for `[Char]` — a list of characters
- String literals use double quotes: `"hello"`
- All list operations apply: `length`, `head`, `tail`, `++`, `map`, etc.
- For performance-critical code, use `Data.Text` from the `text` package

### Numeric Types

#### Integer Types

```haskell
-- Fixed-width, at least [-2^29 .. 2^29-1], typically 64-bit
42 :: Int

-- Arbitrary precision, no overflow
2^100 :: Integer
```

- `Int`: bounded, machine-width. Instance of `Bounded` (`minBound`, `maxBound`)
- `Integer`: unbounded, arbitrary precision. Not an instance of `Bounded`

#### Floating-Point Types

```haskell
3.14 :: Float     -- 32-bit IEEE 754
3.14 :: Double    -- 64-bit IEEE 754 (default)
```

#### Numeric Literals

Numeric literals are **polymorphic**:

```haskell
42    :: Num a => a        -- any numeric type
3.14  :: Fractional a => a -- any fractional type
```

The default rules (in GHCi and with `default` declarations):
- Ambiguous integer defaults to `Integer`
- Ambiguous fractional defaults to `Double`

### Unit Type

```haskell
() :: ()
```

- Has exactly one value: `()`
- Used where Go would use `struct{}` or a void return
- Analogous to "void" but it's a real value you can pass around

---

## Tuple Types

Source: [Report §3.8](https://www.haskell.org/onlinereport/haskell2010/haskellch3.html#x8-360003.8)

```haskell
(1, 'a')           :: (Int, Char)        -- 2-tuple (pair)
(1, 'a', True)     :: (Int, Char, Bool)  -- 3-tuple (triple)
(1, 'a', True, []) :: (Int, Char, Bool, [a])  -- 4-tuple
```

- Tuples are **heterogeneous** — elements can have different types
- Size is part of the type — `(Int, Int)` and `(Int, Int, Int)` are different types
- Standard functions for pairs: `fst :: (a, b) -> a`, `snd :: (a, b) -> b`
- Larger tuples require pattern matching for access
- Maximum tuple size is implementation-dependent (GHC supports up to 62-tuples)

---

## List Types

Source: [Report §3.7](https://www.haskell.org/onlinereport/haskell2010/haskellch3.html#x8-340003.7)

```haskell
[1, 2, 3]    :: [Int]       -- list of Int
"hello"       :: [Char]      -- list of Char (i.e., String)
[]            :: [a]         -- empty list (polymorphic)
```

### Construction

```haskell
(:)  :: a -> [a] -> [a]     -- cons: prepend element
(++) :: [a] -> [a] -> [a]   -- append: concatenate lists
```

- `[1, 2, 3]` is syntactic sugar for `1 : 2 : 3 : []`
- Lists are **homogeneous** — all elements same type
- Lists are singly-linked — O(1) prepend, O(n) append, O(n) length

### List Comprehensions

```haskell
[x * 2 | x <- [1..5]]                    -- [2, 4, 6, 8, 10]
[x * 2 | x <- [1..10], even x]           -- [4, 8, 12, 16, 20]
[(x, y) | x <- [1..3], y <- ['a'..'c']]  -- all combinations
```

### Arithmetic Sequences

```haskell
[1..5]      -- [1, 2, 3, 4, 5]
[1,3..10]   -- [1, 3, 5, 7, 9]
[1..]       -- infinite list: [1, 2, 3, ...] (lazy evaluation)
```

### Common Operations

| Function | Type | Description |
|---|---|---|
| `head` | `[a] -> a` | First element (partial — crashes on `[]`) |
| `tail` | `[a] -> [a]` | All but first (partial) |
| `last` | `[a] -> a` | Last element (partial) |
| `init` | `[a] -> [a]` | All but last (partial) |
| `length` | `[a] -> Int` | Length (O(n)) |
| `null` | `[a] -> Bool` | Check if empty (safe) |
| `reverse` | `[a] -> [a]` | Reverse a list |
| `take` | `Int -> [a] -> [a]` | First n elements |
| `drop` | `Int -> [a] -> [a]` | Drop first n elements |
| `elem` | `Eq a => a -> [a] -> Bool` | Membership test |
| `zip` | `[a] -> [b] -> [(a,b)]` | Pair up elements |
| `map` | `(a -> b) -> [a] -> [b]` | Apply function to each |
| `filter` | `(a -> Bool) -> [a] -> [a]` | Keep matching elements |

---

## Type Declarations

Source: [Report §4.2](https://www.haskell.org/onlinereport/haskell2010/haskellch4.html#x10-680004.2)

### Type Synonyms (`type`)

```haskell
type String = [Char]
type Name = String
type Pair a = (a, a)
type AssocList k v = [(k, v)]
```

- Creates an **alias** — the synonym and the original are the same type
- Can be parameterized with type variables
- Cannot be recursive
- No runtime cost

### Newtype Declarations (`newtype`)

```haskell
newtype Age = Age Int
newtype Wrapper a = Wrap a
```

- Creates a **distinct type** from an existing type
- Must have exactly one constructor with exactly one field
- **Zero runtime overhead** — erased at compile time (unlike `data`)
- Can derive type class instances with `deriving`
- Used for type safety without performance cost

### Data Declarations (`data`) — Preview

```haskell
data Color = Red | Green | Blue              -- enumeration
data Maybe a = Nothing | Just a              -- parameterized
data Either a b = Left a | Right b           -- two type params
data List a = Nil | Cons a (List a)          -- recursive
```

- Creates new algebraic data types (covered in depth in algebraic-data-types module)
- Can have multiple constructors (sum types)
- Constructors can have fields (product types)
- Runtime overhead: tag + fields

---

## Maybe Type

Source: [Report §6.1.8](https://www.haskell.org/onlinereport/haskell2010/haskellch6.html)

```haskell
data Maybe a = Nothing | Just a
```

- `Nothing` represents absence of a value
- `Just x` wraps a present value
- Instance of: `Eq`, `Ord`, `Show`, `Read`, `Functor`, `Monad`

### Common Operations

| Function | Type | Description |
|---|---|---|
| `maybe` | `b -> (a -> b) -> Maybe a -> b` | Default + transform |
| `fromMaybe` | `a -> Maybe a -> a` | Extract with default |
| `isJust` | `Maybe a -> Bool` | Check if `Just` |
| `isNothing` | `Maybe a -> Bool` | Check if `Nothing` |
| `fromJust` | `Maybe a -> a` | Extract (partial — crashes on `Nothing`) |

```haskell
fromMaybe 0 (Just 42)  -- 42
fromMaybe 0 Nothing     -- 0

maybe "none" show (Just 42)  -- "42"
maybe "none" show Nothing    -- "none"
```

---

## Type Classes (Relevant Subset)

Source: [Report §6.3](https://www.haskell.org/onlinereport/haskell2010/haskellch6.html#x13-1260006.3)

Type classes relevant to basic types (full coverage in type-classes module):

| Class | Methods | Instances |
|---|---|---|
| `Eq` | `==`, `/=` | All basic types |
| `Ord` | `<`, `>`, `<=`, `>=`, `compare` | All basic types |
| `Show` | `show` | All basic types (converts to String) |
| `Read` | `read` | All basic types (parses from String) |
| `Num` | `+`, `-`, `*`, `abs`, `signum`, `fromInteger` | `Int`, `Integer`, `Float`, `Double` |
| `Integral` | `div`, `mod`, `toInteger` | `Int`, `Integer` |
| `Fractional` | `/`, `fromRational` | `Float`, `Double` |
| `Enum` | `succ`, `pred`, `toEnum`, `fromEnum` | `Int`, `Char`, `Bool`, etc. |
| `Bounded` | `minBound`, `maxBound` | `Int`, `Char`, `Bool` (not `Integer`) |

### Deriving

```haskell
data Color = Red | Green | Blue
  deriving (Show, Eq, Ord, Enum, Bounded)

show Red         -- "Red"
Red == Green     -- False
Red < Blue       -- True (by declaration order)
[Red ..]         -- [Red, Green, Blue]
minBound :: Color -- Red
```

---

## Conversions Between Numeric Types

There are no implicit numeric conversions. Use explicit functions:

| From | To | Function |
|---|---|---|
| `Int` → `Integer` | `toInteger` |
| `Integer` → `Int` | `fromInteger` (may overflow) |
| `Int` → `Double` | `fromIntegral` |
| `Double` → `Int` | `round`, `floor`, `ceiling`, `truncate` |
| Any integral → any numeric | `fromIntegral` |
| Any fractional → any fractional | `realToFrac` |

```haskell
fromIntegral (42 :: Int) :: Double    -- 42.0
round (3.7 :: Double) :: Int          -- 4
truncate (3.7 :: Double) :: Int       -- 3
```

---

## GHCi Quick Reference

```
ghci                    -- start the REPL
:load File.hs           -- load a file (or :l)
:reload                 -- reload after edits (or :r)
:type expr              -- show the type of an expression (or :t)
:info TypeOrClass       -- show info about a type or class (or :i)
:quit                   -- exit (or :q)
```

```haskell
> :t 42
42 :: Num a => a

> :t "hello"
"hello" :: String

> :t head
head :: [a] -> a

> :i Maybe
type Maybe :: * -> *
data Maybe a = Nothing | Just a
```
