# Python Reference — Variables and Types

> Extracted from the [Python Language Reference](https://docs.python.org/3/reference/)
> and [Python Data Model](https://docs.python.org/3/reference/datamodel.html)
> for the `variables-and-types` module. Covers: variable assignment, naming rules,
> dynamic typing, primitive types, None, type hints, and the object model.

---

## Assignment Statements

Source: [reference/simple_stmts.html#assignment-statements](https://docs.python.org/3/reference/simple_stmts.html#assignment-statements)

Assignment in Python binds a name to an object. Names are not typed — the object holds the type information.

```python
x = 42              # x now refers to an int object
x = "hello"         # x now refers to a str object (rebinding)
```

### Multiple Assignment

```python
a, b, c = 1, 2, 3                    # tuple unpacking
x = y = z = 0                        # chain assignment (all refer to same object)
first, *rest = [1, 2, 3, 4]          # extended unpacking (first=1, rest=[2,3,4])
```

### Augmented Assignment

```python
x += 1              # equivalent to x = x + 1
x *= 2              # equivalent to x = x * 2
```

Augmented assignment may modify the object in-place (for mutable types) or create a new object (for immutable types).

---

## The Python Data Model

Source: [reference/datamodel.html](https://docs.python.org/3/reference/datamodel.html)

Every object in Python has:
- An **identity** (unique, never changes, returned by `id()`)
- A **type** (determines operations, never changes, returned by `type()`)
- A **value** (may be mutable or immutable)

### Object Identity and Equality

| Operator | Meaning | Notes |
|---|---|---|
| `is` | Identity comparison | Tests if two names refer to the same object |
| `==` | Value equality | Tests if two objects have equal values |

```python
a = [1, 2, 3]
b = a             # b refers to the same list object
c = [1, 2, 3]     # c refers to a different list object with equal value

a is b            # True — same object
a is c            # False — different objects
a == c            # True — equal values
```

---

## Naming Rules

Source: [reference/lexical_analysis.html#identifiers](https://docs.python.org/3/reference/lexical_analysis.html#identifiers)

- **Valid characters**: letters (Unicode), digits, underscore (`_`)
- **Cannot start with**: digit
- **Case-sensitive**: `foo` and `Foo` are different names
- **Reserved words**: Cannot use keywords (`if`, `for`, `class`, etc.)

### Naming Conventions (PEP 8)

| Convention | Example | Use |
|---|---|---|
| `snake_case` | `user_count` | Variables, functions |
| `UPPER_CASE` | `MAX_SIZE` | Constants |
| `PascalCase` | `UserAccount` | Classes |
| `_leading_underscore` | `_internal` | Internal/private by convention |
| `__double_leading` | `__private` | Name mangling for class attributes |

---

## Dynamic Typing

Python is **dynamically typed** — names do not have types, only objects do. Type checking happens at runtime.

```python
x = 42              # x refers to an int
x = "hello"         # valid — x now refers to a str
x.upper()           # valid — str has .upper()
x + 10              # TypeError at runtime — str + int not allowed
```

This is in contrast to statically typed languages (Go, Rust, Java) where variable types are fixed at compile time.

---

## Primitive Types

### Numeric Types

Source: [library/stdtypes.html#numeric-types-int-float-complex](https://docs.python.org/3/library/stdtypes.html#numeric-types-int-float-complex)

| Type | Description | Example |
|---|---|---|
| `int` | Arbitrary-precision integer | `42`, `10**100`, `0xFF`, `0b1010` |
| `float` | IEEE 754 double-precision (64-bit) | `3.14`, `1.0e-10`, `float('inf')` |
| `complex` | Complex number | `3+4j`, `complex(3, 4)` |

**Key property:** `int` has unlimited precision (no overflow).

```python
x = 10 ** 100  # 100-digit number — no problem
```

### Boolean Type

Source: [library/stdtypes.html#boolean-type-bool](https://docs.python.org/3/library/stdtypes.html#boolean-type-bool)

- Two values: `True` and `False`
- Subclass of `int`: `True == 1`, `False == 0`
- Many objects have "truthiness" in boolean context

**Falsy values:**
- `None`, `False`
- Zero: `0`, `0.0`, `0j`
- Empty sequences: `""`, `[]`, `()`, `{}`
- Empty mappings: `{}`

**Everything else is truthy.**

### None Type

Source: [library/stdtypes.html#the-null-object](https://docs.python.org/3/library/stdtypes.html#the-null-object)

- `None` is the sole value of type `NoneType`
- Represents absence of value
- Used as default return value for functions with no explicit `return`

```python
x = None

if x is None:  # idiomatic check
    print("no value")
```

**Important:** Use `is None`, not `== None` (identity check is faster and clearer).

---

## String Type

Source: [library/stdtypes.html#text-sequence-type-str](https://docs.python.org/3/library/stdtypes.html#text-sequence-type-str)

- Immutable sequence of Unicode code points
- Represented internally as UTF-8, UTF-16, or UTF-32 depending on content (CPython optimization)

```python
s = "hello"
s = 'hello'           # single or double quotes are equivalent
s = """multiline
string"""             # triple quotes for multiline

# String operations
len(s)                # length (number of characters, not bytes)
s[0]                  # indexing (returns single-character string)
s[1:4]                # slicing
s + " world"          # concatenation (creates new string)
f"Hello, {name}"      # f-strings (formatted string literals)
```

**Immutability:** Cannot modify a string in place. All string methods return new strings.

---

## Sequence Types (Preview)

Source: [library/stdtypes.html#sequence-types-list-tuple-range](https://docs.python.org/3/library/stdtypes.html#sequence-types-list-tuple-range)

### List

- Mutable sequence
- Heterogeneous (can contain different types)
- Dynamic size

```python
items = [1, 2, 3]
items.append(4)       # mutates the list
items[0] = 10         # item assignment
```

### Tuple

- Immutable sequence
- Heterogeneous
- Fixed size after creation

```python
point = (10, 20)
x, y = point          # unpacking
```

### Range

- Immutable sequence of numbers
- Generated lazily (doesn't store all values)

```python
r = range(10)         # 0, 1, 2, ..., 9
r = range(5, 10)      # 5, 6, 7, 8, 9
r = range(0, 10, 2)   # 0, 2, 4, 6, 8
```

---

## Mapping Type: dict

Source: [library/stdtypes.html#mapping-types-dict](https://docs.python.org/3/library/stdtypes.html#mapping-types-dict)

- Mutable mapping of keys to values
- Keys must be hashable (immutable types: `str`, `int`, `tuple`)
- Insertion order preserved (Python 3.7+)

```python
config = {"host": "localhost", "port": 8080}
config["host"]        # access
config["timeout"] = 30  # insert/update
"host" in config      # membership test
```

---

## Type Hints (Optional)

Source: [library/typing.html](https://docs.python.org/3/library/typing.html)

Python 3.5+ supports optional type annotations. These are **not enforced at runtime** but can be checked by tools like `mypy`, `pyright`, or IDEs.

```python
def greet(name: str) -> str:
    return f"Hello, {name}"

x: int = 42
users: list[str] = ["alice", "bob"]
config: dict[str, int] = {"port": 8080}
```

### Common Type Hints

| Hint | Meaning |
|---|---|
| `int`, `str`, `float`, `bool` | Built-in types |
| `list[T]` | List of type T |
| `dict[K, V]` | Dictionary with keys K, values V |
| `tuple[T, U]` | Tuple with specific types |
| `Optional[T]` | `T | None` (can be None) |
| `Union[T, U]` | Either T or U |
| `Any` | Any type (disables checking) |

---

## Variables vs Objects

### Key Mental Model

In Python, **variables are names, not storage locations**.

```python
x = [1, 2, 3]         # x is a name bound to a list object
y = x                 # y is another name for the same object
y.append(4)           # mutates the shared object
print(x)              # [1, 2, 3, 4] — x sees the change
```

Compare to languages with value semantics (Go, Rust) where `y = x` creates a copy.

### Reference Semantics

- Assignment creates a new reference to the same object
- Mutation affects all references
- Reassignment breaks the link

```python
a = [1, 2]
b = a                 # b and a refer to same list
b = [3, 4]            # b now refers to a different list
# a is still [1, 2]
```

---

## Mutable vs Immutable Types

| Immutable | Mutable |
|---|---|
| `int`, `float`, `bool`, `str`, `tuple`, `frozenset` | `list`, `dict`, `set`, `bytearray` |

**Immutable objects cannot be changed after creation.** Operations that appear to modify them return new objects.

```python
s = "hello"
s.upper()             # returns "HELLO" (new string)
print(s)              # still "hello"

s = s.upper()         # rebind s to the new string
```

**Mutable objects can be modified in place.**

```python
lst = [1, 2, 3]
lst.append(4)         # mutates lst
print(lst)            # [1, 2, 3, 4]
```

---

## Special Attributes

Every object has these:

- `__class__` — the object's type
- `__dict__` — namespace (for objects with attributes)
- `__doc__` — docstring

```python
x = 42
x.__class__           # <class 'int'>
type(x)               # <class 'int'> (same as __class__)
```

---

## The `del` Statement

Source: [reference/simple_stmts.html#the-del-statement](https://docs.python.org/3/reference/simple_stmts.html#the-del-statement)

Deletes a name binding (not the object — garbage collector handles that).

```python
x = 42
del x                 # x no longer exists in this scope
# print(x)            # NameError
```

The object `42` is not deleted — it's garbage collected when no references remain.

---

## Notes

- **Everything is an object**: Even functions, classes, and modules are first-class objects
- **No primitive/reference distinction**: Unlike Java, Python doesn't have "primitive types" — all values are objects
- **Garbage collection**: Automatic (reference counting + cycle detector)
- **Duck typing**: "If it walks like a duck and quacks like a duck, it's a duck" — runtime type checking based on capability, not declared type
