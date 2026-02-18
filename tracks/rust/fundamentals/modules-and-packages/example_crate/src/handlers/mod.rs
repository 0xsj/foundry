// example_crate/src/handlers/mod.rs
//
// A module as a directory. The `handlers/` directory needs mod.rs because
// it contains file-based submodules (health.rs). Without mod.rs, there is
// nowhere to declare those submodules.
//
// Loaded via `mod handlers;` in main.rs.
//
// Demonstrates:
// - Declaring submodules within a module file
// - Re-exporting submodule items at the parent level
// - Shared context types used across handlers

// RequestContext fields are pedagogical examples of pub(crate) — they aren't
// read by health.rs in this minimal example.
#![allow(dead_code)]

// Declare the submodule. Rust looks for src/handlers/health.rs.
pub mod health;

// Shared context available to all handlers.
// pub(crate) — handlers are an implementation detail; callers use the
// specific handler functions, not the context type directly.
pub(crate) struct RequestContext {
    pub(crate) request_id: u64,
    pub(crate) remote_addr: String,
}

impl RequestContext {
    pub(crate) fn new(request_id: u64, remote_addr: &str) -> RequestContext {
        RequestContext {
            request_id,
            remote_addr: remote_addr.to_string(),
        }
    }
}

// Re-export the most commonly used health type at the handlers level,
// so callers can write `handlers::LivenessResult` instead of
// `handlers::health::LivenessResult`.
pub use health::LivenessResult;
