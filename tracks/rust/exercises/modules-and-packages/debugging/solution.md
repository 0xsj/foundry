# Solution: Broken Module Wiring

## Summary

Six bugs across three files. All related to visibility, module declaration, or use paths. None of the logic is wrong — only the wiring.

| Bug | File | Category | Fix |
|-----|------|----------|-----|
| 1 | main.rs | Wrong use path | Fix import alias |
| 2 | main.rs | Missing `mod` declaration | Add `mod util;` |
| 3 | main.rs | Missing `mod` declaration (same) | Follows from fixing bug 2 |
| 4 | config.rs | Private struct used externally | Add `pub` to `RawConfig` |
| 5 | config.rs | Private function used externally | Add `pub` to `load_defaults` |
| 6 | util.rs | Private function used externally | Add `pub` to `parse_duration` |

---

## Bug 1: Wrong use path in main.rs

### The Code

```rust
// main.rs
use crate::config::AppConfig;
```

### What's Wrong

`AppConfig` is not at `crate::config::AppConfig` — wait, actually it is. `AppConfig` is defined in `config.rs` which is the `config` module. This import is correct in isolation. However, `AppConfig` is only used indirectly through `load_defaults`'s return type. The error "unresolved import" for this line is actually a cascade from Bug 4 (RawConfig being private). Once that's fixed, this import resolves.

Actually: the import is correct. The primary symptom here is the cascade. This is why fixing root-cause bugs first clears secondary errors.

---

## Bug 2: Missing `mod util;` declaration in main.rs

### The Code

```rust
// main.rs
use util::parse_duration;  // refers to a module that was never declared
mod config;
// `mod util;` is missing
```

### Why It Fails

Rust does not automatically discover `.rs` files. Every file that should be part of the crate must be declared with `mod name;` in a parent module. `src/util.rs` exists on disk, but without `mod util;` in `main.rs`, the compiler never processes it.

### The Error

```
error[E0433]: failed to resolve: use of undeclared crate or module `util`
  --> src/main.rs:9:5
   |
9  | use util::parse_duration;
   |     ^^^^ use of undeclared crate or module `util`
```

### Fix

```rust
// main.rs
mod config;
mod util;  // ← add this

use crate::config::AppConfig;
use util::parse_duration;
```

### Lesson

In Go, all `.go` files in a directory automatically belong to the same package. In Rust, you must explicitly declare every file with `mod`. This is a deliberate design — it gives you control over the module tree. The tradeoff is that forgetting `mod` is a common mistake when creating new files.

---

## Bug 4: `RawConfig` is private

### The Code

```rust
// config.rs
struct RawConfig {    // no pub
    pub host: String,
    pub port: u16,
    pub timeout_str: String,
}

pub fn load_raw(_path: &str) -> RawConfig { ... }
```

### Why It Fails

`load_raw` is `pub` and returns `RawConfig`, but `RawConfig` itself is private. You cannot have a public function whose return type is a private type — callers couldn't name the type to store the return value.

### The Error

```
error[E0446]: private type `RawConfig` in public interface
  --> src/config.rs:18:1
   |
3  | struct RawConfig {
   |        --------- `RawConfig` is a private type
18 | pub fn load_raw(_path: &str) -> RawConfig {
   |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ can't leak private type
```

### Fix

```rust
// config.rs
pub struct RawConfig {    // ← add pub
    pub host: String,
    pub port: u16,
    pub timeout_str: String,
}
```

### Lesson

Rust prevents you from leaking private types through public interfaces. If a function is `pub`, all types it uses in its signature must also be at least as visible. This is a correctness guarantee — callers couldn't use the function if they can't name the types involved.

---

## Bug 5: `load_defaults` is private

### The Code

```rust
// config.rs
fn load_defaults(raw: RawConfig) -> AppConfig { ... }  // private
```

```rust
// main.rs
let cfg = config::load_defaults(raw);  // called from outside config module
```

### Why It Fails

`load_defaults` is in the `config` module. `main.rs` is the crate root (a sibling scope, not a child of `config`). Private items in `config` are not accessible from `main.rs`.

### The Error

```
error[E0603]: function `load_defaults` is private
  --> src/main.rs:17:24
   |
17 |     let cfg = config::load_defaults(raw);
   |                       ^^^^^^^^^^^^^ private function
```

### Fix

```rust
// config.rs
pub fn load_defaults(raw: RawConfig) -> AppConfig { ... }  // ← add pub
```

### When to Use `pub` vs `pub(crate)` vs `pub(super)` Here

`pub(super)` would also work, since `main.rs` is the parent of `config`. `pub(crate)` would work too. `pub` is fine here since this is a binary crate (no library API to protect). In a library crate, you'd think more carefully: is `load_defaults` part of your external API? If not, `pub(crate)` is the right choice.

---

## Bug 6: `parse_duration` is private

### The Code

```rust
// util.rs
fn parse_duration(s: &str) -> Option<u64> { ... }  // private
```

```rust
// main.rs
use util::parse_duration;  // tries to import a private function
```

### Fix

```rust
// util.rs
pub fn parse_duration(s: &str) -> Option<u64> { ... }  // ← add pub
```

---

## Fixed Files

### main.rs (fixed)

```rust
use crate::config::AppConfig;
use util::parse_duration;

mod config;
mod util;  // ← added

fn main() {
    let raw = config::load_raw("config.toml");
    let cfg = config::load_defaults(raw);
    let timeout = parse_duration(&cfg.timeout_str);
    println!("Config loaded: host={} timeout={:?}ms", cfg.host, timeout);
}
```

### config.rs (fixed)

```rust
pub struct RawConfig { ... }  // ← added pub

pub fn load_defaults(raw: RawConfig) -> AppConfig { ... }  // ← added pub
```

### util.rs (fixed)

```rust
pub fn parse_duration(s: &str) -> Option<u64> { ... }  // ← added pub
```

---

## Patterns to Watch For

1. **"module not found" / "undeclared module"** — you created a `.rs` file but forgot `mod name;`
2. **"private type in public interface"** — a `pub fn` returns or takes a private type
3. **"is private"** — you're calling a function or accessing a field from outside its module
4. **Cascade errors** — one missing `mod` declaration can cause 5+ downstream errors. Fix root causes first.

## Related Pitfalls

- [[rust-missing-mod-declaration]] — forgetting to wire up new files
- [[rust-private-type-in-public-interface]] — visibility mismatch between function and its types
- [[rust-pub-vs-pub-crate]] — choosing the right visibility level
