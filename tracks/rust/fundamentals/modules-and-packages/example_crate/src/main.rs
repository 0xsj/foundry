// example_crate/src/main.rs
//
// The binary crate root. This is where `rustc` starts when building the
// executable. All module declarations live in (or flow from) this file.
//
// Run: cargo run

// Declare modules — each corresponds to a file in src/.
// Without these declarations, config.rs and handlers/ are invisible.
mod config;
mod handlers;

// Bring specific names into scope.
// Without `use`, we'd write `config::AppConfig::default()` everywhere.
use config::AppConfig;
// LivenessResult is re-exported at the handlers level via `pub use` in handlers/mod.rs.
// We can import it from there rather than the deeper path handlers::health::LivenessResult.
use handlers::LivenessResult;

fn main() {
    // Load configuration using the config module's public API.
    let cfg = AppConfig::load_from_env();
    println!("Starting server on {}:{}", cfg.host, cfg.port);

    // Call into the handlers module tree.
    // handlers::health is a submodule declared in handlers/mod.rs.
    let result = handlers::health::check_liveness();
    match result {
        LivenessResult::Alive => println!("Health check: OK"),
        LivenessResult::Degraded(msg) => println!("Health check: DEGRADED — {}", msg),
    }

    // Feature-gated behavior: only compiled when `--features verbose` is passed.
    #[cfg(feature = "verbose")]
    println!("[verbose] server initialization complete");
}
