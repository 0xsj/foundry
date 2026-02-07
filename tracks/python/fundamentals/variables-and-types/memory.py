import sys

# =============================================================================
# 1. EVERYTHING IS AN OBJECT — INSPECT WITH id() AND sys.getsizeof()
# =============================================================================

a = 42
b = 3.14
c = True
d = "hello"

print("=== Sizes and Addresses ===")
print(f"int    — value: {a}, id: {id(a)}, size: {sys.getsizeof(a)} bytes")
print(f"float  — value: {b}, id: {id(b)}, size: {sys.getsizeof(b)} bytes")
print(f"bool   — value: {c}, id: {id(c)}, size: {sys.getsizeof(c)} bytes")
print(f"str    — value: {d}, id: {id(d)}, size: {sys.getsizeof(d)} bytes")
# Q: Why is an int 28 bytes when it's "just 42"?
#    What's in those 28 bytes? (hint: object header, reference count, type pointer)
#    (answer here)

# =============================================================================
# 2. NAMES ARE REFERENCES, NOT BOXES
# =============================================================================

x = [1, 2, 3]
y = x  # y is another name for the SAME object

print("\n=== Reference Sharing ===")
print(f"x id: {id(x)}")
print(f"y id: {id(y)}")
print(f"same object? {x is y}")

y.append(4)
print(f"x after y.append(4): {x}")
# Q: Why did x change when we only modified y?
#    (answer here)

# =============================================================================
# 3. SMALL INTEGER CACHING
# =============================================================================

a = 256
b = 256
print("\n=== Integer Caching ===")
print(f"256: a is b? {a is b} (id a: {id(a)}, id b: {id(b)})")

a = 257
b = 257
print(f"257: a is b? {a is b} (id a: {id(a)}, id b: {id(b)})")
# Q: Why does Python cache small integers? What's the performance implication?
#    (answer here)

# =============================================================================
# 4. IMMUTABLE REBINDING vs MUTABLE MODIFICATION
# =============================================================================

x = 42
print("\n=== Rebinding ===")
print(f"x = 42, id: {id(x)}")

x = 43
print(f"x = 43, id: {id(x)}")
# Q: The id changed — x now points to a DIFFERENT object.
#    Compare this to Rust's shadowing. How are they similar/different?
#    (answer here)

name = "hello"
print(f"\nname = 'hello', id: {id(name)}")

name = name + " world"
print(f"name = 'hello world', id: {id(name)}")
# Q: Strings are immutable. Concatenation creates a NEW string object.
#    What are the performance implications of building strings in a loop?
#    (answer here)

# =============================================================================
# 5. REFERENCE COUNTING
# =============================================================================

print("\n=== Reference Counting ===")
a = [1, 2, 3]
print(f"refcount after creation: {sys.getrefcount(a)}")
# Note: getrefcount itself creates a temporary reference, so count is +1

b = a
print(f"refcount after b = a: {sys.getrefcount(a)}")

c = a
print(f"refcount after c = a: {sys.getrefcount(a)}")

del b
print(f"refcount after del b: {sys.getrefcount(a)}")

del c
print(f"refcount after del c: {sys.getrefcount(a)}")
# Q: When does the list actually get freed from memory?
#    (answer here)

# =============================================================================
# 6. MUTABLE DEFAULT ARGUMENT TRAP
# This is a classic Python gotcha that comes from how references work.
# =============================================================================

def append_to(item, target=[]):
    target.append(item)
    return target

print("\n=== Mutable Default Argument ===")
print(append_to(1))  # [1]
print(append_to(2))  # [1, 2] — wait, what?
print(append_to(3))  # [1, 2, 3] — the default list is SHARED across calls

# Q: Why does this happen? How does this connect to what you learned about
#    references above? What's the fix?
#    (answer here)
