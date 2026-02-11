# Haskell Reference — Variables and Types

> Extracted from [Haskell 2010 Language Report](https://www.haskell.org/onlinereport/haskell2010/) and [GHC User's Guide](https://downloads.haskell.org/ghc/latest/docs/users_guide/)
> for the `variables-and-types` module. Covers: bindings, basic types, lists, tuples, algebraic data types, type annotations.

---

## Declarations and Bindings

Source: [Haskell Report — Declarations](https://www.haskell.org/onlinereport/haskell2010/haskellch4.html)

### Top-Level Bindings

```haskell
-- Type signature (optional but recommended)
name :: Type
name = expression
```

Top-level bindings are immutable and visible throughout the module. Multiple bindings with the same name in the same scope are a compile error.

### Let Expressions

```haskell
let x = 5
    y = 10
in  x + y
```

Introduces local bindings. Bindings are mutually recursive (can reference each other).

### Where Clauses

```haskell
f x = a + b
  where
    a = x * 2
    b = x * 3
```

Attaches local bindings to an equation. Scoped to the equation.

---

## Basic Types

Source: [Haskell Report — Predefined Types](https://www.haskell.org/onlinereport/haskell2010/haskellch6.html)

### Numeric Types

| Type | Description | Range | Literal Examples |
|---|---|---|---|
| `Int` | Fixed-precision integer | At least [-2^29, 2^29-1], typically 64-bit | `42`, `-7`, `0xFF` |
| `Integer` | Arbitrary-precision integer | Unlimited | `2^100`, `factorial 1000` |
| `Float` | IEEE 754 single precision | ~7 decimal digits | `3.14`, `1.0e-5` |
| `Double` | IEEE 754 double precision | ~15 decimal digits | `3.14`, `1.0e-5` |
| `Rational` | Exact rational numbers | Unlimited (from `Data.Ratio`) | `3 % 4` |

### Other Primitives

| Type | Values | Notes |
|---|---|---|
| `Bool` | `True`, `False` | Capitalized (they're data constructors) |
| `Char` | `'a'`, `'λ'`, `'\n'` | Full Unicode codepoint |
| `()` | `()` | Unit type — one value |

### Numeric Conversions

No implicit conversions between numeric types:

| Function | From | To |
|---|---|---|
| `fromIntegral` | Any integral type | Any numeric type |
| `toInteger` | Any integral type | `Integer` |
| `fromInteger` | `Integer` | Any numeric type |
| `realToFrac` | Any real type | Any fractional type |
| `round`, `floor`, `ceiling`, `truncate` | Fractional | Integral |

```haskell
x :: Int
x = 42

y :: Double
y = fromIntegral x   -- 42.0

z :: Integer
z = toInteger x      -- 42
```

---

## Strings

Source: [Haskell Report — Characters and Strings](https://www.haskell.org/onlinereport/haskell2010/haskellch6.html#x13-1190006.1.2)

`String` is a type alias:

```haskell
type String = [Char]
```

### String Literals

```haskell
"hello"           -- :: String (i.e., [Char])
"hello" ++ " world"  -- concatenation
```

### Escape Sequences

| Escape | Meaning |
|---|---|
| `\n` | Newline |
| `\t` | Tab |
| `\\` | Backslash |
| `\"` | Double quote |
| `\97` | Character by decimal code |
| `\x61` | Character by hex code |

### Common String Functions

| Function | Type | Description |
|---|---|---|
| `length` | `[a] -> Int` | Number of elements |
| `head` | `[a] -> a` | First element (partial — crashes on empty) |
| `tail` | `[a] -> [a]` | All but first (partial) |
| `++` | `[a] -> [a] -> [a]` | Concatenation |
| `show` | `Show a => a -> String` | Convert to string |
| `read` | `Read a => String -> a` | Parse from string |
| `words` | `String -> [String]` | Split on whitespace |
| `unwords` | `[String] -> String` | Join with spaces |
| `lines` | `String -> [String]` | Split on newlines |
| `unlines` | `[String] -> String` | Join with newlines |

---

## Lists

Source: [Haskell Report — Lists](https://www.haskell.org/onlinereport/haskell2010/haskellch3.html#x8-340003.7)

### Syntax

```haskell
[]              -- empty list
[1, 2, 3]      -- list literal
1 : [2, 3]     -- cons (prepend): [1, 2, 3]
1 : 2 : 3 : [] -- equivalent to [1, 2, 3]
[1..10]         -- range: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
[1,3..10]       -- range with step: [1, 3, 5, 7, 9]
[1..]           -- infinite range
```

### Common List Functions

| Function | Type | Description |
|---|---|---|
| `head` | `[a] -> a` | First element |
| `tail` | `[a] -> [a]` | All but first |
| `last` | `[a] -> a` | Last element |
| `init` | `[a] -> [a]` | All but last |
| `length` | `[a] -> Int` | Number of elements |
| `null` | `[a] -> Bool` | Is empty? |
| `reverse` | `[a] -> [a]` | Reverse |
| `take` | `Int -> [a] -> [a]` | First n elements |
| `drop` | `Int -> [a] -> [a]` | Remove first n elements |
| `elem` | `Eq a => a -> [a] -> Bool` | Membership test |
| `zip` | `[a] -> [b] -> [(a,b)]` | Pair up elements |
| `map` | `(a -> b) -> [a] -> [b]` | Apply function to each |
| `filter` | `(a -> Bool) -> [a] -> [a]` | Keep matching elements |
| `foldl` | `(b -> a -> b) -> b -> [a] -> b` | Left fold (reduce) |
| `foldr` | `(a -> b -> b) -> b -> [a] -> b` | Right fold |
| `sum` | `Num a => [a] -> a` | Sum of elements |
| `product` | `Num a => [a] -> a` | Product of elements |
| `maximum` | `Ord a => [a] -> a` | Largest element |
| `minimum` | `Ord a => [a] -> a` | Smallest element |

### List Comprehensions

```haskell
[expression | generator, ..., guard, ...]

-- Examples:
[x * 2 | x <- [1..10]]                    -- [2, 4, 6, ..., 20]
[x * 2 | x <- [1..10], x * 2 >= 12]      -- [12, 14, 16, 18, 20]
[(x, y) | x <- [1..3], y <- ['a'..'c']]   -- all pairs
```

---

## Tuples

Source: [Haskell Report — Tuples](https://www.haskell.org/onlinereport/haskell2010/haskellch3.html#x8-360003.8)

Fixed-size, heterogeneous collections:

```haskell
(1, "hello")        :: (Int, String)
(1, 2, 3)           :: (Int, Int, Int)
(True, 'a', 3.14)   :: (Bool, Char, Double)
```

### Tuple Functions (2-tuples only)

| Function | Type | Description |
|---|---|---|
| `fst` | `(a, b) -> a` | First element |
| `snd` | `(a, b) -> b` | Second element |
| `swap` | `(a, b) -> (b, a)` | Swap elements (from `Data.Tuple`) |

For tuples larger than 2, use pattern matching:

```haskell
third :: (a, b, c) -> c
third (_, _, z) = z
```

---

## Algebraic Data Types

Source: [Haskell Report — Data Types](https://www.haskell.org/onlinereport/haskell2010/haskellch4.html#x10-690004.2)

### data Declaration

```haskell
data TypeName = Constructor1 | Constructor2 Type1 Type2 | ...
```

### Nullary Constructors (Enumerations)

```haskell
data Bool = False | True
data Direction = North | South | East | West
data Ordering = LT | EQ | GT
```

### Constructors with Fields

```haskell
data Shape = Circle Double | Rectangle Double Double

-- Circle and Rectangle are constructor functions:
-- Circle :: Double -> Shape
-- Rectangle :: Double -> Double -> Shape
```

### Record Syntax

```haskell
data Person = Person
  { firstName :: String
  , lastName  :: String
  , age       :: Int
  } deriving (Show)

-- Creates accessor functions:
-- firstName :: Person -> String
-- lastName :: Person -> String
-- age :: Person -> Int

-- Record update syntax:
olderPerson = person { age = age person + 1 }
```

### Deriving

```haskell
data Color = Red | Green | Blue
  deriving (Show, Eq, Ord, Enum, Bounded, Read)
```

| Class | Provides |
|---|---|
| `Show` | `show :: a -> String` |
| `Read` | `read :: String -> a` |
| `Eq` | `==`, `/=` |
| `Ord` | `<`, `>`, `<=`, `>=`, `compare` |
| `Enum` | `succ`, `pred`, `[X..Y]` ranges |
| `Bounded` | `minBound`, `maxBound` |

---

## Type Annotations

Source: [Haskell Report — Type Signatures](https://www.haskell.org/onlinereport/haskell2010/haskellch4.html#x10-810004.4.1)

```haskell
-- Top-level (recommended)
name :: Type
name = value

-- Inline
value = (42 :: Int) + (3 :: Int)

-- Function types
add :: Int -> Int -> Int
add x y = x + y

-- Polymorphic
identity :: a -> a
identity x = x

-- Constrained
double :: Num a => a -> a
double x = x + x
```

### Reading Function Types

`->` is right-associative:

```
a -> b -> c   means   a -> (b -> c)
```

This means every function technically takes one argument and returns a new function. `add 3` returns a function `Int -> Int` that adds 3. This is **currying**.

---

## Type Aliases and Newtypes

### type (Alias)

```haskell
type String = [Char]         -- String is just another name for [Char]
type Name = String
type Age = Int
type Person = (Name, Age)
```

Aliases are interchangeable with the original type. No type safety — `Name` and `String` are the same type to the compiler.

### newtype (Wrapper)

```haskell
newtype Name = Name String
newtype Age = Age Int
```

`newtype` creates a distinct type that wraps another. `Name` is NOT the same as `String` — you can't use one where the other is expected. Zero runtime cost (the wrapper is erased at compile time).

### data vs newtype vs type

| | `data` | `newtype` | `type` |
|---|---|---|---|
| New type? | Yes | Yes | No (alias) |
| Multiple constructors? | Yes | No (exactly one) | N/A |
| Multiple fields? | Yes | No (exactly one) | N/A |
| Runtime cost? | Possible | Zero | Zero |
| Use case | General ADTs | Type safety wrapper | Readability |

---

## Common Prelude Functions (Type-Related)

| Function | Type | Description |
|---|---|---|
| `show` | `Show a => a -> String` | Convert to string representation |
| `read` | `Read a => String -> a` | Parse from string (partial — can crash) |
| `fromIntegral` | `(Integral a, Num b) => a -> b` | Convert between numeric types |
| `toInteger` | `Integral a => a -> Integer` | Convert to Integer |
| `fromInteger` | `Num a => Integer -> a` | Convert from Integer |
| `realToFrac` | `(Real a, Fractional b) => a -> b` | Convert real to fractional |
| `maxBound` | `Bounded a => a` | Maximum value for bounded type |
| `minBound` | `Bounded a => a` | Minimum value for bounded type |
| `succ` | `Enum a => a -> a` | Next value |
| `pred` | `Enum a => a -> a` | Previous value |
