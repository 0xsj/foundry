# C# Reference — Variables and Types

> Extracted from the [C# Language Specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/)
> for the `variables-and-types` module. Covers: variable declarations, value types, reference types,
> type system, nullable types, default values, and the CLR type system.

---

## Variables

Source: [Variables (C# Language Specification)](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/variables)

A **variable** represents a storage location. Variables have a type that determines what values can be stored.

### Variable Categories

| Category | Description | Lifetime |
|---|---|---|
| Static variables | Class-level, single instance | Program lifetime |
| Instance variables | Object fields | Object lifetime |
| Array elements | Elements of an array | Array lifetime |
| Value parameters | Method parameters (copied) | Method execution |
| Reference parameters | `ref` parameters (aliased) | Method execution |
| Output parameters | `out` parameters | Method execution |
| Local variables | Declared in methods/blocks | Block scope |

---

## Local Variable Declarations

Source: [Local Variable Declarations](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/statements#1341-local-variable-declarations)

```csharp
int x;                          // declaration (uninitialized)
int y = 42;                     // with initializer
var z = 100;                    // type inference (local variables only)

int a = 1, b = 2, c = 3;        // multiple declarations
```

### Implicitly Typed Local Variables (`var`)

Source: [Implicitly Typed Local Variables](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/declarations#implicitly-typed-local-variables)

The `var` keyword infers the type from the initializer expression:

```csharp
var count = 42;                 // int
var name = "Alice";             // string
var list = new List<int>();     // List<int>
```

**Rules:**
- Only for local variables (not fields, parameters, return types)
- Must have an initializer
- Type is determined at compile time (still statically typed)
- Cannot infer from `null`

---

## Type System Overview

Source: [Types (C# Language Specification)](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/types)

C# has two categories of types:

### Value Types

- **Simple types**: `int`, `bool`, `char`, `float`, `double`, `decimal`, etc.
- **Enum types**: User-defined enumerations
- **Struct types**: User-defined value types
- **Nullable value types**: `T?` where `T` is a value type
- **Tuple types**: `(T1, T2, ...)`

**Characteristics:**
- Stored in-place (usually on the stack)
- Assignment copies the value
- Cannot be `null` (unless wrapped in `Nullable<T>`)
- Inherit from `System.ValueType`

### Reference Types

- **Class types**: `object`, `string`, user-defined classes
- **Interface types**: Abstract contracts
- **Array types**: Single/multi-dimensional, jagged
- **Delegate types**: Function pointers

**Characteristics:**
- Stored on the heap
- Assignment copies the reference (pointer)
- Can be `null`
- Inherit from `System.Object`

---

## Value Types

Source: [Value Types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/types#83-value-types)

### Simple Types

| Type | .NET Type | Size | Range | Default |
|---|---|---|---|---|
| `sbyte` | `System.SByte` | 1 byte | -128 to 127 | `0` |
| `byte` | `System.Byte` | 1 byte | 0 to 255 | `0` |
| `short` | `System.Int16` | 2 bytes | -32,768 to 32,767 | `0` |
| `ushort` | `System.UInt16` | 2 bytes | 0 to 65,535 | `0` |
| `int` | `System.Int32` | 4 bytes | -2,147,483,648 to 2,147,483,647 | `0` |
| `uint` | `System.UInt32` | 4 bytes | 0 to 4,294,967,295 | `0` |
| `long` | `System.Int64` | 8 bytes | -2^63 to 2^63-1 | `0L` |
| `ulong` | `System.UInt64` | 8 bytes | 0 to 2^64-1 | `0UL` |
| `char` | `System.Char` | 2 bytes | U+0000 to U+FFFF (UTF-16 code unit) | `'\0'` |
| `float` | `System.Single` | 4 bytes | IEEE 754 single-precision | `0.0f` |
| `double` | `System.Double` | 8 bytes | IEEE 754 double-precision | `0.0` |
| `decimal` | `System.Decimal` | 16 bytes | 128-bit fixed-point | `0.0m` |
| `bool` | `System.Boolean` | 1 byte | `true` or `false` | `false` |

### Integral Literals

```csharp
int dec = 42;                   // decimal
int hex = 0x2A;                 // hexadecimal
int bin = 0b101010;             // binary (C# 7+)
long big = 1_000_000L;          // digit separators (C# 7+)
```

### Floating-Point Literals

```csharp
double d = 3.14;                // double (default)
float f = 3.14f;                // float (f/F suffix)
decimal m = 3.14m;              // decimal (m/M suffix)

double sci = 1.23e-10;          // scientific notation
```

### Character Literals

```csharp
char c = 'A';
char escape = '\n';             // escape sequences
char unicode = '\u0041';        // Unicode escape (A)
char hex = '\x41';              // Hexadecimal escape (A)
```

---

## Reference Types

Source: [Reference Types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/types#84-reference-types)

### The `object` Type

The ultimate base type of all types. All types (value and reference) derive from `object` (`System.Object`).

```csharp
object obj = 42;                // boxing — int wrapped in object
object str = "hello";           // string is already a reference type
```

### The `string` Type

Source: [String Type](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/reference-types#the-string-type)

- Represents a sequence of Unicode characters (UTF-16)
- **Immutable** — cannot be changed after creation
- Reference type but behaves like a value type (immutability + interning)

```csharp
string s = "hello";
string t = "hello";             // refers to same interned string

string concat = s + " world";   // creates new string
```

**String Literals:**

```csharp
string regular = "Line 1\nLine 2";
string verbatim = @"Line 1
Line 2";                        // verbatim (preserves newlines, escapes backslashes)
string interpolated = $"Value: {x}";  // string interpolation
string raw = """
{
  "key": "value"
}
""";                            // raw string literal (C# 11+)
```

---

## Nullable Types

Source: [Nullable Value Types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/nullable-value-types)

### Nullable Value Types

A nullable value type `T?` can represent all values of `T` plus `null`.

```csharp
int? maybeCount = null;         // Nullable<int>
int? another = 42;

if (maybeCount.HasValue) {
    int value = maybeCount.Value;  // extract value
}

// Null-coalescing operator
int count = maybeCount ?? 0;    // use 0 if null

// Null-conditional operator
int? length = maybeCount?.ToString().Length;  // null if maybeCount is null
```

### Nullable Reference Types (C# 8.0+)

Source: [Nullable Reference Types](https://learn.microsoft.com/en-us/dotnet/csharp/nullable-references)

Opt-in feature that adds compile-time null safety for reference types:

```csharp
#nullable enable

string name = null;             // Warning — non-nullable reference type
string? maybeName = null;       // OK — explicitly nullable

void Greet(string name) {       // name is non-null
    Console.WriteLine(name.ToUpper());  // safe — no null check needed
}

void Process(string? input) {   // input is nullable
    if (input is not null) {
        Console.WriteLine(input.ToUpper());  // safe after null check
    }
}
```

**Nullable annotations:**
- `T` — non-nullable (default when enabled)
- `T?` — nullable
- `T!` — null-forgiving operator (suppress warning)

---

## Default Values

Source: [Default Values](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/default-values)

Every type has a **default value** used when a variable is not explicitly initialized.

| Type | Default Value |
|---|---|
| Numeric types | `0` |
| `bool` | `false` |
| `char` | `'\0'` |
| `enum` | `0` (cast to enum type) |
| Reference types | `null` |
| Nullable value types | `null` |
| Structs | All fields set to their default values |

```csharp
int x = default;                // 0
string s = default;             // null
bool b = default;               // false
```

**Instance fields** are automatically initialized to default values. **Local variables** must be explicitly initialized before use.

---

## Boxing and Unboxing

Source: [Boxing and Unboxing](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/types/boxing-and-unboxing)

**Boxing:** Converting a value type to `object` or an interface type. Allocates an object on the heap.

```csharp
int x = 42;
object obj = x;                 // boxing — allocates on heap
```

**Unboxing:** Extracting the value type from the boxed object.

```csharp
int y = (int)obj;               // unboxing — requires explicit cast
```

**Performance:** Boxing/unboxing has overhead (heap allocation, type checking). Modern code avoids it using generics.

```csharp
// Old way (boxing)
ArrayList list = new ArrayList();
list.Add(42);                   // boxes int to object

// Modern way (no boxing)
List<int> list = new List<int>();
list.Add(42);                   // no boxing with generics
```

---

## Conversions

Source: [Conversions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/conversions)

### Implicit Conversions

Conversions that always succeed and never lose information:

```csharp
int i = 42;
long l = i;                     // implicit widening (int → long)
float f = i;                    // implicit (int → float)
```

### Explicit Conversions (Casts)

Conversions that may lose information or fail at runtime:

```csharp
long l = 1000;
int i = (int)l;                 // explicit narrowing (may overflow)

double d = 3.14;
int x = (int)d;                 // truncates to 3
```

### Checked vs Unchecked

```csharp
int max = int.MaxValue;

unchecked {
    int overflow = max + 1;     // wraps around to int.MinValue
}

checked {
    int overflow = max + 1;     // throws OverflowException
}
```

---

## Constants

Source: [Constants](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/constants)

Constants are compile-time values that cannot change:

```csharp
const int MAX_SIZE = 100;
const string DEFAULT_NAME = "Unknown";

// MAX_SIZE = 200;              // compile error
```

**Rules:**
- Must be initialized at declaration
- Value must be determinable at compile time
- Implicitly `static`
- Can only be primitive types or `string`

---

## readonly Fields

Source: [readonly Keyword](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/readonly)

`readonly` fields can be assigned at declaration or in a constructor, but not after:

```csharp
class Config {
    public readonly string Path;
    public readonly int MaxRetries = 3;  // can initialize here

    public Config(string path) {
        Path = path;                     // or in constructor
    }

    public void Update(string newPath) {
        // Path = newPath;               // compile error
    }
}
```

**`const` vs `readonly`:**

| `const` | `readonly` |
|---|---|
| Compile-time constant | Runtime constant |
| Implicitly static | Can be instance or static |
| Must be primitive or string | Any type |
| Inlined at call sites | Read from memory |

---

## Struct Types

Source: [Structure Types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/struct)

User-defined value types:

```csharp
struct Point {
    public int X;
    public int Y;

    public Point(int x, int y) {
        X = x;
        Y = y;
    }
}

Point p1 = new Point(10, 20);
Point p2 = p1;                  // copy — p2 is independent
p2.X = 100;
// p1.X is still 10
```

**Characteristics:**
- Value type (copied on assignment)
- Cannot inherit from another struct or class
- Can implement interfaces
- Cannot have a parameterless constructor (before C# 10)
- All fields must be assigned before use

---

## Record Types (C# 9+)

Source: [Records](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/record)

Immutable reference types with value semantics for equality:

```csharp
record Person(string Name, int Age);

Person p1 = new Person("Alice", 30);
Person p2 = new Person("Alice", 30);

p1 == p2;                       // true — value equality
p1.Name = "Bob";                // compile error — immutable

Person p3 = p1 with { Age = 31 };  // non-destructive mutation
```

---

## Tuple Types

Source: [Tuple Types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/value-tuples)

Lightweight value types for grouping values:

```csharp
(int, string) tuple = (42, "hello");
int number = tuple.Item1;
string text = tuple.Item2;

// Named tuple elements
(int Count, string Name) named = (42, "Alice");
Console.WriteLine(named.Count);

// Deconstruction
(int count, string name) = named;
```

---

## The `dynamic` Type

Source: [dynamic Type](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/reference-types#the-dynamic-type)

Bypasses compile-time type checking — resolved at runtime:

```csharp
dynamic d = 42;
d = "hello";                    // valid — type changes at runtime
d.SomeMethod();                 // resolved at runtime (may fail)
```

**Use sparingly** — defeats the purpose of static typing.

---

## Notes

- **Value vs Reference:** Fundamental distinction affecting performance and semantics
- **Nullable Reference Types:** Opt-in null safety (C# 8+), similar to TypeScript's `strictNullChecks`
- **`decimal` for money:** Use `decimal` (not `double`) for financial calculations to avoid rounding errors
- **UTF-16:** `char` and `string` use UTF-16. Characters outside the BMP (like emoji) require surrogate pairs.
- **Interning:** String literals are automatically interned for performance
