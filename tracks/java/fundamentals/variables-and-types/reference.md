# Java Reference — Variables and Types

> Extracted from the [Java Language Specification](https://docs.oracle.com/javase/specs/)
> for the `variables-and-types` module. Covers: variable declarations, primitive types,
> reference types, type system, null, wrapper classes, and var keyword (Java 10+).

---

## Variables

Source: [JLS §4.12](https://docs.oracle.com/javase/specs/jls/se17/html/jls-4.html#jls-4.12)

A **variable** is a storage location with an associated type. The type of a variable is determined at compile time.

Variables can hold:
- **Primitive values** (stored directly in the variable)
- **Reference values** (points to an object in memory)

---

## Variable Declarations

Source: [JLS §14.4](https://docs.oracle.com/javase/specs/jls/se17/html/jls-14.html#jls-14.4)

```java
int x;                          // declaration (uninitialized)
int y = 42;                     // declaration with initializer
final int MAX = 100;            // final (cannot be reassigned)

int a, b, c;                    // multiple declarations
int x = 1, y = 2;               // with initializers

var message = "Hello";          // type inference (Java 10+)
```

### Local Variable Type Inference (var)

Source: [JLS §14.4](https://docs.oracle.com/javase/specs/jls/se17/html/jls-14.html#jls-14.4)

Java 10 introduced `var` for local variable type inference:

```java
var count = 42;                 // inferred as int
var name = "Alice";             // inferred as String
var list = new ArrayList<String>();  // inferred as ArrayList<String>
```

**Restrictions:**
- Only for local variables (not fields, parameters, or return types)
- Must have an initializer
- Cannot infer from `null` literal

---

## Primitive Types

Source: [JLS §4.2](https://docs.oracle.com/javase/specs/jls/se17/html/jls-4.html#jls-4.2)

Java has **8 primitive types** that are not objects:

### Integral Types

| Type | Size | Range | Default |
|---|---|---|---|
| `byte` | 8 bits | -128 to 127 | `0` |
| `short` | 16 bits | -32,768 to 32,767 | `0` |
| `int` | 32 bits | -2^31 to 2^31-1 | `0` |
| `long` | 64 bits | -2^63 to 2^63-1 | `0L` |

```java
int count = 42;
long big = 1000000000L;         // L suffix for long literals
byte b = 127;
```

### Floating-Point Types

| Type | Size | Spec | Default |
|---|---|---|---|
| `float` | 32 bits | IEEE 754 single-precision | `0.0f` |
| `double` | 64 bits | IEEE 754 double-precision | `0.0d` |

```java
double pi = 3.14159;
float ratio = 0.5f;             // f suffix for float literals
```

### Boolean Type

| Type | Values | Default |
|---|---|---|
| `boolean` | `true`, `false` | `false` |

**Note:** Unlike C, Java booleans are not integers. `0` is not `false`, `1` is not `true`.

### Character Type

| Type | Size | Range | Default |
|---|---|---|---|
| `char` | 16 bits | Unicode `\u0000` to `\uFFFF` (UTF-16 code unit) | `'\u0000'` |

```java
char letter = 'A';
char unicode = '\u0041';        // also 'A'
```

---

## Reference Types

Source: [JLS §4.3](https://docs.oracle.com/javase/specs/jls/se17/html/jls-4.html#jls-4.3)

Reference types include:
- Class types (`String`, `Object`, `ArrayList`, etc.)
- Interface types (`List`, `Comparable`, etc.)
- Array types (`int[]`, `String[]`, etc.)
- Type variables (generics)

```java
String name = "Alice";          // reference to String object
int[] numbers = {1, 2, 3};      // reference to array object
Object obj = new Object();      // reference to Object instance
```

### Reference vs Primitive

| Primitive | Reference |
|---|---|
| Stored by value | Stored by reference |
| Fixed size | Variable size (object) |
| Always has a value | Can be `null` |
| Cannot call methods | Can call methods |
| Passed by value (copy) | Reference passed by value (copy of pointer) |

---

## null Literal

Source: [JLS §3.10.8](https://docs.oracle.com/javase/specs/jls/se17/html/jls-3.html#jls-3.10.8)

`null` is the default value for reference types. It represents "no object."

```java
String name = null;             // valid — no object assigned
int count = null;               // compile error — primitives cannot be null
```

**NullPointerException:** Occurs when you dereference `null`.

```java
String s = null;
s.length();                     // throws NullPointerException at runtime
```

---

## Wrapper Classes

Source: [Java API](https://docs.oracle.com/en/java/javase/17/docs/api/java.base/java/lang/package-summary.html)

Each primitive type has a corresponding wrapper class (reference type):

| Primitive | Wrapper Class |
|---|---|
| `byte` | `Byte` |
| `short` | `Short` |
| `int` | `Integer` |
| `long` | `Long` |
| `float` | `Float` |
| `double` | `Double` |
| `char` | `Character` |
| `boolean` | `Boolean` |

### Autoboxing and Unboxing

Java automatically converts between primitives and wrappers:

```java
int primitive = 42;
Integer wrapped = primitive;    // autoboxing (int → Integer)
int back = wrapped;             // unboxing (Integer → int)

// Common use case: collections only hold reference types
List<Integer> numbers = new ArrayList<>();
numbers.add(42);                // autoboxing
int first = numbers.get(0);     // unboxing
```

**Caution:** Unboxing `null` throws `NullPointerException`.

```java
Integer wrapped = null;
int x = wrapped;                // NullPointerException
```

---

## Default Values

Source: [JLS §4.12.5](https://docs.oracle.com/javase/specs/jls/se17/html/jls-4.html#jls-4.12.5)

Instance variables and class variables (fields) are initialized to default values if not explicitly initialized:

| Type | Default Value |
|---|---|
| `byte`, `short`, `int`, `long` | `0` |
| `float`, `double` | `0.0` |
| `boolean` | `false` |
| `char` | `'\u0000'` |
| Reference types | `null` |

**Local variables have no default value** — must be explicitly initialized before use.

```java
class Example {
    int field;                  // initialized to 0

    void method() {
        int local;              // NOT initialized
        // System.out.println(local);  // compile error
    }
}
```

---

## final Variables

Source: [JLS §4.12.4](https://docs.oracle.com/javase/specs/jls/se17/html/jls-4.html#jls-4.12.4)

The `final` modifier prevents reassignment:

```java
final int MAX_SIZE = 100;
// MAX_SIZE = 200;              // compile error

final StringBuilder sb = new StringBuilder();
sb.append("Hello");             // OK — mutating object is fine
// sb = new StringBuilder();    // compile error — cannot reassign reference
```

**`final` does not mean immutable** — it only prevents reassignment. The object itself can be mutable.

---

## String Type

Source: [Java API: String](https://docs.oracle.com/en/java/javase/17/docs/api/java.base/java/lang/String.html)

`String` is a reference type representing immutable sequences of characters (UTF-16).

```java
String name = "Alice";          // string literal (interned)
String copy = new String("Alice");  // new object (not recommended)

// String operations
int len = name.length();
char first = name.charAt(0);
String upper = name.toUpperCase();  // returns new String
String concat = name + " Bob";      // concatenation (creates new String)
```

### String Literals and Interning

String literals are stored in the **string pool** — identical literals refer to the same object:

```java
String a = "hello";
String b = "hello";
a == b;                         // true — same object (interned)

String c = new String("hello");
a == c;                         // false — different objects
a.equals(c);                    // true — same value
```

**Use `.equals()` for value comparison, not `==`** (which compares references).

---

## Arrays

Source: [JLS §10](https://docs.oracle.com/javase/specs/jls/se17/html/jls-10.html)

Arrays are objects with fixed size and indexed access:

```java
int[] numbers = new int[5];     // array of 5 ints (initialized to 0)
int[] values = {1, 2, 3, 4, 5}; // array initializer

int first = values[0];          // indexing (0-based)
int len = values.length;        // length (field, not method)
```

**Arrays are covariant** — subtype relationships are preserved:

```java
String[] strings = {"a", "b"};
Object[] objects = strings;     // valid — String[] is a subtype of Object[]
```

---

## Type System

### Static Typing

Java is **statically typed** — all variable types are known at compile time.

```java
int x = 42;
// x = "hello";                 // compile error — type mismatch
```

### Strong Typing

Java is **strongly typed** — no implicit conversions between incompatible types.

```java
int i = 42;
long l = i;                     // OK — widening conversion (int → long)
int j = l;                      // compile error — narrowing requires cast
int k = (int) l;                // explicit cast
```

### Nominal Typing

Java uses **nominal typing** — types are compatible based on their names and declarations, not structure.

```java
class A { int x; }
class B { int x; }

A a = new A();
B b = a;                        // compile error — A and B are different types
```

Compare to structural typing (TypeScript, Go interfaces) where structure determines compatibility.

---

## var Limitations and Best Practices

**When to use `var`:**
- Type is obvious from right-hand side
- Reduces verbosity without losing clarity

```java
var list = new ArrayList<String>();     // clear what type it is
var stream = Files.lines(path);         // clear from method name
```

**When NOT to use `var`:**
- Type is not obvious
- Reduces readability

```java
var result = calculate();               // what type is result?
var x = 10;                             // use explicit type for simple cases
```

---

## Constants

By convention, constants are `static final` fields with `UPPER_SNAKE_CASE` names:

```java
public static final int MAX_RETRIES = 3;
public static final String DEFAULT_HOST = "localhost";
```

---

## Notes

- **Primitive vs Reference:** Fundamental distinction in Java. Primitives are values, references are pointers.
- **No unsigned integers:** Java has no unsigned types (unlike C/C++). Use `long` if `int` range is insufficient.
- **Character encoding:** `char` is UTF-16 code unit (16 bits). Unicode characters outside BMP (like emoji) require two `char` values (surrogate pairs).
- **String immutability:** All String methods return new Strings. For mutable strings, use `StringBuilder` or `StringBuffer`.
- **null safety:** Java has no built-in null safety. Modern Java (14+) supports pattern matching and records which help, but NPE is still common.
