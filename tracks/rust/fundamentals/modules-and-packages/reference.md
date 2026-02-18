# Rust Reference — Modules and Packages

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Cargo Book](https://doc.rust-lang.org/cargo/), and
> [The Rust Edition Guide](https://doc.rust-lang.org/edition-guide/) for the
> `modules-and-packages` module. Covers: crates, modules, visibility, use
> declarations, Cargo.toml, and workspaces.

---

## Crates

Source: [reference/crates-and-source-files.html](https://doc.rust-lang.org/reference/crates-and-source-files.html)

A **crate** is the unit of compilation in Rust. The compiler processes one crate at a time, producing either an executable or a library.

- A **crate root** is the source file that `rustc` starts from when compiling.
- All modules in a crate form a tree rooted at the crate root.
- The crate root implicitly defines the **`crate` keyword** — a path prefix referring to the root of the current crate.

### Crate Types

| Type | Root file (convention) | Output |
|------|----------------------|--------|
| Binary | `src/main.rs` | Executable |
| Library | `src/lib.rs` | `.rlib` / `.so` / `.dylib` |

A single package (Cargo project) may contain:
- At most **one library crate**
- Any number of **binary crates** (via `src/main.rs` or `[[bin]]` in `Cargo.toml`)
- Any number of **example crates** (`examples/`)
- Any number of **integration test crates** (`tests/`)
- Any number of **benchmark crates** (`benches/`)

### Crate Name

The crate's name is set in `Cargo.toml` (`[package] name`). Within the crate, items can be referenced with the `crate::` prefix. External crates are referenced by their name (with `-` replaced by `_` in identifiers).

---

## Modules

Source: [reference/items/modules.html](https://doc.rust-lang.org/reference/items/modules.html)

A **module** is a container for zero or more items. Modules form a tree. The module at the root of the tree is called the **crate root module**.

### Module Declarations

```rust
// Inline module — body defined here
mod math {
    pub fn double(x: i32) -> i32 { x * 2 }
}

// File-based module — body loaded from a file
mod config;   // looks for src/config.rs or src/config/mod.rs
```

### File Search for File-Based Modules

For `mod name;` in a file at path `P`:

| Rust edition | Candidate files |
|--------------|----------------|
| 2015 | `<P_dir>/name.rs`, `<P_dir>/name/mod.rs` |
| 2018+ | `<P_dir>/name.rs`, `<P_dir>/name/mod.rs` |

(The 2018 edition did not change the lookup rules themselves, but established `name.rs` as the preferred convention over `name/mod.rs` for leaf modules.)

The compiler tries both paths. If both exist, it is an error. If neither exists, it is an error.

### Module Attributes

Attributes can be applied to modules:

```rust
#[cfg(feature = "analytics")]
mod analytics;

#[allow(dead_code)]
mod legacy;

/// Documentation comment on a module.
pub mod api;
```

---

## Visibility and Privacy

Source: [reference/visibility-and-privacy.html](https://doc.rust-lang.org/reference/visibility-and-privacy.html)

Rust's privacy rules control which items are accessible from which modules.

### Default Visibility

Items without a `pub` qualifier are **private** to the current module. Private items are accessible to:
- The module itself
- Any descendants (child modules) of that module

A child module can access private items from its parent. A parent **cannot** access private items from its children.

```rust
mod outer {
    fn private_fn() {}  // private to `outer` (and its children)

    mod inner {
        fn call_parent() {
            super::private_fn();  // OK — child accessing parent's private fn
        }
    }
}
```

### Visibility Modifiers

| Syntax | Description |
|--------|-------------|
| (none) | Private — accessible only within the current module and its descendants |
| `pub` | Public — accessible from anywhere (subject to the parent module being accessible) |
| `pub(crate)` | Crate-visible — accessible anywhere in the current crate |
| `pub(super)` | Parent-visible — accessible in the parent module |
| `pub(in path)` | Path-visible — accessible in the specified ancestor module |
| `pub(self)` | Same as no qualifier (private) — rarely used, mainly for clarity |

```rust
pub struct ApiResponse {         // Public
    pub status: u16,             // Public field
    pub(crate) request_id: u64,  // Crate-internal field
    body: Vec<u8>,               // Private field
}

pub(crate) fn internal_helper() {}   // Crate-internal function
pub(super) fn parent_helper() {}     // Parent-accessible function
```

### Visibility Through Re-exports

An item can be **re-exported** from a module, making it accessible at a shorter path:

```rust
// In src/lib.rs:
mod internal {
    pub struct Config { /* ... */ }
}

// Re-export — now accessible as `crate::Config`
pub use internal::Config;
```

The re-exported item's visibility is the maximum of: the `pub use` visibility and the item's own visibility. An item cannot be re-exported to be *more* public than the item itself.

---

## Use Declarations

Source: [reference/items/use-declarations.html](https://doc.rust-lang.org/reference/items/use-declarations.html)

A `use` declaration brings names into scope, creating local aliases for paths.

### Syntax

```rust
use std::collections::HashMap;           // simple path
use std::collections::{HashMap, HashSet}; // multiple from same prefix
use std::io::{self, Read};               // `self` imports the prefix itself
use std::collections::*;                 // glob import (all public items)
use std::fmt::Result as FmtResult;       // rename with `as`
```

### Binding Rules

- `use` creates a binding in the current scope. The binding is private by default.
- `pub use` creates a public binding — it re-exports the item.
- `use` paths are absolute from the crate root, external crate, or `self`/`super`.

### Path Prefixes

| Prefix | Meaning |
|--------|---------|
| `crate::` | Root of the current crate |
| `self::` | Current module |
| `super::` | Parent module |
| `::name` | From an external crate named `name` (2015 edition style) |
| `name::` | From external crate `name` (2018+ edition, no leading `::`) |

### Glob Imports

`use foo::*` imports all **public** items from `foo`. Glob imports:
- Do not import items that are re-exported from other modules if those items conflict
- Can be overridden by explicit bindings at any point
- Generate a warning from Clippy in most non-prelude contexts

Idiomatic use of glob imports:
1. `use super::*;` in `#[cfg(test)]` modules
2. `use crate::prelude::*;` for your own crate's prelude module
3. Avoid in application code — makes provenance unclear

---

## Paths

Source: [reference/paths.html](https://doc.rust-lang.org/reference/paths.html)

A **path** is a sequence of one or more path segments separated by `::`.

### Simple Paths (for use declarations and visibility)

```rust
// Examples of simple paths:
std::collections::HashMap
crate::config::Config
super::helpers::parse_duration
```

### Qualified Paths (for disambiguation)

When a type implements multiple traits with the same method name:

```rust
<Type as Trait>::method(args)
// or for associated types:
<Type as Iterator>::Item
```

### Path Segments

| Segment | Meaning |
|---------|---------|
| `self` | The current module or instance |
| `super` | The parent module |
| `crate` | The crate root |
| `$crate` | The crate root of the crate where the macro is defined |
| An identifier | A module, type, function, or other named item |

---

## Cargo.toml Reference

Source: [cargo/reference/manifest.html](https://doc.rust-lang.org/cargo/reference/manifest.html)

### [package] Table

```toml
[package]
name = "my-crate"          # Required. The crate's name (used as crate root identifier)
version = "0.1.0"          # Required. Semver string
edition = "2021"           # Rust edition. Default: "2015". Options: "2015", "2018", "2021"
authors = ["Name <email>"]
description = "Brief description"
license = "MIT OR Apache-2.0"
repository = "https://github.com/..."
documentation = "https://docs.rs/..."
readme = "README.md"
keywords = ["network", "async"]  # max 5
categories = ["network-programming"]  # from crates.io category list
exclude = ["tests/", "*.md"]
include = ["src/**/*", "Cargo.toml"]
publish = true             # false = do not publish to crates.io
```

### Dependency Tables

```toml
[dependencies]          # Available to src/ (library and binary crates)
[dev-dependencies]      # Only for tests, examples, and benchmarks
[build-dependencies]    # Only for build.rs
[target.'cfg(...)'.dependencies]  # Platform-specific dependencies
```

### Dependency Specification

```toml
[dependencies]
# Version requirement (semver)
serde = "1.0"              # ^1.0 — compatible with 1.0 (no breaking changes)
serde = "^1.0"             # explicit caret — same as above
serde = "~1.0"             # tilde — compatible with 1.0.x only
serde = "=1.0.0"           # exact version

# With features
serde = { version = "1.0", features = ["derive"] }

# Optional (activate via features)
deadpool = { version = "0.10", optional = true }

# From a git repository
my-lib = { git = "https://github.com/...", branch = "main" }
my-lib = { git = "https://github.com/...", tag = "v1.0.0" }
my-lib = { git = "https://github.com/...", rev = "a4b2c3d" }

# From a local path
shared = { path = "../shared" }

# Rename the dependency
httpx = { package = "http", version = "0.2" }  # import as `httpx` in code
```

### [features] Table

```toml
[features]
default = ["feature-a"]          # Features enabled by default
feature-a = []                   # Feature with no additional dependencies
feature-b = ["feature-a", "dep:some-optional-crate"]  # Feature enables another
full = ["feature-a", "feature-b"]
```

Feature names must be valid identifiers. They cannot conflict with dependency names.

`dep:` prefix references an optional dependency without creating a feature with the same name.

### [[bin]], [lib], [[example]], [[test]], [[bench]] Tables

```toml
[lib]
name = "my_lib"           # crate name (default: package name with _ instead of -)
path = "src/lib.rs"       # default
crate-type = ["rlib"]     # rlib (default), cdylib, staticlib, dylib, proc-macro

[[bin]]
name = "server"
path = "src/bin/server.rs"
required-features = ["networking"]  # only built when feature is enabled

[[example]]
name = "simple-usage"
path = "examples/simple.rs"

[[test]]
name = "integration"
path = "tests/integration.rs"
```

---

## Workspaces

Source: [cargo/reference/workspaces.html](https://doc.rust-lang.org/cargo/reference/workspaces.html)

A **workspace** is a set of packages that share a common `Cargo.lock` and output directory (`target/`).

### Workspace Manifest

```toml
# Cargo.toml (workspace root — no [package] table)
[workspace]
members = [
    "shared-types",
    "api-server",
    "worker",
    "tools/*",           # glob — matches all direct children of tools/
]
exclude = ["tools/legacy"]   # exclude from glob
resolver = "2"               # v2 dependency resolver (recommended)

# Workspace-wide dependency versions (Cargo 1.64+)
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1" }
anyhow = "1.0"

# Workspace-wide package metadata
[workspace.package]
edition = "2021"
authors = ["Team <team@example.com>"]
license = "MIT"
```

Member `Cargo.toml` can inherit workspace values:

```toml
[package]
name = "api-server"
version = "0.1.0"
edition.workspace = true          # inherit from workspace
authors.workspace = true

[dependencies]
serde = { workspace = true }      # use workspace version + features
serde = { workspace = true, features = ["rc"] }  # extend workspace features
shared-types = { path = "../shared-types" }
```

### Workspace Properties

- All packages share a single `Cargo.lock` — guarantees consistent dependency versions across crates.
- All packages share a single `target/` output directory — reuses compiled artifacts.
- `cargo build --workspace` builds all members.
- `cargo test --workspace` tests all members.
- `cargo build -p api-server` builds only the specified package.

---

## Integration Tests

Source: [cargo/reference/cargo-targets.html](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests)

Files in `tests/` are **integration test crates**. Each file is compiled as a separate crate that links against the library crate.

Properties:
- Each `tests/*.rs` file is an independent crate — no shared state between test files by default.
- Can only access `pub` items from the library (tests the public API).
- Can use `dev-dependencies`.
- Run with `cargo test`.

```rust
// tests/integration.rs
use my_crate::PublicType;

#[test]
fn test_via_public_api() {
    let t = PublicType::new("arg");
    assert!(t.is_valid());
}
```

For shared helpers across multiple integration test files:

```
tests/
├── common/
│   └── mod.rs    ← shared helpers (not a test crate, just a module)
├── api_test.rs   ← imports common via `mod common;`
└── store_test.rs
```

```rust
// tests/api_test.rs
mod common;  // loads tests/common/mod.rs

#[test]
fn test_with_common_setup() {
    let ctx = common::setup();
    // ...
}
```

---

## Conditional Compilation

Source: [reference/conditional-compilation.html](https://doc.rust-lang.org/reference/conditional-compilation.html)

`#[cfg(...)]` conditionally includes items based on compile-time flags.

### Common Predicates

```rust
#[cfg(feature = "tls")]          // Cargo feature
#[cfg(test)]                     // Compiling for tests
#[cfg(debug_assertions)]         // Debug build
#[cfg(target_os = "linux")]      // Operating system
#[cfg(target_arch = "x86_64")]   // CPU architecture
#[cfg(target_pointer_width = "64")]
```

### Combinators

```rust
#[cfg(all(feature = "tls", target_os = "linux"))]
#[cfg(any(feature = "tls", feature = "native-tls"))]
#[cfg(not(target_os = "windows"))]
```

### `cfg_attr` — Conditional Attributes

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    pub timeout: u64,
}
```

### `cfg!` Macro — Runtime Check

Returns a `bool` at compile time, evaluated at the use site:

```rust
if cfg!(debug_assertions) {
    println!("debug build: enabling verbose logging");
}
```

Unlike `#[cfg]`, `cfg!()` does not remove code from compilation — it evaluates to `true` or `false` and the dead branch is eliminated by the optimizer.
