# Variables and Types — Scala

## The Scala Mindset

Scala sits in a unique position in this curriculum. It runs on the JVM (so you get Java's mature ecosystem, garbage collection, and battle-tested runtime), but it layers on a type system and language design that makes Go look spartan and TypeScript look loose. Scala's core philosophy:

- **Everything is an expression.** There are no statements. `if/else` returns a value. `match` returns a value. Even `try/catch` returns a value.
- **Everything is an object.** There is no `int` vs `Integer` distinction. `42` is an object with methods: `42.toString`, `42.max(10)`. Under the hood, the compiler optimizes to JVM primitives where possible, but conceptually everything is unified.
- **Immutability is the default.** `val` (immutable) is preferred over `var` (mutable). Collections are immutable by default. The language nudges you toward functional programming without forcing it.
- **The type system is your ally.** Scala's type system can express things Go and TypeScript cannot. It catches entire categories of bugs at compile time, and it infers almost everything so the verbosity cost is low.

Coming from JS/TS: imagine TypeScript's type system turned up to 11, with real immutability, no `null`/`undefined` chaos, and pattern matching built into the language. Also everything compiles to JVM bytecode, so you get Java's performance characteristics and library ecosystem.

Coming from Go: imagine trading simplicity for expressiveness. Go has 25 keywords and one way to do things. Scala has a rich type system, multiple paradigms, and usually several idiomatic ways to express the same idea. The tradeoff is a steeper learning curve for more power.

Coming from Rust: Scala shares some DNA — algebraic data types, pattern matching, strong type inference, `Option` instead of null. But you give up memory control entirely (the JVM GC handles it) and gain a richer OOP model.

We'll use **Scala 3** syntax throughout (indentation-based, `enum` keyword, simplified syntax). Where Scala 2 differs significantly, we'll note it.

---

## How Variables Work Under the Hood

### val vs var

Scala has two binding keywords: `val` and `var`.

```scala
val port = 8080        // immutable — cannot be reassigned
var retryCount = 0     // mutable — can be reassigned
retryCount = 1         // OK
// port = 9090         // compile error: Reassignment to val
```

`val` is the default. You reach for `var` only when mutation is truly necessary (and in idiomatic Scala, that's rarely). This is the opposite of Go, where `var`/`:=` is the only option, and closer to Rust's `let` (immutable) vs `let mut`.

| | JS/TS | Go | Rust | Haskell | Scala |
|---|---|---|---|---|---|
| Immutable binding | `const` (shallow) | No equivalent | `let` (deep) | Everything | `val` (shallow) |
| Mutable binding | `let` | `var` / `:=` | `let mut` | Doesn't exist | `var` |
| Deep immutability? | No (`const obj` allows mutation) | N/A | Yes (default) | Always | Depends on the type |

Important nuance: `val` in Scala is like `const` in JavaScript — it prevents **reassignment**, not **mutation**. If you have `val list = mutable.ListBuffer(1, 2, 3)`, you can't point `list` at a different `ListBuffer`, but you can mutate the buffer's contents. This is why Scala pairs `val` with immutable collections — you get deep immutability through convention and library design, not language enforcement.

### Under the Hood: JVM Bytecode

When you write:

```scala
val maxConnections = 100
var currentLoad = 0.0
```

The Scala compiler generates JVM bytecode equivalent to:

```java
// val becomes a final field (or local)
private final int maxConnections = 100;

// var becomes a non-final field with getter/setter
private double currentLoad = 0.0;
```

`val` maps to `final` in the JVM — the JIT compiler can optimize around this. It knows the value won't change, so it can inline it, cache it in a register, or eliminate redundant reads. `var` loses these optimizations because the value could change at any time.

In method-local scope, both `val` and `var` are stack variables (just like local variables in Go or Java). The `final` distinction still helps the JIT but the memory layout is the same — a slot on the stack frame.

### Type Inference

Scala's type inference is significantly more powerful than Go's:

```scala
val port = 8080                    // inferred as Int
val host = "localhost"             // inferred as String
val ratio = 0.75                   // inferred as Double
val enabled = true                 // inferred as Boolean
val endpoints = List("/health", "/ready")  // inferred as List[String]

// Explicit type annotations — same thing, more verbose
val port: Int = 8080
val host: String = "localhost"
```

The compiler walks the expression tree and determines the most specific type that satisfies all constraints. Unlike Go, Scala can infer complex generic types, return types, and even types through multiple levels of function composition.

**Where inference breaks down** (you must annotate explicitly):
- Public method return types (good practice, and required in Scala 3 for non-trivial cases)
- Recursive functions (the compiler needs the return type to check the recursion)
- Overloaded methods
- When you want a wider type than what inference gives you

```scala
// Recursive function — return type required
def factorial(n: Int): Long =
  if n <= 1 then 1L
  else n * factorial(n - 1)

// Wanting a wider type
val x: Long = 42        // without annotation, inferred as Int
val ratio: Double = 1    // without annotation, inferred as Int (which is wrong here)
```

### Your notes
<!-- -->


---

## Stack vs Heap — The JVM Memory Model

### How the JVM Allocates

Unlike Go (escape analysis) or Rust/Zig (you decide), the JVM has a simpler but less controllable model:

- **Stack**: method parameters, local variables of primitive types, and object references (the pointer, not the object itself).
- **Heap**: all objects (including arrays, strings, collections). Managed by the garbage collector.

```scala
def processRequest(timeout: Int): String =
  val maxRetries = 3           // Int on the stack (JVM primitive)
  val config = Config(timeout) // reference on the stack, Config object on the heap
  val message = "processing"   // reference on the stack, String object on the heap
  config.describe()
```

Memory layout:

```
Stack frame for processRequest:
┌──────────────────────┐
│ timeout: 5000 (int)  │        Heap:
│ maxRetries: 3 (int)  │        ┌─────────────────────┐
│ config: ref ─────────────────>│ Config(timeout=5000) │
│ message: ref ────────────┐    └─────────────────────┘
└──────────────────────┘   │    ┌─────────────────────┐
                           └───>│ "processing"         │
                                └─────────────────────┘
```

### Value Classes and @inline — Fighting the Heap

The JVM wants to put objects on the heap. Scala provides **value classes** (Scala 2) and **opaque types** (Scala 3) to avoid heap allocation for simple wrappers:

```scala
// Opaque type — zero allocation overhead at runtime
// Compiles to raw Long in the bytecode
opaque type UserId = Long

object UserId:
  def apply(id: Long): UserId = id
  extension (id: UserId)
    def value: Long = id
    def isValid: Boolean = id > 0

val user: UserId = UserId(42L)
// At runtime, this is just a Long on the stack — no object on the heap
```

Compare this to Go's type aliases (`type UserId int64`) — same idea, zero cost, but Scala's opaque types are *actually* distinct at compile time. You can't accidentally pass a `UserId` where a `Long` is expected, unlike Go where `type UserId int64` and `int64` are freely convertible.

### Primitive Specialization

The JVM has two kinds of types: **primitives** (`int`, `long`, `double`, `boolean` — lowercase, stack-allocated, fast) and **reference types** (objects, heap-allocated, GC-managed). Java forces you to choose: `int` vs `Integer`, `double` vs `Double`.

Scala hides this behind a unified type hierarchy. You write `Int` everywhere, and the compiler decides:

```scala
val x: Int = 42           // compiles to JVM primitive int (stack, 4 bytes)
val list: List[Int] = List(1, 2, 3)  // compiles to List[Integer] — boxed!
```

**Boxing** is when a primitive gets wrapped in an object to fit into a generic context. It's automatic but has a performance cost — heap allocation, indirection, GC pressure. This matters for hot loops and large collections.

```scala
// Unboxed — fast, stack-allocated
val a: Int = 1
val b: Int = 2
val c: Int = a + b        // pure primitive arithmetic

// Boxed — the 1, 2, 3 become java.lang.Integer objects on the heap
val numbers: List[Int] = List(1, 2, 3)
```

For performance-critical code, Scala provides `@specialized` annotations and Array (which uses unboxed JVM arrays). But for day-to-day code, the boxing overhead is negligible. Measure before you optimize.

| | Go | Rust | Zig | Scala |
|---|---|---|---|---|
| Stack allocation | Escape analysis | You control | You control | Primitives only (JVM decides) |
| Heap allocation | Escape analysis | You control | You control | All objects (GC managed) |
| GC | Yes (low latency) | No | No | Yes (JVM GC — tunable, many algorithms) |
| Boxing overhead | Interfaces cause boxing | No boxing (monomorphization) | No boxing (comptime) | Generics cause boxing (except specialized) |

### Your notes
<!-- -->


---

## The Unified Type Hierarchy

### Every Value Is an Object

This is where Scala diverges sharply from every other language in the curriculum. In Go, `int` and `interface{}` are fundamentally different things. In TypeScript, `number` and `object` live in different worlds. In Scala, everything lives in one hierarchy:

```
             Any
            /   \
       AnyVal   AnyRef (= java.lang.Object)
       /    \        \
    Int  Double  String  List[_]  YourClass  ...
    Long Boolean   ...
    Char  Unit
     ...
              \     /
              Nothing
```

- **`Any`** — the top type. Every value in Scala is an `Any`. Like TypeScript's `unknown` or Go's `any` (but it has useful methods: `toString`, `==`, `hashCode`).
- **`AnyVal`** — value types. `Int`, `Double`, `Boolean`, `Char`, `Unit`, etc. These map to JVM primitives when possible.
- **`AnyRef`** — reference types. Equivalent to `java.lang.Object`. Every class, trait, and object you define extends `AnyRef`.
- **`Nothing`** — the bottom type. Subtype of *everything*. No value has type `Nothing`. Used for expressions that never return (exceptions, infinite loops). This is like TypeScript's `never`.
- **`Null`** — the type of `null`. Subtype of all `AnyRef` types. In Scala 3 with strict null checking, this is mostly eliminated.

Why this matters: because `Nothing` is a subtype of everything, `List[Nothing]` (the empty list) is a valid `List[Int]`, `List[String]`, or `List[Anything]`. This is how Scala's `Nil` (the empty list) works — it's typed as `List[Nothing]`, which is compatible with any `List[T]`.

```scala
val empty: List[Nothing] = Nil
val ints: List[Int] = empty       // OK — List[Nothing] <: List[Int]
val strs: List[String] = Nil      // OK — same reason

// Nothing as a return type means "this function never returns normally"
def fail(message: String): Nothing =
  throw RuntimeException(message)

// This compiles because Nothing is a subtype of Int
val port: Int = if true then 8080 else fail("no port configured")
```

### Your notes
<!-- -->


---

## Type System Characteristics

### Static, Nominal + Structural, Strong

Scala's type system is:

- **Static**: all types resolved at compile time
- **Nominal** (primarily): types are identified by name. `class Meters(val value: Double)` and `class Seconds(val value: Double)` are different types even though they have the same shape.
- **Structural** (optionally): Scala supports structural types (duck typing) via refinement types, though they're less common than nominal typing.
- **Strong**: no implicit conversions unless you explicitly define them (via `given` conversions in Scala 3).
- **Unified**: everything is an object, no primitive/object split at the language level.

```scala
// Nominal — these are distinct types
case class Meters(value: Double)
case class Seconds(value: Double)

val distance = Meters(100.0)
val duration = Seconds(9.58)
// distance + duration  // compile error — different types

// Structural — duck typing via refinements
type Closeable = { def close(): Unit }

def withResource(resource: Closeable)(f: Closeable => Unit): Unit =
  try f(resource)
  finally resource.close()
// Accepts any object that has a close() method, regardless of class hierarchy
```

Structural types use reflection under the hood (slow). They're useful for interop with Java libraries that don't share a common interface, but nominal typing is the default and preferred approach.

| Property | Go | TypeScript | Rust | Haskell | Scala |
|---|---|---|---|---|---|
| Static | Yes | Yes (with escape hatches) | Yes | Yes | Yes |
| Structural | Interfaces only | Everywhere | No (traits are nominal) | Type classes | Optional (refinement types) |
| Nominal | Types (not interfaces) | No | Yes | Type classes | Yes (primary) |
| Implicit conversions | No | Yes (coercion) | No | No | Opt-in (`given Conversion`) |
| Unified hierarchy | No (`int` vs `interface{}`) | Partial (`number` vs `object`) | No | No (but everything has kinds) | Yes (`Any` at the top) |

### Your notes
<!-- -->


---

## Primitive Types

### Numeric Types

| Type | Size | Range | JVM Mapping | Notes |
|---|---|---|---|---|
| `Byte` | 1 byte | -128 to 127 | `byte` | |
| `Short` | 2 bytes | -32,768 to 32,767 | `short` | |
| `Int` | 4 bytes | -2^31 to 2^31-1 | `int` | Default for integer literals |
| `Long` | 8 bytes | -2^63 to 2^63-1 | `long` | Suffix with `L`: `42L` |
| `Float` | 4 bytes | IEEE 754 single | `float` | Suffix with `f`: `3.14f` |
| `Double` | 8 bytes | IEEE 754 double | `double` | Default for decimal literals |

```scala
val port: Int = 8080
val timestamp: Long = System.currentTimeMillis()
val loadFactor: Double = 0.75
val precision: Float = 0.001f

// Integer literals
val hex = 0xFF            // 255
val binary = 0b1010       // 10
val grouped = 1_000_000   // underscores for readability (Scala 2.13+)

// Numeric conversions are explicit
val x: Int = 42
val y: Long = x.toLong    // explicit widening
val z: Int = y.toInt      // explicit narrowing — can lose data!
// val w: Long = x         // compile error in most contexts
```

Note: unlike Go where `int` is platform-dependent (64-bit on 64-bit systems), Scala's `Int` is always 32 bits because the JVM spec defines it that way. If you need 64-bit integers, use `Long` explicitly.

### Boolean, Char, Unit

```scala
val enabled: Boolean = true    // 1 byte on JVM, true/false only
val letter: Char = 'A'         // 2 bytes — UTF-16 code unit (not full Unicode!)
val nothing: Unit = ()          // the "void" type — has exactly one value: ()
```

**`Char` gotcha**: Scala's `Char` is a UTF-16 code unit (2 bytes), inherited from the JVM. This means it cannot represent all Unicode characters — emoji and many CJK characters require two `Char` values (a surrogate pair). This is the same limitation as Java and JavaScript. Go's `rune` (int32) and Rust's `char` (4 bytes, Unicode scalar) handle this better.

**`Unit`** is Scala's void. It's an actual type with exactly one value: `()`. Functions that perform side effects return `Unit`:

```scala
def logRequest(method: String, path: String): Unit =
  println(s"$method $path")
  // no explicit return needed — the last expression's value is discarded
```

### Nothing and Null

These are bottom types — types that exist in the type hierarchy but behave specially:

```scala
// Nothing — the type with no values
// Used for: functions that never return, empty collections
def terminate(code: Int): Nothing =
  System.exit(code)
  throw RuntimeException("unreachable")  // compiler needs this since System.exit doesn't return Nothing

val empty: List[Nothing] = List.empty   // compatible with List[AnyType]

// Null — the type of null (avoid in Scala!)
// In Scala 3 with -Yexplicit-nulls, null is NOT a valid value for reference types
val bad: String = null          // compiles by default, but DON'T DO THIS
// Use Option instead:
val good: Option[String] = None // the Scala way
```

### Your notes
<!-- -->


---

## Strings

### Immutable, Backed by java.lang.String

Scala strings are `java.lang.String` under the hood — immutable, UTF-16 encoded, and interned by the JVM for literals.

```scala
val host = "api.example.com"
val endpoint = host + "/v1/webhook"   // creates a new String
// host is not modified — strings are immutable
```

Under the hood:

```
Stack:                        Heap:
┌──────────┐                 ┌──────────────────────┐
│ host: ref ────────────────>│ "api.example.com"     │
│ endpoint: ref ─────┐      └──────────────────────┘
└──────────────┘     │      ┌──────────────────────────────┐
                     └─────>│ "api.example.com/v1/webhook"  │
                            └──────────────────────────────┘
```

### String Interpolation

This is where Scala leaves Go's `fmt.Sprintf` and even TypeScript's template literals behind. Three interpolation modes:

```scala
// s-interpolation — embed expressions
val service = "auth"
val port = 8443
val url = s"https://$service.internal:$port/health"
// "https://auth.internal:8443/health"

// Expressions with braces
val retries = 3
val message = s"Retrying (${retries - 1} attempts remaining)"
// "Retrying (2 attempts remaining)"

// f-interpolation — printf-style formatting
val latency = 123.456789
val log = f"Request completed in $latency%.2f ms"
// "Request completed in 123.46 ms"

val count = 42
val padded = f"$count%05d"   // "00042"

// raw-interpolation — no escape processing
val regex = raw"(\d{3})-(\d{4})"   // backslashes are literal, not escape chars
val path = raw"C:\Users\config"     // "C:\Users\config" — no issues
```

| Feature | Go | TypeScript | Rust | Scala |
|---|---|---|---|---|
| Interpolation | `fmt.Sprintf("%s:%d", host, port)` | `` `${host}:${port}` `` | `format!("{host}:{port}")` | `s"$host:$port"` |
| Format specifiers | `%05d`, `%.2f` | None built-in | `{:05}`, `{:.2}` | `f"$x%05d"`, `f"$x%.2f"` |
| Raw strings | `` `backticks` `` | None | `r"raw"` or `r#"raw"#` | `raw"raw"` |
| Custom interpolation | No | Tagged templates | No | Yes (via `StringContext`) |

### Multiline Strings

```scala
val query = """
  |SELECT id, name, status
  |FROM webhooks
  |WHERE status = 'active'
  |  AND created_at > NOW() - INTERVAL '24 hours'
  |ORDER BY created_at DESC
  |LIMIT 100
  """.stripMargin

// stripMargin removes leading whitespace up to and including the | character
// The result is a clean, left-aligned string with no extra indentation
```

The `"""triple quotes"""` create raw multiline strings. The `stripMargin` method (with `|` as the default margin character) is the idiomatic way to handle indented multiline strings in Scala. Without it, the string would include all the leading whitespace from your source code indentation.

```scala
// Combine interpolation with multiline
val tableName = "audit_events"
val minSeverity = "warn"

val query = s"""
  |SELECT timestamp, message, severity
  |FROM $tableName
  |WHERE severity >= '$minSeverity'
  """.stripMargin
```

### Your notes
<!-- -->


---

## The Option Type — Scala's Null Safety

### The Problem with Null

Tony Hoare called null his "billion dollar mistake." In Java (and by extension, the JVM), any reference type can be `null`, leading to `NullPointerException` at runtime. Scala addresses this with `Option[T]`.

```scala
// Java-style (avoid in Scala)
def findUser(id: Long): User = {
  // might return null — caller has to remember to check
  null  // NullPointerException waiting to happen
}

// Scala-style
def findUser(id: Long): Option[User] =
  if id > 0 then Some(User(id, "alice"))
  else None
```

`Option[T]` is an algebraic data type with two variants:
- `Some(value)` — contains a value
- `None` — represents absence

The type system forces you to handle the absent case. You cannot accidentally call methods on a potentially missing value.

### Working with Option

```scala
case class WebhookConfig(
  url: String,
  secret: Option[String],     // webhook may or may not have a secret
  retryLimit: Option[Int]     // retry limit may not be configured
)

val config = WebhookConfig(
  url = "https://api.example.com/hook",
  secret = Some("whsec_abc123"),
  retryLimit = None
)

// Pattern matching — the most explicit approach
val secretDisplay = config.secret match
  case Some(s) => s"configured (${s.take(8)}...)"
  case None    => "not configured"

// getOrElse — provide a default
val maxRetries = config.retryLimit.getOrElse(3)   // 3 if None

// map — transform the value inside if it exists
val maskedSecret: Option[String] = config.secret.map(s => s.take(4) + "****")
// Some("whse****") or None — map preserves the Option wrapper

// flatMap — chain operations that themselves return Option
def validateSecret(s: String): Option[String] =
  if s.startsWith("whsec_") then Some(s) else None

val validatedSecret: Option[String] = config.secret.flatMap(validateSecret)

// for-comprehension — clean syntax for chaining Options
val result: Option[String] = for
  secret <- config.secret
  validated <- validateSecret(secret)
  limit <- config.retryLimit
yield s"Secret: $validated, retries: $limit"
// None — because retryLimit is None, the whole chain short-circuits
```

### Option vs Null Across Languages

| Language | Null safety approach | Compile-time safe? |
|---|---|---|
| Go | Zero values + explicit `nil` checks | No — nil pointer dereference is runtime |
| TypeScript | `\| undefined` union + strictNullChecks | Yes (when enabled) |
| Rust | `Option<T>` enum, no null | Yes — must pattern match or unwrap |
| Haskell | `Maybe a` = `Just a \| Nothing` | Yes — must pattern match |
| Python | `Optional[T]` annotation (not enforced) | No (unless using mypy) |
| **Scala** | `Option[T]` = `Some(T) \| None` | Yes — type system forces handling |

Scala's `Option` is essentially the same concept as Rust's `Option` and Haskell's `Maybe`. The mental model is identical: wrap potentially absent values in a container type, and the type system ensures you handle the empty case.

### Under the Hood

`Option` is a sealed abstract class with two subclasses:

```scala
// Simplified — actual implementation has more methods
sealed abstract class Option[+A]
case class Some[+A](value: A) extends Option[A]
case object None extends Option[Nothing]
```

The `+A` is **covariance** — it means `Option[String]` is a subtype of `Option[Any]`. And `None` extends `Option[Nothing]` — since `Nothing` is the bottom type and `Option` is covariant, `None` is a valid `Option[AnyType]`. Same trick as `List[Nothing]` from earlier.

### Your notes
<!-- -->


---

## Collections Preview

Scala's collections library is one of the richest in any language. Full coverage comes in the collections module, but here's the foundation.

### Immutable by Default

```scala
val servers = List("web-01", "web-02", "web-03")
// servers is scala.collection.immutable.List — you cannot add or remove elements
// "modifying" operations return new lists

val withBackup = servers :+ "backup-01"        // append — new list
val withPrimary = "primary-01" :: servers       // prepend — new list (O(1))
// servers is unchanged

val ports = Map("http" -> 80, "https" -> 443, "grpc" -> 50051)
val withMetrics = ports + ("metrics" -> 9090)   // new map with added entry
// ports is unchanged

val protocols = Set("tcp", "udp", "quic")
val withHttp3 = protocols + "http3"             // new set
```

### Common Collection Types

| Type | Description | Indexed? | Notes |
|---|---|---|---|
| `List[T]` | Singly-linked list | No (O(n)) | Fast prepend (O(1)), pattern matching friendly |
| `Vector[T]` | Balanced tree (32-ary) | Yes (effectively O(1)) | Best general-purpose immutable sequence |
| `Map[K, V]` | Hash trie | By key (effectively O(1)) | Immutable by default |
| `Set[T]` | Hash trie | N/A | Immutable, no duplicates |
| `Seq[T]` | Abstract sequence | Depends | Supertype of List and Vector |
| `Array[T]` | JVM array | Yes (O(1)) | Mutable, fixed-size, unboxed for primitives |

```scala
// Practical example: processing a list of health check endpoints
case class Endpoint(service: String, path: String, port: Int)

val endpoints = List(
  Endpoint("auth", "/health", 8080),
  Endpoint("billing", "/healthz", 8081),
  Endpoint("notifications", "/ready", 8082),
)

// Filter, transform, collect — all return new collections
val healthPaths: List[String] = endpoints.map(e => s"http://localhost:${e.port}${e.path}")
val httpEndpoints = endpoints.filter(_.port < 8082)
val serviceMap: Map[String, Endpoint] = endpoints.map(e => e.service -> e).toMap
```

### Mutable Collections (When You Need Them)

```scala
import scala.collection.mutable

// Explicit import required — Scala makes you be intentional about mutation
val buffer = mutable.ListBuffer[String]()
buffer += "event-1"
buffer += "event-2"
val frozen: List[String] = buffer.toList  // convert to immutable when done
```

The convention: build with mutable, expose as immutable. Internal implementation details can use `var` and mutable collections for performance, but public APIs should return immutable types.

### Your notes
<!-- -->


---

## Type Inference — How Far It Goes

### Local Inference

Scala infers types locally within expressions, method bodies, and generic arguments:

```scala
// Simple inference
val timeout = 5000                        // Int
val ratio = 0.8                           // Double
val endpoints = List("a", "b", "c")       // List[String]

// Generic method inference
val numbers = List(1, 2, 3)
val doubled = numbers.map(_ * 2)          // List[Int] — inferred from the operation
val strings = numbers.map(_.toString)     // List[String]

// Chained operations
val result = numbers
  .filter(_ > 1)         // List[Int]
  .map(_ * 10)           // List[Int]
  .mkString(", ")        // String
// Scala infers every intermediate type without annotations
```

### Where Inference Cannot Help

```scala
// 1. Public method return types (required for clarity and separate compilation)
def parseConfig(raw: String): Config = ???   // return type required
// def parseConfig(raw: String) = ???         // compiles but considered bad practice for public APIs

// 2. Recursive functions (compiler needs the return type to check recursion)
def retry(attempts: Int): Either[String, Response] =
  if attempts <= 0 then Left("exhausted retries")
  else
    makeRequest() match
      case Right(r) => Right(r)
      case Left(_)  => retry(attempts - 1)   // recursive call — return type must be declared

// 3. Overloaded methods
def send(message: String): Unit = ???
def send(message: String, priority: Int): Unit = ???
// Return types required when overloading

// 4. When you want a wider type
val x = 42            // inferred as Int
val x: Long = 42      // you want Long — must annotate
val x: Any = 42       // you want Any — must annotate
```

### Type Ascription vs Annotation

```scala
// Type annotation (on binding)
val x: Long = 42

// Type ascription (on expression) — less common but useful
val x = 42: Long                   // ascribe the literal to Long
val items = List(1, 2, 3): Seq[Int]  // widen from List to Seq
```

### Compared to Other Languages

| Feature | Go | TypeScript | Rust | Scala |
|---|---|---|---|---|
| Local variable inference | `:=` | `const x = 5` | `let x = 5` | `val x = 5` |
| Generic argument inference | Limited | Good | Good | Excellent |
| Return type inference | Never (always explicit) | Yes | Yes | Yes (but annotate public methods) |
| Multi-expression inference | No | Partial | Good | Very good (chains, for-comprehensions) |

### Your notes
<!-- -->


---

## Case Classes — Data Done Right

### The Problem They Solve

In Go, you define a struct and manually implement `String()`, comparison, and copying. In Java, you write `equals`, `hashCode`, `toString`, getters, setters, and a constructor. Case classes give you all of that for free.

```scala
case class WebhookEvent(
  id: String,
  source: String,
  eventType: String,
  payload: String,
  timestamp: Long,
  retryCount: Int = 0     // default value
)
```

That single declaration generates:
- **Constructor**: `WebhookEvent("evt-1", "billing", "invoice.paid", "{...}", 1707300000L)`
- **Named parameters**: `WebhookEvent(id = "evt-1", source = "billing", ...)`
- **`toString`**: `WebhookEvent(evt-1,billing,invoice.paid,{...},1707300000,0)`
- **`equals` and `hashCode`**: structural equality based on all fields
- **`copy`**: create a modified copy without touching the original
- **Pattern matching support**: destructure in `match` expressions
- **`apply`/`unapply`**: factory method and extractor (no `new` keyword needed)

```scala
val event = WebhookEvent(
  id = "evt-001",
  source = "billing",
  eventType = "invoice.paid",
  payload = """{"amount": 99.99}""",
  timestamp = System.currentTimeMillis()
)

// Structural equality (like comparing by value, not reference)
val event2 = event.copy()
println(event == event2)    // true — compares all fields
// In Java/Go, == on objects/structs compares references or requires custom implementation

// copy — immutable update pattern
val retried = event.copy(retryCount = event.retryCount + 1)
// retried is a new object with retryCount = 1, everything else identical
// event is unchanged

// Pattern matching
event match
  case WebhookEvent(id, _, "invoice.paid", payload, _, _) =>
    println(s"Processing invoice payment $id: $payload")
  case WebhookEvent(id, _, eventType, _, _, retries) if retries > 3 =>
    println(s"Dropping event $id ($eventType) after $retries retries")
  case other =>
    println(s"Queuing event: ${other.id}")
```

### Under the Hood: What the Compiler Generates

For that single `case class` declaration, the Scala compiler generates roughly this much Java equivalent:

```java
// GENERATED (simplified)
public final class WebhookEvent {
    private final String id;
    private final String source;
    // ... all fields as private final

    public WebhookEvent(String id, String source, ...) { ... }

    public String id() { return id; }        // accessor
    public String source() { return source; }

    public WebhookEvent copy(String id, String source, ...) { ... }

    @Override public boolean equals(Object o) {
        // structural comparison of ALL fields
    }

    @Override public int hashCode() {
        // hash of ALL fields
    }

    @Override public String toString() {
        return "WebhookEvent(" + id + "," + source + "," + ... + ")";
    }
}
```

This is why Scala developers rarely write Java-style classes with manual `equals`/`hashCode`. The compiler does it correctly and consistently.

### Case Classes vs Equivalents in Other Languages

| Feature | Go struct | TS interface/type | Rust struct + derive | Scala case class |
|---|---|---|---|---|
| Auto equals | No (deep compare with reflect) | No (manual or lodash) | `#[derive(PartialEq)]` | Yes |
| Auto hash | No | N/A | `#[derive(Hash)]` | Yes |
| Auto toString | No (implement `String()`) | No | `#[derive(Debug)]` | Yes |
| Immutable copy | Manual | Spread: `{...obj, field: val}` | `Clone` + manual | `.copy(field = val)` |
| Pattern matching | No | Discriminated unions (partial) | Yes (with `match`) | Yes (built-in) |
| Default values | Zero values | Optional `?:` | `Default` trait | Parameter defaults |

### Your notes
<!-- -->


---

## Tuples — Lightweight Grouping

### When You Don't Need a Full Type

Tuples are fixed-size, heterogeneous collections. Use them for quick grouping when defining a case class would be overkill.

```scala
// Return multiple values from a function
def parseHostPort(address: String): (String, Int) =
  val parts = address.split(":")
  (parts(0), parts(1).toInt)

val (host, port) = parseHostPort("localhost:8080")
// host: String = "localhost"
// port: Int = 8080

// Named tuple elements (Scala 3)
val endpoint: (host: String, port: Int) = (host = "localhost", port = 8080)
println(endpoint.host)   // "localhost"
println(endpoint.port)   // 8080

// Tuples of different sizes
val pair: (String, Int) = ("timeout", 5000)
val triple: (String, Int, Boolean) = ("cache", 300, true)

// Access by index (zero-based)
val key = pair._1     // "timeout"
val value = pair._2   // 5000

// Pattern matching with tuples
def describeConfig(entry: (String, Int, Boolean)): String = entry match
  case (name, value, true)  => s"$name = $value (enabled)"
  case (name, value, false) => s"$name = $value (disabled)"
```

### Tuples vs Case Classes — When to Use Which

**Use tuples for:**
- Private/local groupings within a method
- Simple key-value pairs in maps
- Quick multiple return values
- Short-lived intermediate data

**Use case classes for:**
- Public API types
- Anything with more than 3 fields
- Types that will be pattern-matched frequently
- Data that has behavior (methods)
- Domain objects

```scala
// Tuple is fine here — private, short-lived
private def splitHeader(header: String): (String, String) =
  val idx = header.indexOf(":")
  (header.take(idx).trim, header.drop(idx + 1).trim)

// Case class is better here — public, semantic, reusable
case class HttpHeader(name: String, value: String):
  def isContentType: Boolean = name.equalsIgnoreCase("Content-Type")
```

### Your notes
<!-- -->


---

## Enums and Algebraic Data Types (Scala 3)

### Simple Enums

Scala 3 introduces the `enum` keyword (Scala 2 used sealed traits for this):

```scala
enum LogLevel:
  case Debug, Info, Warn, Error, Fatal

val level = LogLevel.Warn
println(level)           // Warn
println(level.ordinal)   // 2

// Pattern matching on enums
def colorize(level: LogLevel): String = level match
  case LogLevel.Debug => "\u001b[37m"   // gray
  case LogLevel.Info  => "\u001b[32m"   // green
  case LogLevel.Warn  => "\u001b[33m"   // yellow
  case LogLevel.Error => "\u001b[31m"   // red
  case LogLevel.Fatal => "\u001b[35m"   // magenta
// The compiler warns if you forget a case — exhaustive checking
```

### Enums with Data — Algebraic Data Types

This is where Scala's enums become powerful. Each variant can carry different data:

```scala
enum DeliveryResult:
  case Delivered(statusCode: Int, responseTime: Long)
  case Failed(error: String, retryable: Boolean)
  case Timeout(afterMs: Long)
  case Skipped(reason: String)

def handleResult(result: DeliveryResult): Unit = result match
  case DeliveryResult.Delivered(code, time) =>
    println(s"Success: HTTP $code in ${time}ms")
  case DeliveryResult.Failed(err, true) =>
    println(s"Retryable failure: $err")
  case DeliveryResult.Failed(err, false) =>
    println(s"Permanent failure: $err — dropping")
  case DeliveryResult.Timeout(ms) =>
    println(s"Timed out after ${ms}ms")
  case DeliveryResult.Skipped(reason) =>
    println(s"Skipped: $reason")
```

This is the same concept as Rust's `enum`, Haskell's `data`, and TypeScript's discriminated unions — but integrated with Scala's object system. Each variant is a case class (if it has parameters) or a case object (if it doesn't).

**Scala 2 equivalent** (for reference — you'll see this in existing codebases):

```scala
// Scala 2 style — sealed trait + case classes
sealed trait DeliveryResult
case class Delivered(statusCode: Int, responseTime: Long) extends DeliveryResult
case class Failed(error: String, retryable: Boolean) extends DeliveryResult
case class Timeout(afterMs: Long) extends DeliveryResult
case class Skipped(reason: String) extends DeliveryResult
```

The Scala 3 `enum` syntax is sugar for exactly this. Understanding the Scala 2 form is important because most existing Scala code uses it.

### Your notes
<!-- -->


---

## Pattern Matching Preview

### match Expressions

Pattern matching is Scala's swiss army knife for control flow. It replaces `switch`, `if/else` chains, type checks, and destructuring.

```scala
// Basic value matching
def httpStatusCategory(code: Int): String = code match
  case 200 => "OK"
  case 201 => "Created"
  case 204 => "No Content"
  case c if c >= 400 && c < 500 => s"Client Error ($c)"
  case c if c >= 500 => s"Server Error ($c)"
  case c => s"Unknown ($c)"

// Type matching — like instanceof but with extraction
def describeValue(x: Any): String = x match
  case i: Int if i > 0  => s"positive integer: $i"
  case s: String         => s"string of length ${s.length}"
  case list: List[?]     => s"list with ${list.length} elements"
  case _                 => "something else"

// Destructuring case classes
case class RateLimitConfig(maxRequests: Int, windowSeconds: Int, burstAllowed: Boolean)

def describe(config: RateLimitConfig): String = config match
  case RateLimitConfig(max, _, true) if max > 1000 =>
    s"High-throughput burst mode ($max req/window)"
  case RateLimitConfig(max, window, false) =>
    s"Strict: $max requests per ${window}s, no bursting"
  case RateLimitConfig(max, window, true) =>
    s"Standard: $max requests per ${window}s with burst"
```

### Guards and Nested Patterns

```scala
case class Request(method: String, path: String, headers: Map[String, String])

def routeRequest(req: Request): String = req match
  case Request("GET", "/health", _) =>
    "health check"
  case Request("POST", path, headers) if headers.contains("Authorization") =>
    s"authenticated POST to $path"
  case Request("POST", path, _) =>
    s"unauthenticated POST to $path — rejecting"
  case Request(method, path, _) =>
    s"$method $path — routing to default handler"
```

Pattern matching is exhaustive when used with sealed types (enums, sealed traits). The compiler warns you if you miss a case. This is the same guarantee Rust and Haskell provide, and something Go and TypeScript lack.

### Your notes
<!-- -->


---

## Comparison Across Languages

### Variable Declaration

| | Go | TypeScript | Rust | Python | Haskell | Scala |
|---|---|---|---|---|---|---|
| Immutable | N/A | `const` (shallow) | `let` | Convention | Everything | `val` |
| Mutable | `var` / `:=` | `let` | `let mut` | Default | N/A | `var` |
| Type inference | `:=` | `const x = 5` | `let x = 5` | Always | Always | `val x = 5` |
| Zero/default values | Every type has one | `undefined` | None (must init) | `None` by convention | No | JVM defaults (but avoid relying on them) |

### Null/Absence Handling

| | Go | TypeScript | Rust | Python | Haskell | Scala |
|---|---|---|---|---|---|---|
| Null type | `nil` (typed) | `null`, `undefined` | None | `None` | N/A | `null` (discouraged) |
| Safe alternative | Convention (check `err`) | `strictNullChecks` | `Option<T>` | `Optional[T]` (annotation) | `Maybe a` | `Option[T]` |
| Compile-time safe? | No | With flag | Yes | No (unless mypy) | Yes | Yes |

### Type System

| | Go | TypeScript | Rust | Python | Haskell | Scala |
|---|---|---|---|---|---|---|
| Static | Yes | Yes | Yes | No (optional) | Yes | Yes |
| Strength | Strong | Weak (coercion) | Strong | Strong (duck-typed) | Very strong | Strong |
| Generics | Yes (1.18+) | Yes | Yes (monomorphized) | Yes (type hints) | Yes (parametric) | Yes (erased, with specialization) |
| Sum types | No (interface tricks) | Discriminated unions | `enum` | `Union` (3.10+) | `data` | `enum` / sealed trait |
| Pattern matching | No | No (destructuring only) | `match` | `match` (3.10+) | Core feature | `match` — core feature |
| Unified hierarchy | No | Partial | No | Yes (`object` base) | No | Yes (`Any` at the top) |

### Your notes
<!-- -->


---

## Key Takeaways

1. **`val` over `var`, always.** Reach for mutability only when you have a specific reason. Immutable bindings let the JIT optimize better and make concurrent code safer.

2. **Everything is an expression.** `if/else`, `match`, `try/catch` — they all return values. This means fewer intermediate `var`s and more direct, readable code.

3. **The type hierarchy is unified.** `Int` has methods. `42.toString` works. There's no primitive/object split at the language level. Under the hood, the compiler unboxes to JVM primitives where possible.

4. **`Option` replaces `null`.** Never use `null` in Scala code. Wrap absence in `Option[T]`, use `map`/`flatMap`/`getOrElse` to work with it, and let the type system protect you.

5. **Case classes are your data types.** Free `equals`, `hashCode`, `toString`, `copy`, pattern matching. Use them for anything that holds data.

6. **Pattern matching is central.** It's not a niche feature — it's the primary way to branch on data shapes. Combined with sealed types, the compiler guarantees exhaustiveness.

7. **Collections are immutable by default.** `List`, `Map`, `Set` return new instances on "modification." This aligns with `val` — you build a functional-first data pipeline.

8. **The JVM is your runtime.** You get Java's GC (no manual memory management), Java's libraries (anything on Maven Central), and Java's performance characteristics (JIT compilation, warm-up time, steady-state performance). The tradeoff is less control over memory layout compared to Go, Rust, or Zig.

---

## What's Next

The associated code files (`variables.scala`, `types.scala`, `option.scala`) contain runnable examples for each concept. After reading through this lesson, the progression is:

1. Read `reference.md` for the official Scala specification details
2. Run the code examples and experiment in the Scala REPL (`scala`)
3. Work through the exercises in `tracks/scala/exercises/variables-and-types/`
4. Compare notes with the Go, TypeScript, and Rust versions of this lesson

---

> **See also:**
> - [[fundamentals/variables-and-types]] — Cross-language comparison
> - [[fundamentals/scala/variables-and-types]] — Scala-specific deep dive (vault)
