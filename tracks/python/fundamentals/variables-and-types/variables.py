# =============================================================================
# Variables and Types — Python
# =============================================================================
# Fill in each section. Run with: python variables.py
# Or just save and ask for a review.
# =============================================================================

# =============================================================================
# 1. BASIC ASSIGNMENT
# Python has no declaration keyword. Assignment IS declaration.
# =============================================================================
# a) a string


# b) an int


# c) a float


# d) a bool


# =============================================================================
# 2. TYPE HINTS
# Python 3 supports type hints, but they're NOT enforced at runtime.
# They're for documentation and tools like mypy.
# =============================================================================
# a) name: str = "redis"


# b) port: int = 6379


# c) declare a variable with a WRONG type hint (e.g. x: int = "hello")
#    does Python complain? why or why not?


# =============================================================================
# 3. DYNAMIC TYPING
# A variable can be reassigned to a completely different type.
# =============================================================================
# a) assign x = 42, then x = "hello" — this is legal in Python


# b) use type() to check the type at each step
#    print(type(x))


# =============================================================================
# 4. NONE
# Python's equivalent of null/nil/undefined.
# =============================================================================
# a) assign a variable to None


# b) check its type with type()


# c) check if it's None using `is None` (not ==). Why `is` instead of `==`?
#    (answer in a comment)


# =============================================================================
# 5. MULTIPLE ASSIGNMENT
# Python supports several shorthand assignment forms.
# =============================================================================
# a) a, b, c = 1, 2, 3  (tuple unpacking)


# b) x = y = z = 0  (chained assignment)


# c) swap two variables without a temp: a, b = b, a


# =============================================================================
# 6. CONSTANTS (sort of)
# Python has no true constants. Convention is ALL_CAPS.
# =============================================================================
# a) MAX_RETRIES = 3


# b) try reassigning it — Python won't stop you. How does this compare
#    to Go's `const` or Rust's `const`? (answer in a comment)


# =============================================================================
# 7. PRINT EVERYTHING
# Use print() and f-strings. Use type() to show types.
# Example: print(f"port = {port} (type: {type(port).__name__})")
# =============================================================================

print("done")
