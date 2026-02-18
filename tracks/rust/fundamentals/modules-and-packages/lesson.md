# Modules and Packages — Rust

## How Rust Organizes Code

Rust has a layered system with three concepts that are often confused:

- **Package** — a Cargo project. What `cargo new` creates. Contains a `Cargo.toml` and one or more crates.
- **Crate** — a compilation unit. Either a binary (`main.rs`) or a library (`lib.rs`). A package can have at most one library crate and any number of binary crates.
- **Module** — a namespace inside a crate. Defined with `mod`. Controls visibility. Modules are hierarchical; they nest.

The mental model: packages are to Cargo as modules are to the Rust language. Cargo knows about packages and crates. `rustc` knows about modules.

```
my-service/           ← package (one Cargo.toml)
├── Cargo.toml
└── src/
    ├── main.rs       ← binary crate root (the "crate root")
    ├── lib.rs        ← library crate root (optional)
    ├── config.rs     ← module, loaded via `mod config;` in lib.rs or main.rs
    └── handlers/
        ├── mod.rs    ← module, loaded via `mod handlers;`
        └── health.rs ← submodule, loaded via `mod health;` in handlers/mod.rs
```

### Your notes
<!-- -->


---

## Crates

### Binary vs Library

A **binary crate** compiles to an executable. It must have a `fn main()`. Its root is `src/main.rs` by convention (or any file declared under `[[bin]]` in `Cargo.toml`).

A **library crate** compiles to a `.rlib` (or `.so`/`.dylib` for C-compatible libs). No `main`. Its root is `src/lib.rs` by convention. Other code links against it.

```toml
# Cargo.toml

[package]
name = "url-shortener"
version = "0.1.0"
edition = "2021"

# Cargo infers these from the filesystem:
# [[bin]]   → src/main.rs
# [lib]     → src/lib.rs

# You can also be explicit:
[[bin]]
name = "url-shortener"
path = "src/main.rs"
```

### The Crate Root

The crate root is the source file `rustc` starts from. All `mod` declarations in the crate root define the module tree. Everything flows downward from this single root.

```rust
// src/lib.rs — the library crate root
mod config;       // loads src/config.rs
mod handlers;     // loads src/handlers/mod.rs (or src/handlers.rs in 2018+ edition)
mod storage;      // loads src/storage.rs
```

### Your notes
<!-- -->


---

## Modules

### The `mod` Keyword

`mod` does two things: it declares a module exists, and it (optionally) defines its contents inline or by pointing to a file.

```rust
// Inline module — content in the same file
mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    // Private by default — only accessible within this module
    fn internal_helper() -> i32 {
        42
    }
}

// File-based module — content in a separate file
mod config;  // tells rustc to look for src/config.rs or src/config/mod.rs
```

### File Layout (2018 Edition Style)

Rust 2018 (and later editions) allows a cleaner file layout. Given `mod handlers;` in the crate root:

| Edition | Where Rust looks |
|---------|-----------------|
| Old (pre-2018) | `src/handlers/mod.rs` |
| 2018+ | `src/handlers.rs` OR `src/handlers/mod.rs` |

**Prefer `src/handlers.rs`** for simple modules. Use `src/handlers/mod.rs` only when the module has submodules that need their own files — because submodule files must live inside the `handlers/` directory.

```
src/
├── lib.rs
├── config.rs          ← preferred: handlers.rs-style (simple module)
└── handlers/
    ├── mod.rs         ← required when handlers has file-based submodules
    ├── health.rs      ← submodule of handlers
    └── webhook.rs     ← submodule of handlers
```

Inside `src/handlers/mod.rs`:
```rust
pub mod health;    // loads src/handlers/health.rs
pub mod webhook;   // loads src/handlers/webhook.rs

// You can also put shared handler logic here
pub struct HandlerContext { /* ... */ }
```

### Module Paths

Every item in Rust has an absolute path starting from the crate root:

```rust
crate::handlers::health::check_liveness()
//  ^      ^       ^       ^
//  |      |       |       function
//  |      |       submodule (health.rs)
//  |      module (handlers/)
//  crate root keyword
```

You can also use relative paths with `self` and `super`:

```rust
mod parent {
    mod child {
        fn foo() {}

        fn bar() {
            self::foo();         // relative: same module
            super::sibling_fn(); // relative: parent module
        }
    }

    fn sibling_fn() {}
}
```

### Your notes
<!-- -->


---

## Visibility

### The Default: Private

In Rust, **everything is private by default**. Private means accessible only within the current module and its descendants.

```rust
mod auth {
    struct Token {          // private — only auth and its children can see this
        value: String,
    }

    pub struct Session {    // public — anyone can name this type
        id: u64,            // private field — only auth can access
        pub user_id: u64,   // public field
    }

    fn verify(token: &Token) -> bool {  // private function
        !token.value.is_empty()
    }

    pub fn authenticate(raw: &str) -> Option<Session> {  // public function
        let token = Token { value: raw.to_string() };    // can use Token here
        if verify(&token) {
            Some(Session { id: 1, user_id: 42 })
        } else {
            None
        }
    }
}

fn main() {
    // auth::Token { value: "x".into() };  // ERROR: Token is private
    // auth::verify(&token);               // ERROR: verify is private
    let session = auth::authenticate("secret");
    if let Some(s) = session {
        println!("user_id: {}", s.user_id);  // OK: user_id is pub
        // println!("{}", s.id);             // ERROR: id is private
    }
}
```

### Visibility Modifiers

| Modifier | Accessible from |
|----------|----------------|
| (none) | Current module and all its descendants |
| `pub` | Everywhere — full public |
| `pub(crate)` | Anywhere within the current crate only |
| `pub(super)` | Parent module only |
| `pub(in path)` | Specific ancestor module |

```rust
mod network {
    pub(crate) struct Connection {   // visible to whole crate, not external users
        pub(super) socket: u32,      // visible to parent (the module containing network)
        timeout_ms: u64,             // fully private
    }

    pub(crate) fn connect(host: &str) -> Connection {
        Connection { socket: 0, timeout_ms: 5000 }
    }
}
```

### `pub(crate)` in Practice

`pub(crate)` is the most underused visibility modifier. It's the right choice for:
- Shared types used across multiple modules in your crate but not exposed to users
- Internal error types
- Implementation details of a library's public API

If you find yourself writing `pub` on something that isn't part of your library's documented API, it should probably be `pub(crate)`.

### Your notes
<!-- -->


---

## The `use` Keyword

`use` brings names into scope so you don't have to write the full path every time.

```rust
// Without use:
fn process(conn: std::collections::HashMap<String, String>) {
    let _ = std::collections::HashMap::<String, i32>::new();
}

// With use:
use std::collections::HashMap;

fn process(conn: HashMap<String, String>) {
    let _ = HashMap::<String, i32>::new();
}
```

### Nested Paths

Group related imports:

```rust
use std::collections::{HashMap, HashSet, BTreeMap};
use std::io::{self, Read, Write};  // `self` brings io itself into scope

// Same as:
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeMap;
use std::io;
use std::io::Read;
use std::io::Write;
```

### Aliases with `as`

Rename imports to avoid conflicts or improve clarity:

```rust
use std::fmt::Result as FmtResult;
use std::io::Result as IoResult;

fn format_thing() -> FmtResult { Ok(()) }
fn read_thing() -> IoResult<Vec<u8>> { Ok(vec![]) }
```

### Glob Imports

`use module::*` imports everything public from a module:

```rust
use std::collections::*;  // imports HashMap, HashSet, BTreeMap, etc.
```

**Use with caution.** Glob imports make it hard to know where a name comes from. They are idiomatic in exactly two situations:
1. In tests: `use super::*;` to bring everything from the module under test into scope
2. In preludes (see below)

### Re-exports with `pub use`

`pub use` brings a name into scope *and* makes it part of the current module's public API. This is how you build clean APIs.

```rust
// src/lib.rs
mod storage;
mod hasher;
mod api;

// Re-export the types callers actually need:
pub use storage::Store;
pub use hasher::shorten;
pub use api::Command;

// Now callers write:
//   use url_shortener::Store;
// Instead of:
//   use url_shortener::storage::Store;
```

This is the **facade pattern** — you present a clean public API while keeping your internal module structure free to change.

### The Prelude Pattern

The Rust standard library has `std::prelude::*` automatically imported in every file. You can create your own prelude for your crate:

```rust
// src/prelude.rs
pub use crate::error::{Error, Result};
pub use crate::storage::Store;
pub use crate::config::Config;

// src/lib.rs
pub mod prelude;

// Consumer of your library:
use my_crate::prelude::*;  // Gets all the common types at once
```

### Your notes
<!-- -->


---

## Cargo.toml

`Cargo.toml` is the manifest for your package. It declares metadata, dependencies, features, and build configuration.

```toml
[package]
name = "url-shortener"
version = "0.1.0"
edition = "2021"           # Rust edition: 2015, 2018, 2021
authors = ["Alice <alice@example.com>"]
description = "A fast URL shortener service"
license = "MIT"
repository = "https://github.com/example/url-shortener"

[dependencies]
# From crates.io (semver)
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }

# From git
some-crate = { git = "https://github.com/org/crate", branch = "main" }

# From local path (useful in workspaces)
shared-types = { path = "../shared-types" }

# Optional dependency (activated by a feature flag)
deadpool = { version = "0.10", optional = true }

[dev-dependencies]
# Only available for tests and benchmarks
assert_matches = "1.5"
tempfile = "3"

[build-dependencies]
# Only for the build script (build.rs)
cc = "1.0"

[features]
# Feature flags
default = ["connection-pool"]    # features enabled by default
connection-pool = ["dep:deadpool"]  # enables optional deadpool dep
full = ["connection-pool", "metrics"]

[profile.release]
opt-level = 3
lto = true         # link-time optimization
strip = true       # strip debug symbols
```

### Dependency Resolution

Cargo uses a lock file (`Cargo.lock`) to pin exact versions. For applications, commit `Cargo.lock`. For libraries, don't — let downstream users determine the exact versions. Cargo resolves the dependency graph to find a set of versions that satisfies all constraints.

### Your notes
<!-- -->


---

## Workspaces

A **workspace** is a collection of related packages that share a single `Cargo.lock` and output directory. This is how large Rust projects structure multi-crate repositories.

```
my-platform/
├── Cargo.toml        ← workspace root
├── Cargo.lock        ← shared lock file
├── shared-types/     ← library crate
│   ├── Cargo.toml
│   └── src/lib.rs
├── api-server/       ← binary crate
│   ├── Cargo.toml
│   └── src/main.rs
└── worker/           ← binary crate
    ├── Cargo.toml
    └── src/main.rs
```

```toml
# my-platform/Cargo.toml (workspace root)
[workspace]
members = [
    "shared-types",
    "api-server",
    "worker",
]
resolver = "2"   # use the v2 dependency resolver (recommended)

# Workspace-level dependency versions (Cargo 1.64+)
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

```toml
# api-server/Cargo.toml
[package]
name = "api-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Reference the workspace dependency (inherits version)
serde = { workspace = true }
tokio = { workspace = true }
# Local workspace crate
shared-types = { path = "../shared-types" }
```

Benefits of workspaces:
- One `cargo build` at the root builds everything
- Shared `Cargo.lock` prevents version drift between crates
- Shared compilation cache (faster incremental builds)
- `cargo test --workspace` runs all tests at once

### Your notes
<!-- -->


---

## Cargo Features

Features are named boolean flags that enable optional functionality. They let consumers opt into capabilities without paying for unused code.

```toml
# Cargo.toml
[features]
default = []                   # nothing enabled by default
tls = ["dep:rustls"]           # enable TLS support
metrics = ["dep:prometheus"]   # enable metrics collection
full = ["tls", "metrics"]      # convenience: enable everything

[dependencies]
rustls = { version = "0.21", optional = true }
prometheus = { version = "0.13", optional = true }
```

In code, use `#[cfg(feature = "...")]` to conditionally compile:

```rust
#[cfg(feature = "tls")]
mod tls_support {
    pub fn wrap_with_tls(stream: TcpStream) -> TlsStream {
        // ...
    }
}

#[cfg(feature = "metrics")]
fn record_request(duration_ms: u64) {
    prometheus::REQUESTS.inc();
    prometheus::LATENCY.observe(duration_ms as f64);
}

#[cfg(not(feature = "metrics"))]
fn record_request(_duration_ms: u64) {
    // no-op
}
```

Activate features when using a dependency:

```bash
# Enable tls feature
cargo build --features tls

# Enable multiple features
cargo build --features "tls,metrics"

# Enable all features
cargo build --all-features
```

```toml
# Or in a dependent crate's Cargo.toml:
[dependencies]
my-crate = { version = "1.0", features = ["tls"] }
```

### Your notes
<!-- -->


---

## Integration Tests

Rust distinguishes between **unit tests** (inside the source file, in a `#[cfg(test)]` module) and **integration tests** (in a separate `tests/` directory).

```
src/
├── lib.rs
└── storage.rs
tests/               ← integration tests live here
├── store_test.rs    ← each file is a separate integration test crate
└── api_test.rs
```

Integration tests are their own crates. They can only access the **public API** of your library. This makes them a natural API correctness check.

```rust
// tests/store_test.rs

// Must use the crate by its published name:
use url_shortener::Store;
use url_shortener::shorten;

#[test]
fn test_round_trip() {
    let store = Store::new();
    let key = shorten("https://example.com/long/path");
    assert!(store.resolve(&key).is_some());
}
```

Run them with `cargo test`. Integration tests run automatically alongside unit tests.

**When to use integration tests vs unit tests:**
- Unit tests: test individual functions, internal logic, edge cases. Can test private code.
- Integration tests: test the public API from a user's perspective. Verify that modules compose correctly.

### Your notes
<!-- -->


---

## Comparison to Go

This is where Go and Rust diverge significantly in philosophy.

| Concept | Go | Rust |
|---------|----|----|
| Unit of compilation | Package | Crate |
| Namespace mechanism | Package (flat within dir) | Module (hierarchical tree) |
| Visibility | Uppercase = public, lowercase = private | `pub` modifier, private by default |
| Import | `import "github.com/org/pkg"` | `use crate::module::Type` |
| Dependency manager | `go.mod` | `Cargo.toml` |
| Registry | `pkg.go.dev` (no central registry) | `crates.io` |
| Module structure | All `.go` files in a dir = same package | Files must be explicitly declared with `mod` |
| Sub-packages | Separate directories, separate `import` | Module tree, declared hierarchically |
| Re-exports | No mechanism (package is the unit) | `pub use` for clean facades |

**Key difference — flat vs hierarchical:**

In Go, a directory defines a package. All `.go` files in a directory automatically belong to the same package and share a namespace. There is no hierarchical module tree.

In Rust, a crate has a module tree. Files are not automatically included — each file must be declared via `mod`. You control the hierarchy explicitly. This means you can have `crate::http::server::Handler` as a deeply nested path.

**Key difference — visibility:**

Go's rule is elegant but blunt: identifier starts with uppercase = exported, lowercase = unexported. No gradations.

Rust's rule is more expressive: `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`. The default (no modifier) means private. This lets library authors expose different surfaces to internal code vs external consumers.

### Your notes
<!-- -->


---

## Comparison to TypeScript / JavaScript

| Concept | JavaScript/TypeScript | Rust |
|---------|-----------------------|------|
| Module system | ES Modules (import/export) | mod + use |
| Export | `export const foo = ...` | `pub fn foo()` |
| Import | `import { foo } from './bar'` | `use crate::bar::foo;` |
| Default export | `export default class Foo` | No equivalent |
| Re-export | `export { foo } from './bar'` | `pub use crate::bar::foo;` |
| Package manager | npm/yarn/pnpm | Cargo |
| Package manifest | `package.json` | `Cargo.toml` |
| Lock file | `package-lock.json` / `yarn.lock` | `Cargo.lock` |
| Registry | npmjs.com | crates.io |
| Optional deps | (no built-in mechanism) | Cargo features |
| Monorepo | npm workspaces | Cargo workspaces |

**Key difference — explicit declaration:**

In ESM, any `.ts` file can `import` from any other file by path. No registration required.

In Rust, a file only becomes part of your crate when a parent module declares it with `mod`. This explicit declaration is what allows the module tree structure. If you create `src/foo.rs` without a `mod foo;` somewhere in the tree, it's invisible to the compiler.

**Key difference — `pub use` vs re-exports:**

TypeScript: `export { foo, bar } from './internal'` is common but there's no notion of "internal" vs "external" consumers.

Rust: `pub use` combined with `pub(crate)` lets you have deeply structured internal code while presenting a flat, clean public API. Libraries commonly do this — the internal structure is the developer's concern, the `pub use` surface is the user's concern.

### Your notes
<!-- -->
