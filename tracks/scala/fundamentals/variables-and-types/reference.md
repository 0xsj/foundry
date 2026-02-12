# Scala 3 Reference --- Variables and Types

> Extracted from the [Scala 3 Language Reference](https://docs.scala-lang.org/scala3/reference/)
> and the [Scala 3 Book](https://docs.scala-lang.org/scala3/book/introduction.html)
> for the `variables-and-types` module. Covers: value definitions, type hierarchy, literal types,
> numeric types, strings, Option, tuples, type inference, type aliases, opaque types, union types,
> intersection types, enums, case classes, pattern matching, conversions, and equality.

---

## Value Definitions

Source: [Scala 3 Reference -- Definitions](https://docs.scala-lang.org/scala3/reference/), [SLS 4.1-4.2](https://www.scala-lang.org/files/archive/spec/3.4/04-basic-definitions.html)

Scala provides three forms of value binding: `val`, `var`, and `lazy val`.

### `val` --- Immutable Binding

A `val` definition binds a name to a value. Once assigned, it cannot be reassigned. The right-hand side is evaluated eagerly at the point of definition.

```scala
val x: Int = 42
val name = "foundry"   // type inferred as String
val (a, b) = (1, 2)    // destructuring bind
```

- The type annotation is optional when the compiler can infer it.
- A `val` defined at the top level of an object or class becomes a field with a getter method.
- A `val` in a local scope is a stack-allocated local variable.

### `var` --- Mutable Binding

A `var` definition introduces a mutable variable. It can be reassigned after initialization.

```scala
var count: Int = 0
count = count + 1
var label = "untitled"
label = "config"
```

- At the class level, `var` generates both a getter and a setter method.
- Mutable state is discouraged in idiomatic Scala. Prefer `val` and immutable data structures.

### `lazy val` --- Lazy Evaluation

A `lazy val` defers evaluation of its right-hand side until the value is first accessed. The result is memoized (computed once, cached thereafter).

```scala
lazy val config: Map[String, String] = loadConfigFromDisk()
lazy val expensive = computeHeavyResult()
```

- Thread-safe: the Scala compiler generates synchronization to ensure the initializer runs exactly once, even under concurrent access.
- Useful for expensive computations, circular dependencies, and optional initialization.
- A `lazy val` incurs a small overhead on each access (a volatile read to check initialization status).

### Summary Table

| Form | Mutable | Evaluation | Reassignable | Use Case |
|---|---|---|---|---|
| `val` | No | Eager (at definition) | No | Default choice for bindings |
| `var` | Yes | Eager (at definition) | Yes | Mutable state when necessary |
| `lazy val` | No | Lazy (on first access) | No | Deferred/expensive computation |

---

## Type Hierarchy

Source: [Scala 3 Book -- Types and the Type System](https://docs.scala-lang.org/scala3/book/types-introduction.html), [Unified Types](https://docs.scala-lang.org/scala3/book/first-look-at-types.html)

Scala has a unified type hierarchy rooted at `Any`. Every value is an object; there are no primitive types at the language level (though the compiler optimizes value types to JVM primitives where possible).

```
                   Any
                  /   \
             AnyVal   AnyRef (= java.lang.Object)
            /  |  \       \
        Int  Double  ...   String, List, user classes ...
            \   |   /         /
             \  |  /         /
              \ | /         /
            Nothing -------/
```

### Top Types

| Type | Description |
|---|---|
| `Any` | Supertype of all types. Defines universal methods: `equals`, `hashCode`, `toString`, `isInstanceOf`, `asInstanceOf`. |
| `AnyVal` | Supertype of all value types (`Int`, `Double`, `Boolean`, `Char`, `Unit`, etc.). Instances are not heap-allocated by default (mapped to JVM primitives). |
| `AnyRef` | Supertype of all reference types. Alias for `java.lang.Object` on the JVM. All non-value types extend `AnyRef`. |
| `Matchable` | Scala 3 type that sits between `Any` and `AnyRef`/`AnyVal`. A value must be `Matchable` to be the scrutinee of a pattern match with type tests. Prevents matching on opaque or parametric types. |

### Bottom Types

| Type | Description |
|---|---|
| `Nothing` | Subtype of every type. Has no instances. Used as the return type of expressions that never complete normally (e.g., `throw`, `sys.error`, infinite loops). Also the element type of `Nil` (the empty `List[Nothing]`). |
| `Null` | Subtype of all reference types (`AnyRef`). Has a single value: `null`. In Scala 3 with `-Yexplicit-nulls`, `Null` is no longer a subtype of `AnyRef` unless opted in, making null safety more explicit. |

### Unit

`Unit` is a value type with exactly one instance: `()`. It is the return type of expressions evaluated solely for their side effects, analogous to `void` in C/Java.

```scala
def log(msg: String): Unit = println(msg)
val result: Unit = ()
```

- `Unit` is a subtype of `AnyVal`.
- Any expression can be discarded to `Unit`, though the compiler may warn about discarded non-Unit values.

---

## Literal Types

Source: [Scala 3 Reference -- Literal Types](https://docs.scala-lang.org/scala3/reference/new-types/literal-types.html), [SLS 1.3](https://www.scala-lang.org/files/archive/spec/3.4/01-lexical-syntax.html)

In Scala 3, literals can be used as types (singleton types). This is useful for type-level programming and refined APIs.

### Integer Literals

```scala
val dec: Int = 42
val hex: Int = 0xFF
val long: Long = 42L
val bin: Int = 0b10101010
```

- Decimal: sequence of digits `0-9` (no leading zero for non-zero literals).
- Hexadecimal: prefix `0x` or `0X`, digits `0-9` and `a-f`/`A-F`.
- Binary: prefix `0b` or `0B`, digits `0` and `1` (Scala 2.13+/Scala 3).
- Underscores allowed as separators: `1_000_000`, `0xFF_FF`.
- Suffix `L` or `l` for `Long` literals.

### Floating-Point Literals

```scala
val pi: Double = 3.14159
val avogadro: Double = 6.022e23
val precise: Float = 3.14f
```

- Default type is `Double`.
- Suffix `f` or `F` for `Float`.
- Suffix `d` or `D` for explicit `Double` (rarely used).
- Scientific notation: `1.5e10`, `2.5E-3`.

### Boolean Literals

```scala
val yes: Boolean = true
val no: Boolean = false
```

Two values only: `true` and `false`.

### Character Literals

```scala
val letter: Char = 'A'
val unicode: Char = '\u0041'
val newline: Char = '\n'
```

- A single character enclosed in single quotes.
- Escape sequences: `\b`, `\t`, `\n`, `\f`, `\r`, `\"`, `\'`, `\\`.
- Unicode escapes: `\uXXXX` where `XXXX` is a four-digit hex code.
- Represented as a 16-bit unsigned integer (UTF-16 code unit) on the JVM.

### String Literals

```scala
val greeting: String = "Hello, world"
val escaped: String = "line1\nline2"
val raw: String = """no \n escaping here"""
```

### Symbol Literals (Deprecated)

Symbol literals (`'foo`) were available in Scala 2 but are **deprecated in Scala 3**. Use plain strings or enums instead.

### Literal Singleton Types (Scala 3)

Literals can serve as types. This enables type-level constraints:

```scala
val x: 42 = 42           // x has singleton type 42
val s: "hello" = "hello" // s has singleton type "hello"
val t: true = true        // t has singleton type true

def port: 8080 = 8080

// Useful in APIs
def configure(host: "localhost" | "0.0.0.0", port: Int): Unit = ???
```

Literal types are subtypes of their corresponding value types (e.g., `42 <: Int`, `"hello" <: String`).

---

## Numeric Types

Source: [Scala 3 Book -- First Look at Types](https://docs.scala-lang.org/scala3/book/first-look-at-types.html), [scala.AnyVal](https://www.scala-lang.org/api/3.x/scala/AnyVal.html)

All numeric types extend `AnyVal` and are mapped to JVM primitives where possible.

### Integer Types

| Type | Size | Min | Max | JVM Primitive |
|---|---|---|---|---|
| `Byte` | 8-bit signed | -128 | 127 | `byte` |
| `Short` | 16-bit signed | -32,768 | 32,767 | `short` |
| `Int` | 32-bit signed | -2,147,483,648 | 2,147,483,647 | `int` |
| `Long` | 64-bit signed | -9,223,372,036,854,775,808 | 9,223,372,036,854,775,807 | `long` |

### Floating-Point Types

| Type | Size | Spec | JVM Primitive |
|---|---|---|---|
| `Float` | 32-bit | IEEE 754 single precision | `float` |
| `Double` | 64-bit | IEEE 754 double precision | `double` |

### Char (Numeric)

| Type | Size | Min | Max | JVM Primitive |
|---|---|---|---|---|
| `Char` | 16-bit unsigned | 0 (`\u0000`) | 65,535 (`\uFFFF`) | `char` |

`Char` is classified under `AnyVal` and participates in numeric widening conversions.

### Default Literal Types

| Literal Form | Inferred Type | Example |
|---|---|---|
| Integer without suffix | `Int` | `42` |
| Integer with `L` suffix | `Long` | `42L` |
| Decimal without suffix | `Double` | `3.14` |
| Decimal with `f` suffix | `Float` | `3.14f` |

### Numeric Widening (Implicit)

Scala performs implicit widening conversions where no precision is lost:

```
Byte -> Short -> Int -> Long -> Float -> Double
                 Char -> Int
```

```scala
val b: Byte = 42
val i: Int = b       // implicit widening: Byte -> Int
val l: Long = i      // implicit widening: Int -> Long
val d: Double = l    // implicit widening: Long -> Double
val c: Char = 'A'
val ci: Int = c      // implicit widening: Char -> Int
```

> **Note:** `Long -> Float` and `Long -> Double` may lose precision for very large values, but the conversion is still considered a widening in the JVM specification.

### BigInt and BigDecimal

For arbitrary-precision arithmetic, use `BigInt` and `BigDecimal` from `scala.math`:

```scala
val big: BigInt = BigInt("99999999999999999999999999999")
val precise: BigDecimal = BigDecimal("0.1") + BigDecimal("0.2")  // exactly 0.3
```

These are reference types (extend `AnyRef`), not value types.

---

## String Type

Source: [Scala 3 Book -- First Look at Types](https://docs.scala-lang.org/scala3/book/first-look-at-types.html), [java.lang.String](https://docs.oracle.com/en/java/javase/17/docs/api/java.base/java/lang/String.html)

`String` in Scala is `java.lang.String`. It is an immutable sequence of `Char` values (UTF-16 code units).

### String Interpolation

Scala provides three built-in string interpolators:

#### `s` interpolator --- String interpolation

```scala
val name = "world"
val greeting = s"Hello, $name"                  // "Hello, world"
val expr = s"1 + 1 = ${1 + 1}"                 // "1 + 1 = 2"
val nested = s"${if x > 0 then "pos" else "neg"}"
```

#### `f` interpolator --- Formatted interpolation (printf-style)

```scala
val pi = 3.14159
val formatted = f"Pi is $pi%.2f"                // "Pi is 3.14"
val padded = f"${42}%05d"                       // "00042"
```

Type-checked at compile time: format specifiers must match argument types.

#### `raw` interpolator --- No escape processing

```scala
val path = raw"C:\Users\new\test"               // backslashes not interpreted
```

### Multiline Strings

Triple-quoted strings preserve whitespace and newlines:

```scala
val sql = """
  |SELECT *
  |FROM users
  |WHERE active = true
  """.stripMargin

val json = """
  {
    "name": "foundry",
    "version": "1.0"
  }
"""
```

- `stripMargin` removes leading whitespace up to and including the `|` character (configurable).
- Triple-quoted strings can be combined with interpolators: `s"""..."""`, `f"""..."""`.

### Common Operations

| Operation | Example | Result |
|---|---|---|
| Length | `"hello".length` | `5` |
| Concatenation | `"a" + "b"` | `"ab"` |
| Char access | `"hello"(1)` | `'e'` |
| Substring | `"hello".substring(1, 3)` | `"el"` |
| Split | `"a,b,c".split(",")` | `Array("a", "b", "c")` |
| Trim | `" hi ".trim` | `"hi"` |
| Upper/Lower | `"Hi".toUpperCase` | `"HI"` |
| Contains | `"hello".contains("ell")` | `true` |
| Replace | `"aab".replace("a", "x")` | `"xxb"` |

---

## Boolean Type

Source: [scala.Boolean](https://www.scala-lang.org/api/3.x/scala/Boolean.html)

`Boolean` is a value type with two instances: `true` and `false`. Mapped to JVM `boolean`.

### Operations

| Operation | Syntax | Notes |
|---|---|---|
| Logical AND | `a && b` | Short-circuit evaluation |
| Logical OR | `a \|\| b` | Short-circuit evaluation |
| Logical NOT | `!a` | Unary negation |
| Bitwise AND | `a & b` | Evaluates both operands |
| Bitwise OR | `a \| b` | Evaluates both operands |
| Bitwise XOR | `a ^ b` | Exclusive or |
| Equality | `a == b` | Structural equality |

`Boolean` is used as the condition type in `if`, `while`, and `do-while` expressions.

---

## Unit Type

Source: [scala.Unit](https://www.scala-lang.org/api/3.x/scala/Unit.html)

`Unit` is a value type carrying no meaningful information. It has exactly one value, written `()`.

```scala
val u: Unit = ()
def fireAndForget(): Unit = sendNotification()
```

- Analogous to `void` in Java/C, but `Unit` is an actual type with an actual value.
- Any expression can be converted to `Unit` by discarding its value; the compiler may issue a warning with `-Wvalue-discard`.
- `Unit` is a subtype of `AnyVal`.

---

## Nothing and Null

Source: [Scala 3 Book -- First Look at Types](https://docs.scala-lang.org/scala3/book/first-look-at-types.html)

### Nothing

`Nothing` is the **bottom type** of the entire type hierarchy. It is a subtype of every other type. `Nothing` has no instances.

Uses:
- Return type for methods that never return normally: `throw`, `sys.error`, infinite loops.
- Type parameter for empty collections: `Nil` has type `List[Nothing]`, which is a subtype of `List[A]` for any `A` (due to covariance).
- The `???` method (placeholder for unimplemented code) has return type `Nothing`.

```scala
def fail(msg: String): Nothing = throw new RuntimeException(msg)
val empty: List[Nothing] = Nil
val todo: Int = ???          // compiles because Nothing <: Int
```

### Null

`Null` is a subtype of all reference types (`AnyRef` subtypes). It has exactly one value: `null`.

```scala
val s: String = null         // legal (but discouraged)
val n: Null = null
// val i: Int = null          // error: Null is not a subtype of Int (AnyVal)
```

**Scala 3 Explicit Nulls:** With the compiler flag `-Yexplicit-nulls`, `Null` is no longer a subtype of `AnyRef`. This makes null-safety part of the type system:

```scala
// With -Yexplicit-nulls:
val s: String = null         // error
val s: String | Null = null  // must explicitly opt in
```

---

## Option[A]

Source: [scala.Option](https://www.scala-lang.org/api/3.x/scala/Option.html), [Scala 3 Book -- Functional Error Handling](https://docs.scala-lang.org/scala3/book/fp-functional-error-handling.html)

`Option[A]` is a container type representing an optional value. It has exactly two subtypes:

| Subtype | Description | Example |
|---|---|---|
| `Some[A]` | Contains a value of type `A` | `Some(42)` |
| `None` | Represents absence of a value | `None` |

### Construction

```scala
val present: Option[Int] = Some(42)
val absent: Option[Int] = None
val fromNullable: Option[String] = Option(nullableJavaMethod())  // None if null
```

`Option.apply(x)` returns `None` if `x` is `null`, `Some(x)` otherwise. This is the standard bridge from Java nullable APIs.

### Core Methods

| Method | Signature | Description |
|---|---|---|
| `get` | `Option[A] => A` | Returns value or throws `NoSuchElementException`. Avoid. |
| `getOrElse` | `Option[A] => (=> A) => A` | Returns value or a default. |
| `orElse` | `Option[A] => (=> Option[A]) => Option[A]` | Returns self if non-empty, otherwise the alternative. |
| `map` | `Option[A] => (A => B) => Option[B]` | Transforms the contained value. |
| `flatMap` | `Option[A] => (A => Option[B]) => Option[B]` | Chains optional computations. |
| `filter` | `Option[A] => (A => Boolean) => Option[A]` | Returns `None` if predicate fails. |
| `fold` | `Option[A] => (=> B) => (A => B) => B` | Catamorphism: handle both cases. |
| `isEmpty` | `Option[A] => Boolean` | `true` if `None`. |
| `isDefined` | `Option[A] => Boolean` | `true` if `Some`. |
| `contains` | `Option[A] => A => Boolean` | `true` if contains the value. |
| `exists` | `Option[A] => (A => Boolean) => Boolean` | `true` if predicate holds. |
| `forall` | `Option[A] => (A => Boolean) => Boolean` | `true` if empty or predicate holds. |
| `toList` | `Option[A] => List[A]` | `Nil` or single-element list. |

### Idiomatic Usage

```scala
// Pattern matching
val msg = opt match
  case Some(value) => s"Got $value"
  case None        => "Nothing here"

// for-comprehension (monadic chaining)
val result: Option[String] = for
  user  <- findUser(id)
  email <- user.email
  domain <- extractDomain(email)
yield domain

// getOrElse for defaults
val port: Int = config.get("port").flatMap(_.toIntOption).getOrElse(8080)
```

---

## Tuple Types

Source: [Scala 3 Reference -- Tuples](https://docs.scala-lang.org/scala3/reference/other-new-features/tuples.html), [scala.Tuple](https://www.scala-lang.org/api/3.x/scala/Tuple.html)

Tuples are finite, ordered, heterogeneous sequences of values. Scala 3 extends tuple support beyond the 22-element limit of Scala 2.

### Syntax

```scala
val pair: (Int, String) = (1, "one")
val triple: (Int, String, Boolean) = (1, "one", true)
val unit: EmptyTuple = EmptyTuple   // 0-element tuple
```

`(A, B, C)` is syntactic sugar for `Tuple3[A, B, C]`.

### Element Access

```scala
val t = (42, "hello", true)
val first: Int = t._1          // 42 (1-indexed)
val second: String = t._2      // "hello"
val third: Boolean = t._3      // true

// Scala 3: also accessible by index with apply
val x: Int = t(0)              // 42 (0-indexed, requires import scala.language.implicitConversions)
```

### Destructuring

```scala
val (x, y, z) = (1, 2, 3)
val (head, tail) = (1, "rest")

// In match expressions
(1, "hello") match
  case (n, s) => s"$n: $s"
```

### Named Tuples (Scala 3.5+)

Scala 3.5 introduces named tuples where elements can be accessed by name:

```scala
type City = (name: String, population: Int, country: String)
val nyc: City = (name = "New York", population = 8_336_817, country = "US")

val n: String = nyc.name
val p: Int = nyc.population
```

Named tuples are structurally typed --- any tuple with the same names and types is compatible.

### Tuple Operations (Scala 3)

| Operation | Example | Result |
|---|---|---|
| Concatenation | `(1, 2) ++ (3, 4)` | `(1, 2, 3, 4)` |
| Size | `(1, 2, 3).size` | `3` |
| Head | `(1, 2, 3).head` | `1` |
| Tail | `(1, 2, 3).tail` | `(2, 3)` |
| Map | `(1, 2, 3).map([T] => (t: T) => ...)` | Polymorphic map |
| toList | `(1, 2, 3).toList` | `List(1, 2, 3)` (if homogeneous) |

---

## Type Inference

Source: [Scala 3 Reference -- Type Inference](https://docs.scala-lang.org/scala3/reference/), [SLS 6.26](https://www.scala-lang.org/files/archive/spec/3.4/06-expressions.html)

Scala uses **local type inference** based on bidirectional type checking and constraint solving. The compiler infers types from initializers, return expressions, and usage context.

### Where Inference Works

```scala
val x = 42                          // Int
val xs = List(1, 2, 3)             // List[Int]
val m = Map("a" -> 1, "b" -> 2)   // Map[String, Int]
val f = (x: Int) => x * 2         // Int => Int
```

### Where Type Annotations Are Required

| Context | Required? | Example |
|---|---|---|
| `val` / `var` with initializer | No | `val x = 42` |
| `val` / `var` without initializer | Yes | `val x: Int` (abstract) |
| Method parameters | Yes | `def f(x: Int)` |
| Method return type (public API) | Recommended | `def f(x: Int): String` |
| Method return type (recursive) | Yes | `def fac(n: Int): Int = ...` |
| Method return type (overridden) | No (inherited) | `override def toString = ...` |
| Lambda parameters (with expected type) | No | `list.map(x => x + 1)` |
| Lambda parameters (no expected type) | Yes | `val f = (x: Int) => x + 1` |
| Generic type parameters | Usually no | `List(1, 2, 3)` infers `List[Int]` |
| Implicit/given definitions | Yes | `given ord: Ordering[Int] = ...` |

### Return Type Inference

```scala
// Inferred return type: Int
def double(x: Int) = x * 2

// Required explicit annotation: recursive methods
def factorial(n: Int): Int =
  if n <= 1 then 1 else n * factorial(n - 1)

// Recommended: public API methods
def parse(input: String): Either[Error, Config] = ???
```

### Widening Rules

When no expected type is available, the compiler applies widening rules to infer the least upper bound:

```scala
val x = if true then 1 else 2.0    // Double (widening Int to Double)
val y = if true then "a" else 1    // Any (no common numeric type)
val z = if true then Some(1) else None  // Option[Int]
```

---

## Type Aliases

Source: [Scala 3 Reference -- Type Aliases](https://docs.scala-lang.org/scala3/reference/), [SLS 4.3](https://www.scala-lang.org/files/archive/spec/3.4/04-basic-definitions.html#type-member-definitions)

A type alias creates a new name for an existing type. The alias and the original type are interchangeable (transparent).

```scala
type UserId = String
type Headers = Map[String, List[String]]
type Callback[A] = A => Unit
type Matrix = Array[Array[Double]]
```

### Usage

```scala
def findUser(id: UserId): Option[User] = ???
val headers: Headers = Map("Content-Type" -> List("application/json"))
```

### Parameterized Type Aliases

```scala
type Pair[A] = (A, A)
type Result[A] = Either[String, A]
type StringMap[V] = Map[String, V]

val p: Pair[Int] = (1, 2)
val r: Result[Int] = Right(42)
```

### Type Members in Traits/Classes

```scala
trait Container:
  type Element
  def get: Element

class IntContainer extends Container:
  type Element = Int
  def get: Int = 42
```

Type aliases are transparent: `UserId` and `String` are fully interchangeable. The compiler erases the alias. For type safety without transparency, use opaque type aliases.

---

## Opaque Type Aliases

Source: [Scala 3 Reference -- Opaque Type Aliases](https://docs.scala-lang.org/scala3/reference/other-new-features/opaques.html)

Opaque type aliases provide type abstraction without runtime overhead. Outside their defining scope, the alias is opaque (not interchangeable with its underlying type). Inside the defining scope, it is transparent.

```scala
object UserId:
  opaque type UserId = String

  def apply(raw: String): UserId = raw         // inside: transparent
  extension (id: UserId)
    def value: String = id                      // inside: transparent
    def isValid: Boolean = id.nonEmpty

import UserId.UserId

val id: UserId = UserId("usr_abc123")
// val s: String = id                           // error: outside scope, opaque
val s: String = id.value                        // OK: explicit unwrap via extension
```

### With Bounds

Opaque types can declare upper or lower bounds visible from outside:

```scala
object Percentage:
  opaque type Percentage <: Double = Double

  def apply(value: Double): Percentage =
    require(value >= 0.0 && value <= 100.0)
    value

import Percentage.Percentage

val p: Percentage = Percentage(85.0)
val d: Double = p                              // OK: Percentage <: Double (declared bound)
// val p2: Percentage = 50.0                    // error: Double is not Percentage
```

### Comparison with Other Approaches

| Approach | Runtime Cost | Type Safety | Boilerplate |
|---|---|---|---|
| Type alias (`type X = Y`) | None | None (transparent) | Minimal |
| Opaque type alias | None | Full (outside scope) | Low |
| Value class (`AnyVal`) | Minimal (sometimes boxed) | Full | Medium |
| Case class wrapper | Heap allocation | Full | Medium |

---

## Union Types

Source: [Scala 3 Reference -- Union Types](https://docs.scala-lang.org/scala3/reference/new-types/union-types.html)

A union type `A | B` includes all values of type `A` and all values of type `B`. Union types are a Scala 3 feature with no direct Scala 2 equivalent.

```scala
def parse(input: String): Int | String =
  input.toIntOption match
    case Some(n) => n
    case None    => s"Not a number: $input"

val result: Int | String = parse("42")
```

### Properties

- `A | B` is a **supertype** of both `A` and `B`.
- Union is commutative: `A | B =:= B | A`.
- Union is associative: `(A | B) | C =:= A | (B | C)`.
- `A | Nothing =:= A`.
- `A | Any =:= Any`.
- Union types are **untagged** --- there is no runtime wrapper. You must use pattern matching or `isInstanceOf` to distinguish cases.

### Pattern Matching on Unions

```scala
def handle(x: Int | String | Boolean): String = x match
  case i: Int     => s"number: $i"
  case s: String  => s"text: $s"
  case b: Boolean => s"flag: $b"
```

### Extracting from Unions

```scala
type JsonValue = String | Int | Double | Boolean | Null

def stringify(v: JsonValue): String = v match
  case s: String  => s""""$s""""
  case n: Int     => n.toString
  case d: Double  => d.toString
  case b: Boolean => b.toString
  case null       => "null"
```

---

## Intersection Types

Source: [Scala 3 Reference -- Intersection Types](https://docs.scala-lang.org/scala3/reference/new-types/intersection-types.html)

An intersection type `A & B` includes values that are simultaneously of type `A` and type `B`. It replaces Scala 2's compound types (`A with B`).

```scala
trait Loggable:
  def log(msg: String): Unit

trait Serializable:
  def serialize: Array[Byte]

def process(obj: Loggable & Serializable): Unit =
  obj.log("processing")
  val bytes = obj.serialize
```

### Properties

- `A & B` is a **subtype** of both `A` and `B`.
- Intersection is commutative: `A & B =:= B & A`.
- Intersection is associative: `(A & B) & C =:= A & (B & C)`.
- `A & Any =:= A`.
- `A & Nothing =:= Nothing`.
- If `A <: B` then `A & B =:= A`.

### Member Resolution

When both types define a member with the same name, the intersection type has a member with the intersection of their types:

```scala
trait A:
  def value: Int | String

trait B:
  def value: String | Boolean

// (A & B).value has type (Int | String) & (String | Boolean) = String
```

---

## Enum Types

Source: [Scala 3 Reference -- Enums](https://docs.scala-lang.org/scala3/reference/enums/enums.html)

The `enum` keyword in Scala 3 defines enumerations and algebraic data types. It replaces the verbose sealed trait + case object pattern of Scala 2.

### Simple Enums

```scala
enum Color:
  case Red, Green, Blue
```

- Each case is a singleton instance.
- `Color.values` returns an `Array[Color]` of all cases.
- `Color.valueOf("Red")` returns the case by name (throws `IllegalArgumentException` if not found).
- `Color.Red.ordinal` returns the 0-based index.

### Parameterized Enums

```scala
enum Planet(val mass: Double, val radius: Double):
  case Mercury extends Planet(3.303e+23, 2.4397e6)
  case Venus   extends Planet(4.869e+24, 6.0518e6)
  case Earth   extends Planet(5.976e+24, 6.37814e6)

  def surfaceGravity: Double = 6.67300e-11 * mass / (radius * radius)
```

### Algebraic Data Types (ADTs)

Enums with heterogeneous cases define ADTs:

```scala
enum Expr:
  case Literal(value: Double)
  case Add(left: Expr, right: Expr)
  case Multiply(left: Expr, right: Expr)
  case Negate(expr: Expr)
```

Parameterized cases are **case classes** under the hood. They get `apply`, `unapply`, `copy`, `equals`, `hashCode`, and `toString`.

### Generalized ADTs (GADTs)

```scala
enum Json:
  case JNull
  case JBool(value: Boolean)
  case JNum(value: Double)
  case JStr(value: String)
  case JArr(values: List[Json])
  case JObj(fields: Map[String, Json])
```

### Enum with Members

```scala
enum Direction:
  case North, South, East, West

  def opposite: Direction = this match
    case North => South
    case South => North
    case East  => West
    case West  => East
```

### Scala 3 Enum vs Scala 2 Patterns

| Feature | Scala 3 `enum` | Scala 2 `sealed trait` |
|---|---|---|
| Simple values | `case Red, Green, Blue` | `case object Red extends Color` |
| Exhaustiveness | Automatic | Automatic (with `sealed`) |
| `ordinal` | Built-in | Manual |
| `values` | Built-in | Manual (or `enumeratum` library) |
| ADTs | `case Literal(v: Int)` | `case class Literal(v: Int) extends Expr` |

---

## Case Classes

Source: [Scala 3 Book -- Domain Modeling](https://docs.scala-lang.org/scala3/book/domain-modeling-tools.html#case-classes)

A `case class` is a class with automatic derivation of several useful methods. It is the primary tool for modeling immutable data in Scala.

```scala
case class Point(x: Double, y: Double)
case class Config(host: String, port: Int, debug: Boolean = false)
```

### Automatically Generated

| Feature | Description |
|---|---|
| `apply` factory | `Point(1.0, 2.0)` instead of `new Point(1.0, 2.0)` |
| `unapply` extractor | Enables pattern matching: `case Point(x, y) => ...` |
| `copy` method | `p.copy(x = 3.0)` creates modified copy |
| `equals` / `hashCode` | Structural equality based on all fields |
| `toString` | `Point(1.0, 2.0)` (descriptive string) |
| `Product` trait | `productArity`, `productElement`, `productIterator` |
| `Serializable` trait | JVM serialization support |

### Constructor Parameters

- All parameters are `val` by default (public, immutable).
- Parameters can be declared `var` (mutable, but discouraged).
- Default values are supported.
- Named arguments are supported.

```scala
val cfg = Config(host = "localhost", port = 8080)
val debug = cfg.copy(debug = true)
```

### Companion Object

Every case class automatically gets a companion object containing the `apply` and `unapply` methods:

```scala
// Compiler generates roughly:
object Point:
  def apply(x: Double, y: Double): Point = new Point(x, y)
  def unapply(p: Point): Some[(Double, Double)] = Some((p.x, p.y))
```

### Case Classes vs Regular Classes

| Feature | `case class` | `class` |
|---|---|---|
| Constructor params | `val` by default | Private by default |
| `equals` / `hashCode` | Structural (auto) | Referential (default) |
| `toString` | Descriptive (auto) | `ClassName@hash` (default) |
| Pattern matching | Yes (`unapply`) | No (unless manually defined) |
| `copy` method | Yes | No |
| Immutability | Encouraged (val fields) | No default |

### Case Objects

A case object is a singleton with the same benefits as a case class (serializable, descriptive `toString`, usable in pattern matching):

```scala
case object NotFound
case object Timeout
```

---

## Pattern Matching Basics

Source: [Scala 3 Reference -- Match Expressions](https://docs.scala-lang.org/scala3/reference/changed-features/match-syntax.html), [Scala 3 Book -- Control Structures](https://docs.scala-lang.org/scala3/book/control-structures.html#match-expressions)

Pattern matching is a core language feature in Scala. A `match` expression compares a value (scrutinee) against a sequence of patterns.

### Syntax (Scala 3)

```scala
x match
  case pattern1 => result1
  case pattern2 => result2
  case _        => default
```

### Pattern Types

#### Literal Patterns

```scala
def describe(x: Int): String = x match
  case 0 => "zero"
  case 1 => "one"
  case _ => "other"
```

#### Variable Patterns

```scala
x match
  case n => s"got $n"   // binds x to n (always matches)
```

#### Type Patterns

```scala
def inspect(x: Any): String = x match
  case i: Int    => s"Int: $i"
  case s: String => s"String: $s"
  case _         => "unknown"
```

> **Warning:** Type patterns on generic types are erased at runtime on the JVM. `case l: List[Int]` will match any `List`, not just `List[Int]`. The compiler issues an unchecked warning.

#### Constructor Patterns (Case Classes)

```scala
case class Point(x: Int, y: Int)

point match
  case Point(0, 0)    => "origin"
  case Point(x, 0)    => s"x-axis at $x"
  case Point(0, y)    => s"y-axis at $y"
  case Point(x, y)    => s"($x, $y)"
```

#### Tuple Patterns

```scala
(1, "hello") match
  case (1, s) => s"one and $s"
  case (n, s) => s"$n and $s"
```

#### Sequence Patterns

```scala
list match
  case Nil          => "empty"
  case head :: Nil   => s"single: $head"
  case head :: tail  => s"head: $head, rest: $tail"
  case _            => "other"
```

#### Guard Clauses

```scala
x match
  case n if n > 0 => "positive"
  case n if n < 0 => "negative"
  case _          => "zero"
```

#### Or Patterns

```scala
x match
  case 0 | 1 => "binary"
  case _     => "other"
```

#### Binding Patterns (`@`)

```scala
expr match
  case n @ Negate(Negate(_)) => s"double negation: $n"
  case other                 => s"other: $other"
```

### Exhaustiveness Checking

The compiler checks that match expressions on `sealed` types and `enum` types are exhaustive. A warning is issued if cases are missing:

```scala
enum Color:
  case Red, Green, Blue

color match                         // warning: missing case Blue
  case Color.Red   => "red"
  case Color.Green => "green"
```

### Match as Expression

`match` is an expression and returns a value:

```scala
val label: String = status match
  case 200 => "OK"
  case 404 => "Not Found"
  case _   => "Unknown"
```

---

## Conversions

Source: [Scala 3 Book -- First Look at Types](https://docs.scala-lang.org/scala3/book/first-look-at-types.html), [scala.AnyVal subtypes](https://www.scala-lang.org/api/3.x/)

### Implicit Widening Conversions

The compiler inserts implicit widening conversions for numeric types where no precision is lost (following the JVM specification):

```
Byte -> Short -> Int -> Long -> Float -> Double
                 Char -> Int
```

```scala
val b: Byte = 42
val i: Int = b          // OK: implicit widening
val d: Double = 3.14f   // OK: Float -> Double
```

### Explicit Conversions (`toX` methods)

For narrowing or non-implicit conversions, use explicit methods on numeric types:

| Method | Description | Example |
|---|---|---|
| `.toByte` | Truncate to 8-bit signed | `256.toByte` returns `0` |
| `.toShort` | Truncate to 16-bit signed | `70000.toShort` returns `4464` |
| `.toInt` | Truncate to 32-bit signed | `3.14.toInt` returns `3` |
| `.toLong` | Convert to 64-bit signed | `42.toLong` returns `42L` |
| `.toFloat` | Convert to 32-bit IEEE 754 | `42.toFloat` returns `42.0f` |
| `.toDouble` | Convert to 64-bit IEEE 754 | `42.toDouble` returns `42.0` |
| `.toChar` | Convert Int to Char | `65.toChar` returns `'A'` |

### String Conversions

```scala
val s = "42"
val i: Int = s.toInt                  // throws NumberFormatException if invalid
val safe: Option[Int] = s.toIntOption // None if invalid (Scala 2.13+/3)
val d: Double = s.toDouble
val safed: Option[Double] = s.toDoubleOption
```

### Type Ascription and Casting

```scala
// Type ascription (compile-time, safe)
val x: Any = 42
val y = x: Any

// Unsafe cast (runtime, may throw ClassCastException)
val i = x.asInstanceOf[Int]

// Type test
if x.isInstanceOf[Int] then
  val i = x.asInstanceOf[Int]
```

Prefer pattern matching over `isInstanceOf`/`asInstanceOf`:

```scala
x match
  case i: Int => s"got int: $i"
  case _      => "not an int"
```

---

## Equality

Source: [Scala 3 Reference -- Multiversal Equality](https://docs.scala-lang.org/scala3/reference/contextual/multiversal-equality.html), [Scala 3 Book -- Equality](https://docs.scala-lang.org/scala3/book/ca-multiversal-equality.html)

### Structural Equality (`==` and `!=`)

The `==` operator in Scala delegates to `equals`. It performs **structural** (value-based) equality, not referential equality. It is also null-safe.

```scala
val a = List(1, 2, 3)
val b = List(1, 2, 3)
a == b     // true (structural equality)
a != b     // false

val s1 = "hello"
val s2 = "hello"
s1 == s2   // true

null == "hello"   // false (no NullPointerException)
"hello" == null   // false (no NullPointerException)
```

### Referential Equality (`eq` and `ne`)

The `eq` method tests whether two references point to the same object in memory. Only available on `AnyRef` types.

```scala
val a = new String("hello")
val b = new String("hello")
a == b    // true  (structural: same characters)
a eq b    // false (referential: different objects)
a ne b    // true

val c = a
a eq c    // true  (same reference)
```

### Multiversal Equality (Scala 3)

By default, Scala allows comparing any two values with `==`. Scala 3 introduces **strict equality** via the `CanEqual` type class, enabled with `import scala.language.strictEquality`:

```scala
import scala.language.strictEquality

case class UserId(value: String) derives CanEqual
case class OrderId(value: String) derives CanEqual

val uid = UserId("abc")
val oid = OrderId("abc")
// uid == oid              // error: values of types UserId and OrderId cannot be compared
uid == UserId("abc")       // OK: same type

// To allow cross-type comparison, provide a given instance:
given CanEqual[UserId, OrderId] = CanEqual.derived
uid == oid                 // now OK
```

### Equality Summary

| Operator | Meaning | Null-safe | Available On |
|---|---|---|---|
| `==` | Structural equality (`equals`) | Yes | `Any` |
| `!=` | Structural inequality | Yes | `Any` |
| `eq` | Referential identity | N/A | `AnyRef` |
| `ne` | Referential non-identity | N/A | `AnyRef` |

### Case Class Equality

Case classes automatically derive structural `equals` and `hashCode` based on all constructor parameters:

```scala
case class Point(x: Int, y: Int)

Point(1, 2) == Point(1, 2)   // true (structural)
Point(1, 2) == Point(3, 4)   // false
```

---

## Quick Reference Table: All Value Types

| Type | Size | Default Value | Literal Example | JVM Mapping |
|---|---|---|---|---|
| `Byte` | 8-bit | `0` | `42.toByte` | `byte` |
| `Short` | 16-bit | `0` | `42.toShort` | `short` |
| `Int` | 32-bit | `0` | `42` | `int` |
| `Long` | 64-bit | `0L` | `42L` | `long` |
| `Float` | 32-bit | `0.0f` | `3.14f` | `float` |
| `Double` | 64-bit | `0.0` | `3.14` | `double` |
| `Char` | 16-bit | `'\u0000'` | `'A'` | `char` |
| `Boolean` | JVM-dependent | `false` | `true` | `boolean` |
| `Unit` | N/A | `()` | `()` | `void` |

---

## References

- [Scala 3 Language Reference](https://docs.scala-lang.org/scala3/reference/)
- [Scala 3 Book](https://docs.scala-lang.org/scala3/book/introduction.html)
- [Scala 3 API Documentation](https://www.scala-lang.org/api/3.x/)
- [Scala Language Specification (3.4)](https://www.scala-lang.org/files/archive/spec/3.4/)
- [Scala 3 Migration Guide](https://docs.scala-lang.org/scala3/guides/migration/compatibility-intro.html)
- [Opaque Type Aliases](https://docs.scala-lang.org/scala3/reference/other-new-features/opaques.html)
- [Union Types](https://docs.scala-lang.org/scala3/reference/new-types/union-types.html)
- [Intersection Types](https://docs.scala-lang.org/scala3/reference/new-types/intersection-types.html)
- [Enums](https://docs.scala-lang.org/scala3/reference/enums/enums.html)
- [Multiversal Equality](https://docs.scala-lang.org/scala3/reference/contextual/multiversal-equality.html)
