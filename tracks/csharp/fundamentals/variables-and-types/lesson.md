# Variables and Types — C#

## How Variables Work Under the Hood

### Declaration and Memory

When you write `int x = 42;`, here's what happens:

1. The CLR (Common Language Runtime) allocates space — stack for value types, heap for reference types
2. The size depends on the type: `int` is always 4 bytes (32-bit), `long` is 8 bytes
3. The value `42` is written into that memory
4. The name `x` is compile-time only — at runtime it's just a memory location

```csharp
int x = 42;
Console.WriteLine($"value: {x}, size: {sizeof(int)} bytes");
// Output: value: 42, size: 4 bytes
```

### Stack vs Heap

C# has a fundamental distinction that affects performance and behavior:

- **Value types** (structs, primitives) — stored on the stack, copied on assignment
- **Reference types** (classes, strings, arrays) — stored on the heap, reference copied on assignment

```csharp
// Value type — lives on the stack
int a = 10;
int b = a;      // b gets a COPY of the value
b = 20;         // changing b doesn't affect a
// a is still 10

// Reference type — object lives on the heap
var list1 = new List<int> { 1, 2, 3 };
var list2 = list1;   // list2 refers to the SAME object
list2.Add(4);        // mutates the shared object
// list1 now contains [1, 2, 3, 4]
```

**Key insight:** In C#, value types act like Go's default behavior (copy), while reference types act like JavaScript's default behavior (shared reference).

### Your notes
<!-- -->


---

## Type System

### Static, Nominal, Strong

C#'s type system is:
- **Static**: Types are checked at compile time
- **Nominal**: Types match by name, not structure (unlike Go interfaces or TypeScript)
- **Strong**: No implicit conversions between incompatible types

```csharp
int x = 42;
long y = x;           // OK — implicit widening conversion
int z = y;            // ERROR — requires explicit cast
int w = (int)y;       // OK — explicit cast
```

### Type Inference with `var`

C# supports type inference for local variables:

```csharp
var count = 42;                 // inferred as int
var name = "Alice";             // inferred as string
var list = new List<string>();  // inferred as List<string>

// var doesn't make C# dynamic — type is still fixed at compile time
count = "hello";                // ERROR — count is int
```

**When to use `var`:**
- Type is obvious from the right side
- Reduces verbosity

**When NOT to use:**
- Return type isn't clear from method name
- Harms readability

### Your notes
<!-- -->


---

## Primitive Types (Value Types)

C# has built-in value types that map to .NET types:

| C# Type | .NET Type | Size | Range | Default |
|---|---|---|---|---|
| `bool` | `Boolean` | 1 byte | `true` or `false` | `false` |
| `byte` | `Byte` | 1 byte | 0 to 255 | `0` |
| `sbyte` | `SByte` | 1 byte | -128 to 127 | `0` |
| `short` | `Int16` | 2 bytes | -32,768 to 32,767 | `0` |
| `ushort` | `UInt16` | 2 bytes | 0 to 65,535 | `0` |
| `int` | `Int32` | 4 bytes | -2^31 to 2^31-1 | `0` |
| `uint` | `UInt32` | 4 bytes | 0 to 2^32-1 | `0` |
| `long` | `Int64` | 8 bytes | -2^63 to 2^63-1 | `0L` |
| `ulong` | `UInt64` | 8 bytes | 0 to 2^64-1 | `0` |
| `float` | `Single` | 4 bytes | IEEE 754 single | `0.0f` |
| `double` | `Double` | 8 bytes | IEEE 754 double | `0.0d` |
| `decimal` | `Decimal` | 16 bytes | 128-bit high precision | `0.0m` |
| `char` | `Char` | 2 bytes | UTF-16 code unit | `'\0'` |

**Key differences from other languages:**

- **`decimal`** — Fixed-point type for financial calculations (no floating-point rounding errors)
- **Unsigned types** — C# has them, unlike Java
- **`char`** — Always UTF-16, unlike Rust's Unicode scalar value

```csharp
int count = 42;
long big = 1_000_000_000L;      // underscores for readability
float ratio = 0.5f;             // f suffix required
double pi = 3.14159;            // default for decimals
decimal price = 99.99m;         // m suffix for decimal
```

### Your notes
<!-- -->


---

## Strings Are Reference Types (Immutable)

In C#, `string` is a reference type (lives on the heap) but behaves like a value type in some ways:

```csharp
string s = "hello";
// s is a reference to a string object on the heap
```

**Immutability:** Like Go and Rust strings, C# strings cannot be modified in place:

```csharp
string name = "Alice";
string upper = name.ToUpper();  // Returns NEW string, doesn't modify name
// name is still "Alice"
```

**String interning:** String literals are interned (reused):

```csharp
string a = "hello";
string b = "hello";
// a and b refer to the SAME object in memory (interned)
```

### String Interpolation

C# has excellent string formatting:

```csharp
string name = "Alice";
int age = 30;

// String interpolation (preferred)
string message = $"Hello, {name}! You are {age} years old.";

// Format expressions
string formatted = $"Pi to 2 decimals: {Math.PI:F2}";  // 3.14

// Verbatim strings (escape backslashes)
string path = @"C:\Users\Alice\Documents";

// Raw string literals (C# 11+)
string json = """
{
  "name": "Alice",
  "age": 30
}
""";
```

### Your notes
<!-- -->


---

## Nullable Types

By default, **value types cannot be null**. Reference types can.

```csharp
int x = null;           // ERROR — int cannot be null
string s = null;        // OK — string is a reference type
```

**Nullable value types:** Use `?` to make a value type nullable:

```csharp
int? maybeCount = null;     // Nullable<int>
if (maybeCount.HasValue) {
    int value = maybeCount.Value;
}

// Null-coalescing operator
int count = maybeCount ?? 0;    // use 0 if null
```

**Nullable reference types (C# 8+):** Opt-in null safety for reference types:

```csharp
#nullable enable

string name = null;         // WARNING — non-nullable reference type
string? maybeName = null;   // OK — explicitly nullable

void Greet(string name) {   // name is non-null
    Console.WriteLine($"Hello, {name.ToUpper()}");  // safe
}
```

This is similar to TypeScript's `strictNullChecks` — opt-in compile-time null safety.

### Your notes
<!-- -->


---

## Default Values

Every type has a default value. For value types, it's zero/false. For reference types, it's `null`.

| Type | Default |
|---|---|
| Numeric types | `0` |
| `bool` | `false` |
| `char` | `'\0'` |
| Reference types | `null` |
| Nullable value types | `null` |

```csharp
int x = default;        // 0
string s = default;     // null
bool b = default;       // false

// Arrays are initialized to default values
int[] numbers = new int[5];  // [0, 0, 0, 0, 0]
```

### Your notes
<!-- -->


---

## Value Types vs Reference Types (Deep Dive)

### Value Types (Stack-Allocated)

- Primitives (`int`, `bool`, `double`, etc.)
- `struct` types (custom value types)
- Enums
- Tuples (`(int, string)`)

**Behavior:**
- Assigned by **copying the value**
- Live on the stack (usually — can be boxed to heap)
- No garbage collection needed
- Cannot be `null` (unless wrapped in `Nullable<T>`)

```csharp
struct Point {  // value type
    public int X;
    public int Y;
}

Point p1 = new Point { X = 10, Y = 20 };
Point p2 = p1;      // COPY — p2 gets its own independent values
p2.X = 100;
// p1.X is still 10
```

### Reference Types (Heap-Allocated)

- `class` types
- `string`
- Arrays
- Delegates

**Behavior:**
- Assigned by **copying the reference** (pointer)
- Live on the heap
- Garbage collected
- Can be `null`

```csharp
class Person {  // reference type
    public string Name { get; set; }
}

Person p1 = new Person { Name = "Alice" };
Person p2 = p1;     // REFERENCE copy — p2 points to same object
p2.Name = "Bob";
// p1.Name is now "Bob" too
```

### Boxing and Unboxing

**Boxing:** Converting a value type to `object` (allocates on heap)

```csharp
int x = 42;
object obj = x;     // boxing — copies x to heap
```

**Unboxing:** Converting `object` back to value type

```csharp
int y = (int)obj;   // unboxing — copies back to stack
```

**Performance:** Boxing/unboxing is expensive. Modern C# avoids it with generics.

### Your notes
<!-- -->


---

## Constants and readonly

```csharp
const int MAX_SIZE = 100;           // compile-time constant
MAX_SIZE = 200;                     // ERROR — cannot reassign

readonly string ConfigPath;         // runtime constant (set in constructor)

public MyClass() {
    ConfigPath = "/etc/config";     // OK in constructor
}
```

**`const` vs `readonly`:**

| `const` | `readonly` |
|---|---|
| Must be initialized at declaration | Can be initialized in constructor |
| Compile-time constant | Runtime constant |
| Implicitly `static` | Can be instance or static |
| Only primitives and strings | Any type |

### Your notes
<!-- -->


---

## Composite Types (Preview)

These get their own lessons, but here's a quick map:

| Type | Description | Value or Reference |
|---|---|---|
| `int[]` | Array — fixed size | Reference |
| `List<T>` | Dynamic list | Reference |
| `Dictionary<K,V>` | Hash map | Reference |
| `struct` | Custom value type | Value |
| `class` | Custom reference type | Reference |
| `(int, string)` | Tuple — ad-hoc grouping | Value |
| `record` | Immutable reference type (C# 9+) | Reference |

### Your notes
<!-- -->


---

## Comparison to Other Languages

Coming from JavaScript/TypeScript:

| JavaScript/TypeScript | C# |
|---|---|
| `let x = 42` | `var x = 42;` or `int x = 42;` |
| Everything is a reference (except primitives) | Explicit value vs reference types |
| `null` and `undefined` | Just `null` (no undefined) |
| Weak typing (JS) / structural (TS) | Strong, nominal typing |
| Automatic semicolon insertion | Semicolons required |

Coming from Go:

| Go | C# |
|---|---|
| `x := 42` | `var x = 42;` |
| Pointers explicit (`*int`) | References implicit (for reference types) |
| `nil` | `null` |
| Structs are value types | `struct` are value types, `class` are reference |
| No exceptions (errors as values) | Exceptions (`try/catch`) |

### Your notes
<!-- -->
