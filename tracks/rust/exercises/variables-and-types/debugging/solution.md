# Solution: Config Parser Bugs

## Bug 1: Clone/move confusion in `apply_defaults`

### Root Cause

The function clones `config`, mutates the clone (`defaults`), but then returns
the *original* `config` instead of the modified `defaults`.

```rust
fn apply_defaults(config: Config) -> Config {
    let mut defaults = config.clone();
    if defaults.port == 0 {
        defaults.port = 8080;
    }
    if defaults.host.is_empty() {
        defaults.host = String::from("localhost");
    }
    config  // BUG: returns the unmodified original
}
```

### Why It Compiles

Both `config` and `defaults` are valid `Config` values. Returning either one
type-checks. Rust's ownership system prevents use-after-move, but it cannot
detect *which* value you intended to return -- that is a logic error.

### Fix

Option A -- return the modified clone:

```rust
fn apply_defaults(config: Config) -> Config {
    let mut defaults = config.clone();
    if defaults.port == 0 { defaults.port = 8080; }
    if defaults.host.is_empty() { defaults.host = String::from("localhost"); }
    defaults  // return the modified clone
}
```

Option B -- skip the clone, mutate the input directly (better):

```rust
fn apply_defaults(mut config: Config) -> Config {
    if config.port == 0 { config.port = 8080; }
    if config.host.is_empty() { config.host = String::from("localhost"); }
    config
}
```

Option B is idiomatic Rust. Since `apply_defaults` takes ownership of `config`,
we can add `mut` to the parameter and modify it in place. No clone needed.

### Lesson

- `clone()` creates a separate value. Mutating the clone does not affect the
  original.
- When a function takes ownership (`config: Config`, not `&config`), you can
  make the parameter mutable (`mut config`) and modify it directly.
- Unnecessary clones are a code smell in Rust. If you own the value, just
  modify it.

---

## Bug 2: Integer overflow via truncating `as` cast in `parse_port`

### Root Cause

The port string is parsed as `u32`, then cast to `u16` with `as`. The `as`
keyword performs a *truncating* cast -- it silently discards the high bits.

```rust
let num: u32 = input.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
Ok(num as u16)  // 70000u32 as u16 == 4464 (70000 % 65536)
```

`70000` in binary is `0b1_0001_0001_0111_0000`. Truncating to 16 bits gives
`0b0001_0001_0111_0000` = 4464.

### Why It Compiles

`as` casts between numeric types are always valid in Rust -- they never panic.
This is by design (it's a low-level operation), but it means the programmer is
responsible for range checking.

### Fix

Option A -- parse directly as u16:

```rust
fn parse_port(input: &str) -> Result<u16, String> {
    input.parse::<u16>().map_err(|e| e.to_string())
}
```

This is the simplest fix. `"70000".parse::<u16>()` returns `Err` because
70000 is out of range for `u16`.

Option B -- parse as u32 and validate the range:

```rust
fn parse_port(input: &str) -> Result<u16, String> {
    let num: u32 = input.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
    u16::try_from(num).map_err(|_| format!("port {} out of range (0-65535)", num))
}
```

### Lesson

- `as` casts between integer types are **truncating**, not checked. They will
  never produce an error -- they silently wrap.
- Prefer `try_from()` / `try_into()` for checked conversions. They return
  `Result` and make overflow explicit.
- Alternatively, parse directly into the target type when possible.
- This is a common source of subtle bugs. In Go, the equivalent `uint16(70000)`
  also silently truncates. In TypeScript, all numbers are f64 so overflow
  manifests differently (loss of precision beyond 2^53).

---

## Bug 3: Shadowing hides the type in `parse_debug`

### Root Cause

The variable `debug` is shadowed from `&str` to `usize` (its length), and
then compared to 0. Any non-empty string has length > 0, so `parse_debug`
returns `true` for *every* non-empty input including `"false"`.

```rust
fn parse_debug(input: &str) -> bool {
    let debug = input.trim();       // debug: &str
    let debug = debug.len();        // debug: usize (shadows the &str!)
    debug != 0                      // true for ANY non-empty string
}
```

Trace for `"false"`:
1. `debug = "false"` (after trim)
2. `debug = 5` (length of "false")
3. `5 != 0` => `true`

### Why It Compiles

Rust allows shadowing -- you can reuse a variable name with a different type.
This is intentional and useful (e.g., `let x = x.parse::<i32>()?` to convert
and keep the name). But here, the author accidentally replaced the string
value with its length.

### Fix

```rust
fn parse_debug(input: &str) -> bool {
    match input.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => true,
        "false" | "0" | "no" => false,
        _ => false,  // default to disabled for unknown input
    }
}
```

### Lesson

- Shadowing changes the type of a binding. The old value becomes inaccessible.
  This is powerful but can cause subtle bugs if done accidentally.
- When you see `let x = x.something()`, check whether the return type matches
  your intent.
- For boolean parsing, always use explicit pattern matching. Never rely on
  truthiness/length checks -- that is a JavaScript/Python habit that does not
  translate to Rust.
- Clippy (`cargo clippy`) can catch some shadowing issues with the
  `clippy::shadow_reuse` and `clippy::shadow_unrelated` lints.

---

## Summary

| Bug | Category | Rust Concept | Prevention |
|-----|----------|-------------|------------|
| Returns original instead of clone | Logic error | Move semantics, `mut` params | Prefer `mut` params over clone; review return values |
| Truncating `as` cast | Silent data loss | `as` vs `try_from` | Always use `try_from`/`try_into` for fallible conversions |
| Shadowing hides type change | Accidental shadowing | Variable shadowing | Enable `clippy::shadow_unrelated`; review `let x = x.` chains |

## Related Pitfalls

- [[rust-as-cast-truncation]] -- `as` casts between integer types
- [[rust-shadowing-pitfalls]] -- When shadowing helps vs hurts
- [[rust-clone-vs-move]] -- When to clone vs take ownership
