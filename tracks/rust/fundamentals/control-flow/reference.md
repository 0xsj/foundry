# Rust Reference — Control Flow

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/) and
> [The Rust Programming Language](https://doc.rust-lang.org/book/) for the
> `control-flow` module. Covers: if/if-let/let-else expressions, loop/while/for,
> match expressions and patterns, break/continue/return.

---

## if and if-let Expressions

Source: [reference/expressions/if-expr.html](https://doc.rust-lang.org/reference/expressions/if-expr.html)

An `if` expression evaluates a **boolean** condition and branches accordingly.

- The condition must be of type `bool`. No implicit coercion from other types.
- Braces around each block are required.
- An `if` without `else` has type `()`. An `if`/`else` produces a value; both
  branches must have the same type.

```rust
if CONDITION { BLOCK } else if CONDITION { BLOCK } else { BLOCK }
```

### if let

An `if let` expression matches a **refutable pattern** against a scrutinee:

```rust
if let PATTERN = EXPRESSION { BLOCK } else { BLOCK }
```

- If the pattern matches, bindings are introduced into the `then` block scope.
- The `else` block, if present, does not have access to the bindings.
- Multiple patterns with `|`: `if let Some(x) | Some(x) = ...` (both sides must
  bind the same variables with the same types).

### let-else Statements

Source: [reference/statements.html](https://doc.rust-lang.org/reference/statements.html)

```rust
let PATTERN = EXPRESSION else { DIVERGING_BLOCK };
```

- The pattern is **refutable** (it can fail to match).
- If the pattern matches, bindings are introduced into the surrounding scope.
- If the pattern does not match, the `else` block runs. It **must diverge**
  (return type is `!`): `return`, `break`, `continue`, `panic!()`, or `loop {}`.

```rust
let Ok(config) = load_config("app.toml") else {
    eprintln!("fatal: cannot load config");
    std::process::exit(1);
};
// config is available here
```

---

## Loop Expressions

Source: [reference/expressions/loop-expr.html](https://doc.rust-lang.org/reference/expressions/loop-expr.html)

### loop

```rust
loop { BLOCK }
```

- Executes the block repeatedly until a `break` expression is reached.
- A `loop` expression has type `!` (never, because it never returns normally)
  unless `break` is used.
- When `break VALUE` is used, the loop expression evaluates to `VALUE`.
- `break` without a value breaks with `()`.

```rust
let result: i32 = loop {
    let n = compute();
    if n > 0 { break n; }
};
```

### while

```rust
while CONDITION { BLOCK }
```

- Re-evaluates the condition before each iteration.
- `while` always has type `()` — it cannot return a value via `break`.

### while let

```rust
while let PATTERN = EXPRESSION { BLOCK }
```

- Re-matches the pattern before each iteration.
- Terminates when the pattern does not match.
- Always has type `()`.

```rust
while let Some(top) = stack.pop() {
    println!("{}", top);
}
```

### for

```rust
for PATTERN in EXPRESSION { BLOCK }
```

- Iterates over a value that implements `IntoIterator`.
- `EXPRESSION` is evaluated once and converted to an iterator.
- `PATTERN` is matched against each item.
- Always has type `()`.
- The `break` and `continue` expressions are valid inside a `for` loop.

```rust
for (key, val) in &map {
    println!("{}: {}", key, val);
}
```

The `for` loop desugars to:

```rust
{
    let mut iter = IntoIterator::into_iter(EXPRESSION);
    loop {
        match iter.next() {
            Some(PATTERN) => { BLOCK },
            None => break,
        }
    }
}
```

---

## Loop Labels and Label-Qualified Expressions

Source: [reference/expressions/loop-expr.html#loop-labels](https://doc.rust-lang.org/reference/expressions/loop-expr.html#loop-labels)

- A loop expression may be labeled: `'label: loop { ... }`
- Labels start with `'` (apostrophe), followed by a non-keyword identifier.
- `break 'label` and `continue 'label` target the labeled loop.
- `break 'label VALUE` breaks the labeled loop with a value.

```rust
'outer: for i in 0..10 {
    for j in 0..10 {
        if i + j == 15 {
            break 'outer;
        }
    }
}
```

---

## break, continue, return

Source: [reference/expressions/loop-expr.html#break-expressions](https://doc.rust-lang.org/reference/expressions/loop-expr.html#break-expressions)

| Expression | Type | Notes |
|---|---|---|
| `break` | `!` | Exits the nearest enclosing loop |
| `break 'label` | `!` | Exits the labeled loop |
| `break VALUE` | `!` | Exits nearest loop; loop expression evaluates to `VALUE` |
| `break 'label VALUE` | `!` | Exits labeled loop with value |
| `continue` | `!` | Skips to the next iteration of the nearest loop |
| `continue 'label` | `!` | Skips to the next iteration of the labeled loop |
| `return` | `!` | Returns from the enclosing function |
| `return VALUE` | `!` | Returns `VALUE` from the enclosing function |

All of these have type `!` (the never type), which can coerce to any type.

---

## match Expressions

Source: [reference/expressions/match-expr.html](https://doc.rust-lang.org/reference/expressions/match-expr.html)

```rust
match SCRUTINEE {
    PATTERN [if GUARD] => EXPRESSION,
    PATTERN [if GUARD] => EXPRESSION,
    ...
}
```

- `SCRUTINEE` is the value being matched.
- Each **arm** has a pattern, an optional guard (`if CONDITION`), and an expression.
- Arms are tested in declaration order — the first matching arm executes.
- The `match` expression is **exhaustive**: the compiler requires that every
  possible value of the scrutinee type is covered.
- All arms must have the same type.
- A `match` arm body may be a block `{ ... }` for multi-statement arms.

### Match Guards

A guard is a boolean condition after the pattern:

```rust
match msg {
    Message::Retry(n) if n < 3 => retry(n),
    Message::Retry(_) => discard(),
    _ => process(),
}
```

- If the guard is `false`, the arm is skipped (not considered a match).
- The guard sees bindings from the pattern.
- The compiler **does not consider guards for exhaustiveness** — you may need a
  `_` wildcard arm even if guards would theoretically cover everything.

---

## Patterns

Source: [reference/patterns.html](https://doc.rust-lang.org/reference/patterns.html)

Patterns appear in `match`, `if let`, `while let`, `let`, function parameters,
and `for` loops.

### Pattern Categories

| Pattern | Syntax | Example |
|---|---|---|
| Wildcard | `_` | `_ => {}` |
| Literal | `42`, `true`, `"hello"` | `0 => "zero"` |
| Range | `a..=b` | `1..=9 => "digit"` |
| Identifier binding | `name` | `x => println!("{x}")` |
| Binding with `@` | `name @ PATTERN` | `n @ 1..=9 => ...` |
| Struct | `Struct { field, .. }` | `Point { x, y: 0 }` |
| Tuple struct | `TupleStruct(a, b)` | `Some(x)`, `Ok(v)` |
| Tuple | `(a, b, c)` | `(true, x)` |
| Slice | `[a, b, ..]` | `[first, ..]` |
| Reference | `&PATTERN` | `&x` |
| Or | `A \| B` | `Some(x) \| None` |

### Refutability

- **Irrefutable patterns** always match: `x`, `(a, b)`, `_`. Used in `let`.
- **Refutable patterns** may fail: `Some(x)`, `42`, `Ok(v)`. Used in `if let`,
  `while let`, and `let-else`.

Using a refutable pattern where irrefutable is required is a compile error, and
vice versa.

### Binding Modes

By default, patterns destructure by value (move). Patterns inside a `match` on
a reference automatically apply **match ergonomics** — the compiler adjusts
binding modes so you can often write:

```rust
let s = Some(String::from("hello"));
if let Some(x) = &s {
    // x: &String — automatically referenced
}
// s is still valid
```

---

## Ranges in Patterns

Source: [reference/patterns.html#range-patterns](https://doc.rust-lang.org/reference/patterns.html#range-patterns)

- `a..=b` — inclusive range pattern. Both endpoints required. Matches a..=b.
- Ranges are only valid for: numeric types, `char`.
- Half-open ranges (`a..b`, exclusive) are **not valid in patterns** (only in
  expressions). Use `a..=b` in match arms.

```rust
match byte {
    b'A'..=b'Z' => "uppercase",
    b'a'..=b'z' => "lowercase",
    b'0'..=b'9' => "digit",
    _ => "other",
}
```

---

## Struct Patterns

Source: [reference/patterns.html#struct-patterns](https://doc.rust-lang.org/reference/patterns.html#struct-patterns)

```rust
StructName { field1, field2: PATTERN, .. }
```

- `..` ignores remaining fields (required if not exhaustive over fields).
- Shorthand: `{ field }` binds the field to a name of the same identifier.
- Fields can be further destructured recursively.

---

## Binding with @

Source: [reference/patterns.html#identifier-patterns](https://doc.rust-lang.org/reference/patterns.html#identifier-patterns)

```rust
let n @ 1..=12 = month else {
    panic!("invalid month");
};
```

`@` binds the matched value to a name while also applying a subpattern. The
binding is only introduced if the subpattern matches.

---

## Operator Precedence in Patterns

Or-patterns (`|`) have lower precedence than `@`:

```rust
// Parses as: (x @ 1) | (2) — binds x only in the first alternative
match val {
    x @ 1 | 2 => println!("{}", x),  // x not bound in the 2 case
    _ => {}
}

// To bind both:
match val {
    x @ (1 | 2) => println!("{}", x), // x bound in both cases
    _ => {}
}
```

---

## if in match Arms vs. Guards

An expression in a match arm body is evaluated only when the arm matches.
A guard (`if CONDITION`) is evaluated as part of pattern matching:

```rust
// Guard on the arm:
match x {
    n if n < 0 => println!("negative"),  // guard
    _ => println!("non-negative"),
}

// if inside the arm body — same result, but semantically different:
match x {
    n => {
        if n < 0 { println!("negative"); }
        else { println!("non-negative"); }
    }
}
```

Prefer guards when the condition determines which arm to run; prefer body `if`s
for logic that runs after the arm is selected.

---

## Diverging Expressions

Source: [reference/types/never.html](https://doc.rust-lang.org/reference/types/never.html)

The never type `!` is the return type of expressions that never complete normally:
`panic!()`, `break`, `continue`, `return`, `loop {}`, `std::process::exit()`.

`!` coerces to any type, which is why this compiles:

```rust
let x: i32 = if condition {
    42
} else {
    panic!("unreachable") // type !, coerces to i32
};
```

And why `let-else` works: the `else` block must have type `!`, which is satisfied
by any diverging expression.
