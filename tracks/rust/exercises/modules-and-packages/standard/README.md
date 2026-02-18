# Standard Exercise: URL Shortener — Module Restructure

## Scenario

You inherited a URL shortener CLI tool written by a backend engineer who was "just trying to get it working." All the logic lives in a single `main.rs`: the in-memory store, the hashing algorithm, the command-line parsing, and the analytics tracking are tangled together. The tool works, but adding features requires reading through 300 lines of intermixed concerns. Your task is to break it into a well-organized multi-module crate with a clean public API.

## Brief

Restructure the monolithic `starter/main.rs` into a library crate (`src/lib.rs`) with four modules — `storage`, `hasher`, `api`, and `stats` — plus a thin binary entry point (`src/main.rs`). Create a prelude-style re-export so consumers can import everything they need from one place.

## Acceptance Criteria

1. **Module boundaries**
   - `storage` — owns the `Store` struct (in-memory HashMap), `insert`, `resolve`, and `remove` operations
   - `hasher` — owns the `shorten` function that produces a 6-character key from a URL
   - `api` — owns the `Command` enum and `parse_args` function for CLI parsing
   - `stats` — owns the `Stats` struct and `record_hit` / `report` operations

2. **Library crate (`src/lib.rs`)**
   - Declares all four modules
   - Re-exports the public API so callers write `use url_shortener::Store` not `use url_shortener::storage::Store`
   - Public re-exports: `Store`, `shorten`, `Command`, `parse_args`, `Stats`

3. **Binary entry point (`src/main.rs`)**
   - Contains only `fn main()` — no business logic
   - Uses `use url_shortener::prelude::*;` (or explicit imports from lib) to access types
   - Dispatches to the appropriate modules based on the parsed command

4. **Visibility discipline**
   - `Store`'s internal `HashMap` field is private — mutations go through methods
   - `Stats`'s counter fields are private — reads go through `report()`
   - Helper functions within modules are private (not `pub`)
   - Use `pub(crate)` for anything shared between modules but not part of the external API

5. **Cargo.toml**
   - Package named `url-shortener`
   - Edition 2021
   - No external dependencies (stdlib only)

6. **All tests pass** — the test suite in the reference solution verifies the public API

## Constraints

- The `Store` must not expose its `HashMap` directly — encapsulate it
- `shorten` must be deterministic (same URL always produces the same key)
- `parse_args` does not read from `std::env::args` directly — it takes `&[String]` (makes it testable)
- No `unwrap()` in `parse_args` — return `Result<Command, String>`

## Hints

<details>
<summary>Hint 1: Start with the file structure</summary>

Create the files first, then fill them in:

```
src/
├── lib.rs          ← mod declarations + pub use re-exports
├── main.rs         ← fn main() only
├── storage.rs      ← Store struct + impl
├── hasher.rs       ← shorten() function
├── api.rs          ← Command enum + parse_args()
└── stats.rs        ← Stats struct + impl
```

In `lib.rs`, start with:
```rust
pub mod storage;
pub mod hasher;
pub mod api;
pub mod stats;

pub use storage::Store;
pub use hasher::shorten;
pub use api::{Command, parse_args};
pub use stats::Stats;
```

</details>

<details>
<summary>Hint 2: The hasher function</summary>

The starter's `shorten` uses a simple djb2-style hash. Move it exactly — the hash function itself doesn't need to change, only its location. It takes `&str` and returns `String`.

```rust
// src/hasher.rs
pub fn shorten(url: &str) -> String {
    // implementation from starter/main.rs
}
```

</details>

<details>
<summary>Hint 3: Private fields and pub methods</summary>

In the monolith, `Store` accesses its `HashMap` directly. In the split version:

```rust
// src/storage.rs
use std::collections::HashMap;

pub struct Store {
    entries: HashMap<String, String>,  // private!
}

impl Store {
    pub fn new() -> Store { ... }
    pub fn insert(&mut self, key: String, url: String) { ... }
    pub fn resolve(&self, key: &str) -> Option<&str> { ... }
    pub fn remove(&mut self, key: &str) -> bool { ... }
}
```

</details>

<details>
<summary>Hint 4: parse_args return type</summary>

```rust
// src/api.rs
pub enum Command {
    Shorten { url: String },
    Resolve { key: String },
    Remove { key: String },
    Stats,
    Help,
}

pub fn parse_args(args: &[String]) -> Result<Command, String> {
    // args[0] is the subcommand
    match args.first().map(|s| s.as_str()) {
        Some("shorten") => { /* ... */ }
        Some("resolve") => { /* ... */ }
        // ...
        _ => Err(format!("unknown command: {:?}", args.first())),
    }
}
```

</details>
