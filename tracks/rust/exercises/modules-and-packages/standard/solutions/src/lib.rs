// url-shortener/src/lib.rs
//
// The library crate root. This file does two things:
//   1. Declares the module tree
//   2. Re-exports the public API via `pub use`
//
// External users of this library interact with the crate through these
// re-exports. They write `use url_shortener::Store`, not
// `use url_shortener::storage::Store`. The internal module structure is
// an implementation detail.

// Module declarations — each corresponds to a file in src/
pub mod api;
pub mod hasher;
pub mod stats;
pub mod storage;

// Public API re-exports — the facade
//
// These control what's visible at the crate root. Without these, callers
// would have to use the full path: `url_shortener::storage::Store`.
// With them, `url_shortener::Store` works.
pub use api::{parse_args, Command};
pub use hasher::shorten;
pub use stats::Stats;
pub use storage::Store;

/// A convenience prelude for importing the full public API at once.
///
/// Usage in the binary or in tests:
///   use url_shortener::prelude::*;
pub mod prelude {
    pub use super::{parse_args, shorten, Command, Stats, Store};
}
