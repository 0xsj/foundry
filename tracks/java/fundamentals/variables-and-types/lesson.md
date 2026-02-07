# Variables and Types — Java

## How Variables Work Under the Hood

### The Two Worlds: Primitives and Objects

Java has a hard split between two kinds of values:

- **Primitives**: stored directly on the stack (or inline in objects). No overhead. Not objects.
- **Reference types**: stored on the heap. Variables hold a pointer to the object, not the object itself.

```java
int x = 42;              // 4 bytes on the stack. that's it.
String s = "hello";      // s is a reference (pointer) on the stack
                          // the String object lives on the heap
```

```
Stack:                        Heap:
┌────────────┐
│ x: 42      │     (no heap allocation for primitives)
├────────────┤
│ s: 0x7f... ├───────→  String { value: ['h','e','l','l','o'], hash: ... }
└────────────┘
```

### JVM Memory Layout

The JVM manages memory in regions:

- **Stack**: one per thread. Holds local variables (primitives and references). Frames are pushed/popped per method call.
- **Heap**: shared across threads. All objects live here. Managed by the garbage collector.
- **Metaspace**: class metadata, method bytecode (replaced PermGen in Java 8).
- **String Pool**: interned strings live here (part of the heap).

```java
String a = "hello";           // string literal — goes to the String Pool
String b = "hello";           // reuses the SAME pooled object
String c = new String("hello"); // forces a NEW heap object (don't do this)

System.out.println(a == b);   // true — same reference (pooled)
System.out.println(a == c);   // false — different references
System.out.println(a.equals(c)); // true — same content
```

`==` compares references (addresses) for objects. `.equals()` compares content. This is one of Java's most common bugs.

### Garbage Collection

Java uses generational garbage collection:
- **Young generation**: new objects. Collected frequently (minor GC).
- **Old generation**: objects that survived multiple GCs. Collected less often (major GC).
- **Algorithms**: G1 (default since Java 9), ZGC and Shenandoah (low-latency options).

You don't manually free memory. When nothing references an object, the GC eventually reclaims it.

### Your notes
<!-- -->


---

## Type System

### Static, Nominal, Strong

Java's type system is:
- **Static**: types checked at compile time
- **Nominal**: types are identified by name. Two classes with identical fields are still different types.
- **Strong**: no implicit narrowing conversions. Widening is allowed (`int` → `long`).

```java
int a = 42;
long b = a;          // implicit widening — ok
// int c = b;        // error: possible lossy conversion from long to int
int c = (int) b;     // explicit narrowing cast required
```

### Primitive Types

| Type | Size | Default (field) | Range |
|---|---|---|---|
| `byte` | 1 byte | `0` | -128 to 127 |
| `short` | 2 bytes | `0` | -32,768 to 32,767 |
| `int` | 4 bytes | `0` | ~±2.1 billion |
| `long` | 8 bytes | `0L` | ~±9.2 quintillion |
| `float` | 4 bytes | `0.0f` | IEEE 754 |
| `double` | 8 bytes | `0.0d` | IEEE 754 |
| `char` | 2 bytes | `'\u0000'` | UTF-16 code unit |
| `boolean` | ~1 byte | `false` | JVM-dependent size |

### Wrapper Classes (Autoboxing)

Every primitive has a wrapper class: `int` → `Integer`, `double` → `Double`, etc.

```java
int a = 42;
Integer b = a;         // autoboxing: primitive → object (implicit)
int c = b;             // unboxing: object → primitive (implicit)
Integer d = null;      // wrapper can be null — primitive can't
// int e = d;          // NullPointerException at runtime! Unboxing null.
```

Autoboxing is convenient but has costs:
- Heap allocation for the wrapper object
- Possible NPE from unboxing null
- Identity comparisons break: `new Integer(5) == new Integer(5)` is `false`

```java
Integer x = 127;
Integer y = 127;
System.out.println(x == y);   // true — cached range [-128, 127]

Integer a = 128;
Integer b = 128;
System.out.println(a == b);   // false — different objects
System.out.println(a.equals(b)); // true — same value
```

### Your notes
<!-- -->


---

## Default Values: The Two Rules

**Rule 1: Class/instance fields get default zero values**
```java
public class Example {
    int count;          // defaults to 0
    String name;        // defaults to null
    boolean active;     // defaults to false
}
```

**Rule 2: Local variables have NO defaults — must initialize before use**
```java
void method() {
    int x;
    // System.out.println(x); // compile error: variable x might not have been initialized
    x = 42;
    System.out.println(x);    // ok now
}
```

This is a middle ground:
- Go: everything gets a zero value (fields AND locals)
- Rust: nothing gets a default (must always initialize)
- Java: fields get defaults, locals don't

### Your notes
<!-- -->


---

## `final` and Constants

```java
final int MAX_RETRIES = 3;     // can't reassign
// MAX_RETRIES = 5;            // compile error

final List<String> names = new ArrayList<>();
names.add("redis");            // legal! final prevents REASSIGNMENT, not mutation
// names = new ArrayList<>();  // compile error — can't reassign the reference
```

`final` = the reference can't change. The object it points to can still be mutated. This is the same concept as Rust's distinction between `let` (immutable binding) and the mutability of the data itself — except Rust's borrow checker actually prevents mutation through immutable references.

For true constants:
```java
public static final int MAX_RETRIES = 3;  // class-level constant
```

### Your notes
<!-- -->


---

## `var` (Type Inference)

Since Java 10, local variables can use `var`:

```java
var name = "redis";       // inferred as String
var port = 6379;          // inferred as int
var scores = List.of(1, 2, 3);  // inferred as List<Integer>

// var x;                 // error — can't infer without initializer
// var y = null;          // error — can't infer type from null
```

Restrictions:
- Local variables only (not fields, parameters, or return types)
- Must have an initializer
- Cannot be `null` without a cast

Compare to Go's `:=` which is more pervasive, or Rust's `let` where inference is the default everywhere.

### Your notes
<!-- -->


---

## Records (Java 16+)

Records are immutable data carriers — like Go structs but with built-in `equals`, `hashCode`, `toString`:

```java
record Point(int x, int y) {}

var p = new Point(10, 20);
System.out.println(p.x());       // 10 — accessor method (not field access)
System.out.println(p);           // Point[x=10, y=20]

var q = new Point(10, 20);
System.out.println(p.equals(q)); // true — value-based equality
```

Records are `final`, all fields are `final`. You get the immutability that Rust gives by default, but opt-in.

### Your notes
<!-- -->


---

## Composite Types (Preview)

| Type | Description | Mutable | Notes |
|---|---|---|---|
| `int[]` | Primitive array | Yes (contents) | Fixed size |
| `String[]` | Reference array | Yes (contents) | Fixed size |
| `ArrayList<T>` | Dynamic array | Yes | No primitives (autoboxing) |
| `HashMap<K,V>` | Hash map | Yes | |
| `record` | Immutable data | No | Java 16+ |
| `enum` | Fixed set of values | — | Much more powerful than C enums |

### Your notes
<!-- -->
