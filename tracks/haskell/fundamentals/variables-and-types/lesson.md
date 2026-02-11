# Variables and Types — Haskell

## The Haskell Mindset

Haskell is a fundamentally different beast from everything else in this curriculum. A few ground rules before we start:

- **Everything is immutable.** There is no `var`, no reassignment, no mutation. A "variable" in Haskell is a name bound to a value, forever. If you want a different value, you create a new binding.
- **Everything is an expression.** There are no statements. `if/else` returns a value. `let` returns a value. Even a function definition is an expression.
- **Functions are the core building block.** Not classes, not structs, not objects. Functions that take values and return values. That's it.
- **Types are inferred but powerful.** Haskell's type system is far stronger than Go, TypeScript, or Zig. It can express things those type systems can't.

Coming from JS/TS: imagine if `const` was the only keyword, everything was an expression, and the type system was 10x more powerful. That's Haskell.

Coming from Go/Zig: imagine giving up all control over memory layout in exchange for a type system that catches entire categories of bugs at compile time and a language that's extremely concise.

---

## Bindings, Not Variables

### Let and Where

Haskell doesn't have variables in the imperative sense. It has **bindings** — names attached to values.

```haskell
-- Top-level binding (visible in the whole module)
greeting :: String
greeting = "hello"

-- greeting = "world"   -- ERROR: multiple declarations of 'greeting'
```

You can't reassign. Ever. This is not like JS's `const` where the binding is fixed but the value might be mutable (objects). In Haskell, the value itself is immutable too. There's nothing to mutate.

Inside functions, you use `let...in` or `where`:

```haskell
circleArea :: Double -> Double
circleArea radius =
    let pi = 3.14159
        radiusSquared = radius * radius
    in  pi * radiusSquared

-- Or equivalently with where:
circleArea' :: Double -> Double
circleArea' radius = pi * radiusSquared
  where
    pi = 3.14159
    radiusSquared = radius * radius
```

`let...in` defines bindings and then uses them. `where` puts the bindings after the expression. Same result, different reading order. `where` reads more like natural language ("the area is pi times r squared, where pi is 3.14...").

| | JS/TS | Go | Zig | Haskell |
|---|---|---|---|---|
| Immutable binding | `const` (shallow) | No direct equiv | `const` (deep) | Everything (no keyword needed) |
| Mutable binding | `let` | `var` / `:=` | `var` | Doesn't exist |
| Reassignment | `let` only | Yes | `var` only | Never |

### Your notes
<!-- -->


---

## Type System

### Static, Inferred, Strong — Extremely Strong

Haskell's type system is:
- **Static**: all types known at compile time
- **Inferred**: you almost never need to write type annotations (but you should for top-level definitions — it's good documentation)
- **Strong**: no implicit conversions whatsoever
- **Parametrically polymorphic**: generics are the default, not the exception

```haskell
x = 42          -- Haskell infers: x :: Num a => a (polymorphic!)
y = 42 :: Int   -- explicitly Int
z = 42 :: Double -- explicitly Double

-- These are different types and cannot be mixed:
-- y + z         -- ERROR: no instance for (Num Int) arising from (+) with Double
```

Notice something unusual: `x = 42` without a type annotation doesn't infer a concrete type like `Int`. Haskell infers the most general type: `Num a => a`, meaning "any numeric type." The concrete type is decided when `x` is actually used. This is called **polymorphism** and it's pervasive in Haskell.

### Primitive Types

| Type | Description | Examples | Equivalent in Go/TS |
|---|---|---|---|
| `Int` | Fixed-precision integer (at least 30 bits, usually 64-bit) | `42`, `-7` | `int` / `number` |
| `Integer` | Arbitrary-precision integer (unlimited) | `2^100` | `big.Int` / `BigInt` |
| `Float` | Single-precision float | `3.14` | `float32` / — |
| `Double` | Double-precision float | `3.14` | `float64` / `number` |
| `Char` | Single Unicode character | `'a'`, `'λ'` | `rune` / — |
| `Bool` | Boolean | `True`, `False` | `bool` / `boolean` |
| `()` | Unit (like void) | `()` | — / `void` |

Key differences from other languages:
- **`Int` vs `Integer`**: `Int` is machine-sized (fast, can overflow). `Integer` is arbitrary precision (slower, never overflows). Most languages only give you one or the other.
- **`Char`** is a full Unicode codepoint, not a byte. `'λ'` is a valid `Char`.
- **`()`** is the unit type — it has exactly one value, also written `()`. It's used where other languages use `void`.

### Type Annotations

Type annotations go on a separate line above the definition, using `::` ("has type"):

```haskell
age :: Int
age = 30

name :: String        -- String is an alias for [Char]
name = "hello"

isValid :: Bool
isValid = True

-- Functions
add :: Int -> Int -> Int
add x y = x + y
```

The `->` in function types reads as "takes X and returns Y." `Int -> Int -> Int` means "takes an Int, then another Int, and returns an Int." (This is actually currying — we'll cover it in functions-and-closures.)

### Your notes
<!-- -->


---

## Strings and Characters

### String = [Char]

In Haskell, `String` is a type alias for `[Char]` — a linked list of characters. This is elegant but **slow** for real work.

```haskell
greeting :: String        -- same as [Char]
greeting = "hello"

-- A string is literally a list of characters:
-- "hello" == ['h', 'e', 'l', 'l', 'o']

firstChar :: Char
firstChar = head greeting   -- 'h'

len :: Int
len = length greeting       -- 5

-- String concatenation with ++
full :: String
full = "hello" ++ " " ++ "world"
```

| | JS/TS | Go | Zig | Haskell |
|---|---|---|---|---|
| String type | `string` (UTF-16) | `string` (bytes) | `[]const u8` (bytes) | `String` = `[Char]` (linked list) |
| Character type | None (single-char string) | `rune` (int32) | `u8` (byte) | `Char` (Unicode codepoint) |
| Performance | Good | Good | Great | Poor (linked list) |
| Real-world alternative | — | — | — | `Text` (from `Data.Text`) |

For production Haskell, you'd use `Data.Text` (packed UTF-16) or `Data.ByteString` (raw bytes). But `String` is fine for learning and is what the standard library uses.

### Your notes
<!-- -->


---

## Lists

Lists are the fundamental collection in Haskell. They're **singly-linked lists**, not arrays.

```haskell
numbers :: [Int]
numbers = [1, 2, 3, 4, 5]

-- The : operator (cons) prepends an element
moreNumbers :: [Int]
moreNumbers = 0 : numbers   -- [0, 1, 2, 3, 4, 5]

-- ++ concatenates two lists
combined :: [Int]
combined = [1, 2] ++ [3, 4]  -- [1, 2, 3, 4]

-- Common operations
hd = head numbers      -- 1 (first element)
tl = tail numbers      -- [2, 3, 4, 5] (everything after first)
ln = length numbers    -- 5
isEmpty = null numbers -- False
rev = reverse numbers  -- [5, 4, 3, 2, 1]
```

### List Comprehensions

Haskell has list comprehensions similar to Python:

```haskell
evens :: [Int]
evens = [x | x <- [1..20], even x]   -- [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]

pairs :: [(Int, Int)]
pairs = [(x, y) | x <- [1..3], y <- [1..3], x /= y]
-- [(1,2), (1,3), (2,1), (2,3), (3,1), (3,2)]
```

### Infinite Lists (Lazy Evaluation Preview)

Because Haskell is lazy, lists can be infinite:

```haskell
nats :: [Int]
nats = [1..]              -- infinite list: 1, 2, 3, 4, ...

firstTen :: [Int]
firstTen = take 10 nats   -- [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

-- This doesn't crash because Haskell only evaluates what's needed
```

This is impossible in Go, Zig, or TypeScript (without generators). Haskell's laziness means values aren't computed until they're actually used. We'll cover this in depth in the lazy-evaluation module.

| | JS/TS array | Go slice | Zig array/slice | Haskell list |
|---|---|---|---|---|
| Structure | Dynamic array | Dynamic array (ptr+len+cap) | Fixed array / view | Singly-linked list |
| Indexing | O(1) | O(1) | O(1) | O(n) |
| Prepend | O(n) | O(n) | N/A | O(1) |
| Append | O(1) amortized | O(1) amortized | N/A | O(n) |
| Lazy? | No | No | No | Yes |
| Infinite? | No | No | No | Yes |

### Your notes
<!-- -->


---

## Tuples

Tuples are fixed-size collections of different types. Unlike lists (which are homogeneous and variable-length), tuples are heterogeneous and fixed-length.

```haskell
point :: (Double, Double)
point = (3.0, 4.0)

person :: (String, Int, Bool)
person = ("Alice", 30, True)

-- Access with fst and snd (only for 2-tuples)
x = fst point    -- 3.0
y = snd point    -- 4.0

-- For larger tuples, use pattern matching
getName :: (String, Int, Bool) -> String
getName (name, _, _) = name
```

Tuples are closer to Go's multiple return values than to any JS concept:

```go
// Go
func divide(a, b int) (int, error) { ... }
```
```haskell
-- Haskell
divide :: Int -> Int -> (Int, Int)  -- returns (quotient, remainder)
divide a b = (a `div` b, a `mod` b)
```

### Your notes
<!-- -->


---

## Algebraic Data Types (Preview)

This gets its own module, but you'll see it everywhere so here's the core idea. Haskell lets you define custom types with `data`:

### Sum Types (OR — like enums)

```haskell
data Direction = North | South | East | West
  deriving (Show)   -- auto-generates string representation

data Color = Red | Green | Blue
  deriving (Show, Eq)   -- Show + equality comparison
```

This is like Zig's `enum` or TypeScript's union types. A `Direction` is one of four values.

### Product Types (AND — like structs)

```haskell
data Point = Point Double Double
  deriving (Show)

data Person = Person
  { personName :: String
  , personAge  :: Int
  , personActive :: Bool
  } deriving (Show)

-- Create a Person
alice :: Person
alice = Person { personName = "Alice", personAge = 30, personActive = True }

-- Access fields
aliceName = personName alice   -- "Alice"
```

Record syntax (with `{ }`) gives you accessor functions for free. `personName` is a function `Person -> String`.

### Sum + Product (the real power)

```haskell
data Shape
  = Circle Double              -- radius
  | Rectangle Double Double    -- width, height
  | Triangle Double Double Double  -- three sides
  deriving (Show)

area :: Shape -> Double
area (Circle r) = pi * r * r
area (Rectangle w h) = w * h
area (Triangle a b c) =                    -- Heron's formula
  let s = (a + b + c) / 2
  in  sqrt (s * (s - a) * (s - b) * (s - c))
```

This is like Zig's tagged unions or Rust's enums — each variant can carry different data, and pattern matching is exhaustive. But Haskell's version is more concise and deeply integrated into the language.

| | TS discriminated union | Go (interface + types) | Zig tagged union | Rust enum | Haskell data |
|---|---|---|---|---|---|
| Syntax | `type A = B \| C` | Interface + struct per variant | `union(enum)` | `enum A { B, C }` | `data A = B \| C` |
| Exhaustive? | With narrowing | No | Yes | Yes | Yes |
| Built-in? | Sort of | No | Yes | Yes | Yes |

### Your notes
<!-- -->


---

## Type Classes (Preview)

When you see `deriving (Show, Eq)`, those are **type classes** — Haskell's version of interfaces/traits. A type class defines behavior that types can implement.

```haskell
-- Show: can be converted to a String
-- Eq: can be compared with == and /=
-- Ord: can be ordered with <, >, <=, >=
-- Num: supports +, -, *, etc.
```

When Haskell infers `x = 42 :: Num a => a`, the `Num a =>` part is a **constraint** saying "the type `a` must implement the `Num` type class." This is like Go's interface constraints or Rust's trait bounds.

We'll cover type classes in depth in their own module.

### Your notes
<!-- -->


---

## Pattern Matching (Preview)

Pattern matching is how you destructure and branch in Haskell. It replaces if/else chains, switch statements, and manual field access:

```haskell
-- On function arguments
greet :: String -> String
greet "Alice" = "Hi, Alice!"
greet "Bob"   = "Hey, Bob!"
greet name    = "Hello, " ++ name

-- On lists
describeList :: [a] -> String
describeList []     = "empty"
describeList [_]    = "one element"
describeList [_,_]  = "two elements"
describeList _      = "many elements"

-- On tuples
addPair :: (Int, Int) -> Int
addPair (x, y) = x + y
```

This is the primary control flow mechanism in Haskell. We'll use it extensively in control-flow and beyond.

### Your notes
<!-- -->


---

## IO (Preview)

Haskell separates pure computation from side effects using the `IO` type. Printing to the screen, reading files, network calls — all wrapped in `IO`.

```haskell
main :: IO ()
main = do
    putStrLn "What is your name?"
    name <- getLine
    putStrLn ("Hello, " ++ name ++ "!")
```

The `do` notation makes IO look imperative, but under the hood it's a chain of function compositions. The type `IO ()` means "an IO action that produces unit (nothing useful)." `IO String` would mean "an IO action that produces a String."

This is the biggest mental shift from every other language in the curriculum. In Go, TS, Zig — functions can do IO whenever they want. In Haskell, a function's type tells you whether it can do IO. A function `Int -> Int` is *guaranteed* to be pure — no side effects, no network calls, no file writes. This is enforced by the compiler.

### Your notes
<!-- -->
