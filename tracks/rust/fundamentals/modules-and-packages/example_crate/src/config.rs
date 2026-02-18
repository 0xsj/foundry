// example_crate/src/config.rs
//
// A module as a single file. Loaded via `mod config;` in main.rs.
// This is the preferred style for leaf modules with no submodules.
//
// Demonstrates:
// - pub vs private items
// - pub(crate) for internal-only types
// - struct constructors that borrow (&str) and own (String)

// Allow dead_code for this example file — InternalKey, DB_URL_KEY,
// for_testing, etc. are pedagogical examples of pub(crate) patterns,
// not items used by the simple main.rs in this example crate.
#![allow(dead_code)]

/// Application configuration loaded at startup.
/// Public — this type is part of the module's public interface.
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    /// Private field — external code cannot read or write this directly.
    /// Callers use `is_debug_mode()` instead.
    debug: bool,
}

/// An internal-only configuration key. Available anywhere in this crate,
/// but not exposed to external users of this crate as a library.
pub(crate) struct InternalKey {
    pub(crate) name: &'static str,
}

/// Well-known configuration keys used internally.
pub(crate) const DB_URL_KEY: InternalKey = InternalKey { name: "DATABASE_URL" };

impl AppConfig {
    /// Load configuration from environment variables, falling back to defaults.
    pub fn load_from_env() -> AppConfig {
        // In a real service, you'd read from std::env::var.
        // We use defaults here to keep the example self-contained.
        AppConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            debug: false,
        }
    }

    /// Create a config for use in tests. `pub(crate)` — available in tests
    /// within this crate, but not to external consumers.
    pub(crate) fn for_testing() -> AppConfig {
        AppConfig {
            host: "127.0.0.1".to_string(),
            port: 0,  // OS picks the port
            debug: true,
        }
    }

    /// Public accessor for the private `debug` field.
    pub fn is_debug_mode(&self) -> bool {
        self.debug
    }

    /// Returns the socket address as a formatted string.
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// Unit tests for the config module.
// #[cfg(test)] means this block is only compiled when running tests.
#[cfg(test)]
mod tests {
    use super::*;  // bring everything from the parent module into scope

    #[test]
    fn test_addr_formatting() {
        let cfg = AppConfig {
            host: "localhost".to_string(),
            port: 3000,
            debug: false,
        };
        assert_eq!(cfg.addr(), "localhost:3000");
    }

    #[test]
    fn test_debug_accessor() {
        // Can use pub(crate) fn here because we're inside the crate
        let cfg = AppConfig::for_testing();
        assert!(cfg.is_debug_mode());
    }
}
