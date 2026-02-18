// example_crate/src/handlers/health.rs
//
// A submodule of handlers. This file is loaded via `pub mod health;` in
// handlers/mod.rs. It lives at src/handlers/health.rs because that is the
// conventional location for a submodule of `handlers`.
//
// Demonstrates:
// - Accessing the parent module via `super::`
// - A public type returned through the handlers public API
// - A private helper function

/// The result of a liveness check.
/// `pub` — this type is exported through handlers/mod.rs via `pub use`.
#[derive(Debug, PartialEq)]
pub enum LivenessResult {
    Alive,
    Degraded(String),
}

/// Check whether the service is alive.
/// In production, this checks database connections, external services, etc.
pub fn check_liveness() -> LivenessResult {
    // Access the parent module (handlers) via `super::`.
    // This is how you reference sibling modules or parent-level types.
    let _ctx = super::RequestContext::new(0, "127.0.0.1");

    // Simulate a healthy check.
    if dependencies_healthy() {
        LivenessResult::Alive
    } else {
        LivenessResult::Degraded("dependency check failed".to_string())
    }
}

/// Private helper. Not visible outside this module.
fn dependencies_healthy() -> bool {
    // In a real service: check DB connection, Redis, etc.
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liveness_returns_alive() {
        assert_eq!(check_liveness(), LivenessResult::Alive);
    }
}
