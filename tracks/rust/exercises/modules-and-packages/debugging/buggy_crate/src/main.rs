// config-loader/src/main.rs
//
// Entry point for the config loading service.
// This file has module and import bugs — fix them so the crate builds
// and all tests pass.

// BUG 1: This import path is wrong. The module is declared as `config`
// but the use path tries to import from a non-existent nested module.
use crate::config::AppConfig;

// BUG 2: `util` is never declared as a module anywhere in this crate,
// but it's referenced here. The actual module file exists at src/util.rs.
use util::parse_duration;

mod config;
// Missing: mod util;

fn main() {
    // Load config applying defaults for missing values
    let raw = config::load_raw("config.toml");

    // BUG 3: load_defaults is defined in config.rs but is private.
    // It's used here in main.rs (a sibling, not a child), so it needs
    // appropriate visibility.
    let cfg = config::load_defaults(raw);

    let timeout = parse_duration(&cfg.timeout_str);
    println!("Config loaded: host={} timeout={:?}ms", cfg.host, timeout);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_compiles() {
        // If this test runs, the crate compiles correctly.
        // The real test is just: does `cargo test` succeed without errors?
        let _ = config::load_raw("");
    }
}
