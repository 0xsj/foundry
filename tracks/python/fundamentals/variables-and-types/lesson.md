# Variables and Types — Python

## How Variables Work Under the Hood

### Everything Is an Object

In Python, every value is an object on the heap. There are no stack-allocated primitives like in Go or Rust. Even `42` is an object with a type, a reference count, and methods.

```python
x = 42
print(type(x))       # <class 'int'>
print(id(x))         # memory address of the int object
print(x.bit_length()) # 6 — yes, you can call methods on integers
```

### Variables Are Name Tags, Not Boxes

A Python variable is **not** a location that holds a value. It's a **name** that points to an object. This is fundamentally different from Go/Rust/Java.

```python
a = [1, 2, 3]
b = a              # b points to the SAME list object, not a copy
b.append(4)
print(a)           # [1, 2, 3, 4] — both names reference the same object
print(id(a) == id(b))  # True — same address
```

```
Names:              Objects (heap):
a ──────────┐
            ├──→  [1, 2, 3, 4]   (id: 0x7f...)
b ──────────┘
```

Assignment (`=`) never copies data. It creates a new reference to an existing object.

### Reference Counting + GC

CPython (the standard implementation) uses:
1. **Reference counting**: each object tracks how many names point to it. When the count hits 0, it's freed immediately.
2. **Cycle collector**: catches reference cycles that reference counting can't handle.

```python
import sys

x = "hello"
print(sys.getrefcount(x))  # shows reference count (will be at least 2 —
                            # one for x, one for the function argument)
```

### Your notes
<!-- -->


---

## Type System

### Dynamic, Strong, Duck-Typed

Python's type system is:
- **Dynamic**: types are checked at runtime, not compile time
- **Strong**: no implicit coercion between unrelated types (`"3" + 4` raises TypeError)
- **Duck-typed**: "if it quacks like a duck..." — no interfaces needed, just call the method

```python
x = 42          # x is an int right now
x = "hello"     # now x is a str — Python doesn't care

# Strong typing means this fails:
# result = "port: " + 8080   # TypeError: can't concat str and int
result = "port: " + str(8080) # must convert explicitly
```

Wait — Python is "strong" but also does this:
```python
True + 1    # 2 — bool is a subclass of int
1.0 + 2    # 3.0 — numeric tower: int promotes to float
```

These aren't exceptions to strong typing — `bool` literally inherits from `int`, and the numeric types have defined promotion rules.

### Built-in Types

| Type | Mutable | Notes |
|---|---|---|
| `int` | No | Arbitrary precision — no overflow! |
| `float` | No | 64-bit IEEE 754 double |
| `bool` | No | Subclass of int. `True == 1`, `False == 0` |
| `str` | No | Unicode (UTF-8 internal). Immutable. |
| `bytes` | No | Raw byte sequence |
| `None` | — | Singleton. The "null" of Python. |
| `list` | Yes | Dynamic array |
| `tuple` | No | Immutable sequence |
| `dict` | Yes | Hash map |
| `set` | Yes | Hash set |

### Immutable Does Not Mean Unchangeable References

```python
x = (1, [2, 3], 4)   # tuple containing a list
# x[0] = 99          # TypeError — can't change tuple elements
x[1].append(5)        # works! The list inside is still mutable
print(x)              # (1, [2, 3, 5], 4)
```

The tuple's references are fixed. The objects those references point to can still change (if they're mutable).

### Your notes
<!-- -->


---

## Integer Internals

Python integers have arbitrary precision — they never overflow:

```python
x = 2 ** 1000   # perfectly fine — Python allocates as many bytes as needed
print(x)         # a very large number
```

Under the hood, CPython stores large ints as arrays of "digits" (each 30 bits). This means:
- Small ints are fast (CPython pre-allocates -5 to 256)
- Large ints are slow (arbitrary precision math)
- There is no `int8`, `int32`, `int64` distinction — just `int`

```python
# Small int caching:
a = 256
b = 256
print(a is b)    # True — same object (cached)

a = 257
b = 257
print(a is b)    # False (usually) — different objects
```

### `is` vs `==`

- `==` compares values
- `is` compares identity (are they the same object in memory?)

```python
a = [1, 2, 3]
b = [1, 2, 3]
print(a == b)    # True — same content
print(a is b)    # False — different objects

c = None
print(c is None) # True — None is a singleton, always use `is` for None checks
```

### Your notes
<!-- -->


---

## Type Hints (Not Enforcement)

Python 3.5+ supports type hints. They are **completely ignored at runtime**.

```python
x: int = "hello"   # no error at runtime — Python doesn't check
print(x)            # "hello" — works fine
```

Type hints exist for:
- Documentation
- Editor/IDE support (autocomplete, hover info)
- Static analysis tools like `mypy`

```python
def add(a: int, b: int) -> int:
    return a + b

add("hello", "world")  # runs fine — returns "helloworld"
                        # mypy would catch this, Python won't
```

To get actual runtime type checking, you need libraries like `pydantic` or `beartype`.

### Your notes
<!-- -->


---

## The bool Trap

```python
bool("false")    # True — non-empty string is truthy
bool("")         # False — empty string is falsy
bool(0)          # False
bool(1)          # True
bool([])         # False — empty collection
bool([0])        # True — non-empty collection (even if contents are falsy)
bool(None)       # False
```

This matters when parsing config values. `"false"` is truthy because it's a non-empty string. You must handle bool parsing explicitly:

```python
def parse_bool(s: str) -> bool:
    if s.lower() in ("true", "1", "yes"):
        return True
    if s.lower() in ("false", "0", "no"):
        return False
    raise ValueError(f"invalid bool: {s}")
```

### Your notes
<!-- -->


---

## Composite Types (Preview)

| Type | Syntax | Mutable | Notes |
|---|---|---|---|
| `list` | `[1, 2, 3]` | Yes | Dynamic array |
| `tuple` | `(1, 2, 3)` | No | Immutable sequence |
| `dict` | `{"a": 1}` | Yes | Hash map (ordered since 3.7) |
| `set` | `{1, 2, 3}` | Yes | Unique elements |
| `frozenset` | `frozenset({1,2})` | No | Immutable set |
| `NamedTuple` | `Point = namedtuple(...)` | No | Tuple with named fields |
| `dataclass` | `@dataclass` | Configurable | Struct-like |

### Your notes
<!-- -->
