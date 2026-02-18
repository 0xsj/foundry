# Solution: URL Shortener Module Restructure

## Overview

The monolithic `starter/main.rs` is split into a library crate (`src/lib.rs`) with four focused modules, each owning a single concern. A thin binary (`src/main.rs`) handles only the CLI dispatch.

## Module Boundaries

| Module | File | Owns | Public API |
|--------|------|------|-----------|
| `storage` | `src/storage.rs` | `Store` struct, in-memory HashMap | `Store::new`, `insert`, `resolve`, `remove`, `len` |
| `hasher` | `src/hasher.rs` | URL hashing algorithm | `shorten(&str) -> String` |
| `api` | `src/api.rs` | CLI argument parsing | `Command` enum, `parse_args(&[String]) -> Result<Command, String>` |
| `stats` | `src/stats.rs` | Operation counters | `Stats::new`, `record_*`, `report` |

## Key Decisions

### Private fields everywhere

Every struct's fields are private. External code uses methods, not direct field access. This is the primary lesson of this exercise — encapsulation via module visibility.

```rust
// storage.rs
pub struct Store {
    entries: HashMap<String, String>,  // private
}
// Not accessible as: store.entries.get("key")
// Only accessible as: store.resolve("key")
```

### `pub use` creates a flat public API

`lib.rs` re-exports items from the four modules. The internal path `url_shortener::storage::Store` still works, but users prefer `url_shortener::Store`. This lets us reorganize the module tree without changing the public API contract.

```rust
// lib.rs
pub use storage::Store;
pub use hasher::shorten;
pub use api::{Command, parse_args};
pub use stats::Stats;
```

### Prelude module

`pub mod prelude` groups all common types for a one-line import:

```rust
use url_shortener::prelude::*;
```

This is the same pattern as `tokio::prelude`, `serde::prelude`, and `std::prelude`. Use sparingly — glob imports can make code hard to read. It's most useful in application code where you use most of the API anyway.

### `parse_args` takes `&[String]`, not `std::env::args()`

Taking the args as a parameter makes the function trivially testable without spawning a subprocess. The caller (main) decides where args come from.

### `resolve` returns `Option<&str>`, not `Option<String>`

Returns a borrowed reference into the store's internal HashMap. No heap allocation on the hot read path. The lifetime is tied to `&self` — the reference is valid as long as the store is alive and not mutated.

## Tradeoffs

| Design choice | Benefit | Cost |
|---------------|---------|------|
| Private fields | Encapsulation, freedom to refactor internals | More boilerplate (accessor methods) |
| `pub use` facade | Clean external API | Two paths to the same type can cause confusion |
| Prelude module | Convenient imports | Glob imports hide provenance |
| `parse_args(&[String])` | Testable | Caller must collect args first |

## Extending This Design

To add collision detection to `shorten`:
- Only `src/hasher.rs` needs to change
- `storage.rs` doesn't need to know about collisions
- `main.rs` doesn't change at all

To swap the in-memory store for a file-backed store:
- Only `src/storage.rs` changes
- The public interface (`insert`, `resolve`, `remove`) stays the same
- All callers are unaffected

This is the value of the module boundary — changes are contained.
