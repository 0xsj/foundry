# Variables and Types — Haskell

## The Big Shift: Bindings, Not Variables

In Go, a variable is a named storage location that you can read from and write to:

```go
x := 42
x = 100  // mutate — x now holds a different value
```

In Haskell, there are no variables. There are **bindings** — a name bound to a value, permanently. Once bound, it never changes.

```haskell
x = 42
-- x = 100  -- ERROR: Multiple declarations of 'x'
```

This isn't a limitation. It's the foundation of everything in Haskell. When `x = 42`, you can replace every occurrence of `x` with `42` and the program behaves identically. This property is called **referential transparency** — and it's why Haskell programs are easier to reason about, test, and parallelize.

### Mental Model

| | Go/TS | Haskell |
|---|---|---|
| `x = 42` | "Put 42 in the box labeled x" | "x is the name for the value 42" |
| Mutation | Change what's in the box | Not possible — define a new binding |
| Identity | x is a *location* that holds values | x *is* the value |

Think of Haskell bindings like `const` in JS/TS — except there's no `let` or `var` alternative. Everything is `const`, always.

### Your notes
<!-- -->

---

## Type System

### Static, Strong, Inferred

Haskell's type system is:
- **Static**: types checked at compile time (like Go, unlike Python)
- **Strong**: no implicit conversions (like Go, unlike JS)
- **Inferred**: you rarely need to write types — the compiler figures them out (unlike Go where `:=` infers but function signatures require types)

```haskell
x = 42          -- compiler infers: x :: Integer (by default)
y = 3.14        -- compiler infers: y :: Double
name = "hello"  -- compiler infers: name :: String (which is [Char])
flag = True     -- compiler infers: flag :: Bool
```

You *can* add type annotations, and it's considered good practice for top-level definitions:

```haskell
x :: Int
x = 42

greeting :: String
greeting = "hello"
```

The `::` reads as "has type". So `x :: Int` means "x has type Int".

### Hindley-Milner Type Inference

Haskell uses the **Hindley-Milner** type inference algorithm. It can figure out the types of almost everything without annotations. This is fundamentally more powerful than Go's `:=` or TypeScript's type inference:

```haskell
-- No type annotation needed. Compiler infers:
-- double :: Num a => a -> a
double x = x + x

-- Works with Int, Double, Integer, any numeric type
-- The constraint "Num a =>" means "for any type a that supports arithmetic"
```

In Go, you'd need to write separate functions or use generics. In Haskell, this is the default behavior — functions are polymorphic unless you constrain them.

### Your notes
<!-- -->

---

## Basic Types

### Numeric Types

| Type | What it is | Go equivalent | Notes |
|---|---|---|---|
| `Int` | Machine-width integer | `int` | At least 30 bits, typically 64 bits |
| `Integer` | Arbitrary-precision integer | `*big.Int` | No overflow, unlimited size |
| `Float` | 32-bit floating point | `float32` | Rarely used |
| `Double` | 64-bit floating point | `float64` | Default for decimal literals |

```haskell
smallNum :: Int
smallNum = 42

bigNum :: Integer
bigNum = 2^100  -- no overflow, this just works

pi' :: Double
pi' = 3.14159265358979
```

**Key difference from Go:** Haskell numeric literals are polymorphic. `42` isn't an `Int` or an `Integer` — it's `Num a => a`, meaning "any numeric type." The concrete type is determined by context:

```haskell
x = 42           -- Integer (default for integer literals)
y = 42 :: Int    -- Int (explicitly constrained)
z = 42 :: Double -- Double (yes, 42 becomes 42.0)
```

In Go, `42` is always `int` (or an untyped constant that defaults to `int`). In Haskell, `42` adapts. This is closer to Go's untyped constants, but more powerful.

### Other Primitive Types

| Type | What it is | Go equivalent | Literal example |
|---|---|---|---|
| `Bool` | Boolean | `bool` | `True`, `False` |
| `Char` | Single Unicode character | `rune` | `'a'`, `'λ'` |
| `String` | List of characters (`[Char]`) | `string` | `"hello"` |
| `()` | Unit (empty tuple) | `struct{}` | `()` |

### Strings: A Linked List of Characters

This is where Haskell gets weird compared to Go:

```haskell
-- String is defined as:
type String = [Char]

-- So "hello" is actually: ['h', 'e', 'l', 'l', 'o']
-- Which is a linked list: 'h' : 'e' : 'l' : 'l' : 'o' : []
```

| | Go | Haskell (String) |
|---|---|---|
| Internal structure | Pointer + length (16 bytes) | Linked list of Char |
| `len`/`length` | O(1) | **O(n)** — must traverse the list |
| Character access | `s[i]` is O(1) byte access | `s !! i` is **O(n)** |
| Memory efficiency | Compact byte array | ~24 bytes per character (list node overhead) |
| Immutable? | Yes | Yes |

This is why Haskell has `Text` (from the `text` package) for real programs — it's a packed UTF-16 array, much closer to Go's string. But for learning, `String` as `[Char]` is simpler and teaches list thinking.

```haskell
-- length is O(n) — it walks the linked list
length "hello"  -- 5

-- String operations are list operations
head "hello"    -- 'h'
tail "hello"    -- "ello"
"hello" ++ " world"  -- "hello world" (list concatenation)
```

### Your notes
<!-- -->

---

## Bindings and Scope

### Top-Level Bindings

```haskell
-- These are module-level bindings, visible everywhere in the file
port :: Int
port = 8080

host :: String
host = "localhost"
```

### Let Bindings (Local Scope)

```haskell
circleArea :: Double -> Double
circleArea radius =
  let pi' = 3.14159
      radiusSq = radius * radius
  in pi' * radiusSq
```

`let ... in ...` introduces local bindings. The names only exist within the `in` expression.

### Where Clauses

```haskell
circleArea :: Double -> Double
circleArea radius = pi' * radiusSq
  where
    pi' = 3.14159
    radiusSq = radius * radius
```

`where` does the same thing as `let`, but the definitions come after the expression. It's stylistic — `where` reads more naturally for top-down thinkers.

### Let vs Where

| | `let ... in` | `where` |
|---|---|---|
| Position | Before the expression | After the expression |
| Works in | Any expression | Function definitions, case expressions |
| Style | Bottom-up (define then use) | Top-down (use then define) |

Both are used in practice. Neither is "better."

### Your notes
<!-- -->

---

## Type Annotations and Signatures

### Function Signatures

```haskell
add :: Int -> Int -> Int
add x y = x + y
```

Read `Int -> Int -> Int` as: "takes an `Int`, then another `Int`, and returns an `Int`." The arrows separate parameters. The last type is always the return type.

This is actually **currying** — every function takes exactly one argument and returns a new function:

```haskell
add :: Int -> (Int -> Int)   -- same thing, parens are implicit
-- add 3 returns a function (Int -> Int) that adds 3 to its argument
```

This is fundamentally different from Go's `func add(x, y int) int`. In Haskell, `add 3` is a valid expression — it's a function that adds 3 to whatever you give it. In Go, `add(3)` is a compile error (wrong number of arguments).

### Type Variables (Generics)

```haskell
identity :: a -> a
identity x = x
```

Lowercase `a` is a **type variable** — it stands for any type. This is like Go's generics but built into the language from day one:

```go
// Go equivalent (since Go 1.18)
func Identity[T any](x T) T { return x }
```

In Haskell, generics aren't a feature — they're the default. You opt *out* of generics by specifying a concrete type.

### Type Constraints

```haskell
double :: Num a => a -> a
double x = x + x
```

`Num a =>` is a **constraint**: "for any type `a` that implements the `Num` type class." Type classes are like Go interfaces — they define a set of operations a type must support. We'll cover them in depth in the type-classes module.

For now: `Num` means "supports `+`, `-`, `*`, and a few others." `Eq` means "supports `==` and `/=`." `Ord` means "supports `<`, `>`, `<=`, `>=`."

### Your notes
<!-- -->

---

## Tuples

Tuples are fixed-size, heterogeneous collections — like a Go struct with no field names:

```haskell
point :: (Int, Int)
point = (3, 4)

person :: (String, Int, Bool)
person = ("Alice", 30, True)

-- Access with pattern matching (not indexing)
fst (3, 4)  -- 3 (only for 2-tuples)
snd (3, 4)  -- 4 (only for 2-tuples)

-- For larger tuples, use pattern matching
getName :: (String, Int, Bool) -> String
getName (name, _, _) = name
```

| | Go | Haskell |
|---|---|---|
| Multiple return | `func f() (int, error)` | Return a tuple: `f :: ... -> (Int, String)` |
| Access | Named fields or positional | Pattern matching |
| Heterogeneous? | Only in structs | Yes, tuples are naturally heterogeneous |

### Your notes
<!-- -->

---

## Lists

Lists are Haskell's most fundamental data structure. Every element must be the same type (unlike tuples).

```haskell
nums :: [Int]
nums = [1, 2, 3, 4, 5]

chars :: [Char]    -- same as String
chars = "hello"

empty :: [Int]
empty = []
```

### List Construction

```haskell
-- (:) is "cons" — prepend an element to a list
-- This is O(1)
1 : [2, 3]    -- [1, 2, 3]

-- [1, 2, 3] is syntactic sugar for:
1 : 2 : 3 : []

-- (++) concatenates two lists — O(n) in the left list
[1, 2] ++ [3, 4]  -- [1, 2, 3, 4]
```

### List Operations

```haskell
head [1, 2, 3]    -- 1 (first element — crashes on empty list!)
tail [1, 2, 3]    -- [2, 3] (everything except first)
length [1, 2, 3]  -- 3 (O(n) — linked list traversal)
null []            -- True (safe emptiness check)
reverse [1, 2, 3] -- [3, 2, 1]
take 2 [1, 2, 3]  -- [1, 2]
drop 2 [1, 2, 3]  -- [3]
```

### List Comparison to Go Slices

| | Go slice | Haskell list |
|---|---|---|
| Structure | Contiguous array | Linked list |
| Prepend | O(n) copy | **O(1)** cons |
| Append | Amortized O(1) | O(n) |
| Index access | O(1) | O(n) |
| Length | O(1) | O(n) |
| Immutable? | No | Yes — operations return new lists |

Haskell lists are great for sequential processing (map, filter, fold) but terrible for random access. For array-like performance, use `Data.Vector`.

### Your notes
<!-- -->

---

## Type Aliases and Newtype

### Type Synonyms (`type`)

```haskell
type Name = String
type Age = Int
type Person = (Name, Age)

greet :: Person -> String
greet (name, _) = "Hello, " ++ name
```

`type` creates an alias — `Name` and `String` are **the same type**. No safety gain, just readability. Like Go's `type Celsius = float64` (alias, not new type).

### Newtype

```haskell
newtype Celsius = Celsius Double
newtype Fahrenheit = Fahrenheit Double

-- These are DIFFERENT types. Can't accidentally mix them.
-- convert :: Celsius -> Fahrenheit  -- type-safe conversion
```

`newtype` creates a **distinct type** with zero runtime cost. It's erased at compile time. This is like Go's `type Celsius float64` (new type, not alias) — you can't pass a `Celsius` where `Fahrenheit` is expected.

| | Haskell | Go |
|---|---|---|
| Alias (same type) | `type Name = String` | `type Name = string` |
| New type (distinct) | `newtype Celsius = Celsius Double` | `type Celsius float64` |
| Runtime cost of new type | Zero | Zero |

### Your notes
<!-- -->

---

## No Zero Values — Maybe Instead

Go's design gives every type a zero value. Haskell takes the opposite approach: if a value might not exist, you must say so explicitly with `Maybe`.

```haskell
-- This always has a value
port :: Int
port = 8080

-- This might not have a value
configuredPort :: Maybe Int
configuredPort = Just 8080    -- has a value
-- or
-- configuredPort = Nothing   -- no value
```

| | Go | Haskell |
|---|---|---|
| Optional int | `*int` (nil or pointer to value) | `Maybe Int` (Nothing or Just value) |
| Missing string | `""` (is it empty or unset?) | `Maybe String` (Nothing is unambiguous) |
| Checking | `if p != nil` | Pattern matching on `Just x` / `Nothing` |
| Forgetting to check | Runtime nil panic | **Compile error** — you must handle both cases |

This is the core advantage: the compiler forces you to handle the missing case. In Go, forgetting a nil check is a runtime panic. In Haskell, it's a compile error.

```haskell
-- You MUST handle both cases
showPort :: Maybe Int -> String
showPort Nothing  = "no port configured"
showPort (Just p) = "port: " ++ show p

-- This won't compile — non-exhaustive pattern match warning:
-- showPort (Just p) = "port: " ++ show p
```

### Your notes
<!-- -->

---

## Putting It Together: Comparison Table

| Concept | Go | TypeScript | Haskell |
|---|---|---|---|
| Binding | `x := 42` | `const x = 42` | `x = 42` |
| Mutable | `var x = 42; x = 100` | `let x = 42; x = 100` | Not possible |
| Type annotation | `var x int = 42` | `const x: number = 42` | `x :: Int; x = 42` |
| Type inference | `:=` for locals | Most expressions | Everything (Hindley-Milner) |
| String type | `string` (byte slice header) | `string` (UTF-16) | `String` = `[Char]` (linked list) |
| Optional value | `*int` (nil pointer) | `number \| undefined` | `Maybe Int` |
| Zero value | Every type has one | `undefined` | No zero values — use `Maybe` |
| Generics | `func F[T any](x T) T` | `function f<T>(x: T): T` | `f :: a -> a` (default) |
| Multiple return | `(int, error)` | Tuple or object | Tuple: `(Int, String)` |

### Your notes
<!-- -->
