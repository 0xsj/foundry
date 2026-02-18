// url-shortener/src/main.rs
//
// The binary crate entry point. This file contains only dispatch logic —
// no business logic lives here. All the real work is in the library modules.
//
// This is the correct shape for a binary that wraps a library:
//   - Parse args via the api module
//   - Initialize state
//   - Dispatch to the right module
//   - Print output
//   - Exit on error
//
// Run: cargo run -- shorten https://example.com/long/path
// Run: cargo run -- resolve <key>
// Run: cargo run -- stats

use url_shortener::prelude::*;

fn main() {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    let mut store = Store::new();
    let mut stats = Stats::new();

    match parse_args(&raw_args) {
        Ok(Command::Shorten { url }) => {
            let key = shorten(&url);
            store.insert(key.clone(), url.clone());
            stats.record_shorten();
            println!("Shortened: {} -> {}", url, key);
        }
        Ok(Command::Resolve { key }) => match store.resolve(&key) {
            Some(url) => {
                stats.record_resolve();
                println!("Resolved: {} -> {}", key, url);
            }
            None => {
                eprintln!("Error: key '{}' not found", key);
                std::process::exit(1);
            }
        },
        Ok(Command::Remove { key }) => {
            if store.remove(&key) {
                stats.record_remove();
                println!("Removed: {}", key);
            } else {
                eprintln!("Error: key '{}' not found", key);
                std::process::exit(1);
            }
        }
        Ok(Command::Stats) => {
            println!("{}", stats.report());
            println!("Total entries in store: {}", store.len());
        }
        Ok(Command::Help) => {
            println!("Usage:");
            println!("  shorten <url>  — shorten a URL");
            println!("  resolve <key>  — resolve a short key to its URL");
            println!("  remove <key>   — remove a key from the store");
            println!("  stats          — show usage statistics");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
