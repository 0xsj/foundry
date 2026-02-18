// error_strategies.rs — anyhow vs thiserror, Box<dyn Error>, panic vs Result
//
// Run: rustc error_strategies.rs && ./error_strategies
//
// NOTE: This file uses only the standard library. The anyhow/thiserror
// sections are shown as commented-out code with explanations. To run those
// examples, create a Cargo project and add the crates to Cargo.toml.
//
// This file focuses on the *reasoning* behind each strategy — when to reach for
// which tool, and why the choice matters.

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// STRATEGY 1: Box<dyn Error> — the standard library catch-all
// ---------------------------------------------------------------------------
//
// Box<dyn Error> is the "just make it work" option. Any type implementing
// std::error::Error can be converted to Box<dyn Error> automatically.
// The downside: you cannot match on the concrete error type without downcasting.
//
// Use Box<dyn Error> in:
// - Quick scripts and examples
// - main() functions that just need to propagate
// - Intermediate layers that don't need to inspect error details
//
// Avoid Box<dyn Error> in:
// - Library APIs (callers can't match on variants)
// - When you need thread safety: use Box<dyn Error + Send + Sync>

fn read_and_parse(path: &str) -> Result<u32, Box<dyn std::error::Error>> {
    // ? works here because From<io::Error> and From<ParseIntError> for Box<dyn Error>
    // are provided by the standard library as blanket impls.
    let contents = std::fs::read_to_string(path)?;  // io::Error -> Box<dyn Error>
    let n: u32 = contents.trim().parse()?;           // ParseIntError -> Box<dyn Error>
    Ok(n)
}

// Downcasting: recovering the concrete type from Box<dyn Error>
fn handle_boxed_error(e: Box<dyn std::error::Error>) {
    if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
        match io_err.kind() {
            std::io::ErrorKind::NotFound => println!("  file not found — create it first"),
            std::io::ErrorKind::PermissionDenied => println!("  permission denied"),
            _ => println!("  I/O error: {}", io_err),
        }
    } else if let Some(parse_err) = e.downcast_ref::<std::num::ParseIntError>() {
        println!("  parse error: {} — check the file contents", parse_err);
    } else {
        println!("  unknown error: {}", e);
    }
}

// ---------------------------------------------------------------------------
// STRATEGY 2: Custom enum + Display + Error (manual, no crates)
// ---------------------------------------------------------------------------
//
// The right choice for library code that must be used without external crates,
// or when you want to understand what derive macros generate.

#[derive(Debug)]
pub enum ConfigError {
    FileMissing(String),
    ParseError { line: usize, message: String },
    InvalidValue { key: String, value: String },
    Io(std::io::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::FileMissing(path) => write!(f, "config file not found: {}", path),
            ConfigError::ParseError { line, message } => {
                write!(f, "parse error on line {}: {}", line, message)
            }
            ConfigError::InvalidValue { key, value } => {
                write!(f, "invalid value '{}' for key '{}'", value, key)
            }
            ConfigError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let ConfigError::Io(e) = self { Some(e) } else { None }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

// ---------------------------------------------------------------------------
// STRATEGY 3: What anyhow looks like (shown with stdlib approximation)
// ---------------------------------------------------------------------------
//
// anyhow::Error is essentially a newtype over Box<dyn Error + Send + Sync + 'static>
// that adds:
//   - .context("message") to wrap with a message string
//   - .with_context(|| "message") for lazy context
//   - .chain() for iterating the error chain
//   - {:#} alternate format that prints the full chain inline
//
// Here we simulate the context-adding pattern without the crate:

struct WithContext<E: std::error::Error> {
    context: String,
    source: E,
}

impl<E: std::error::Error> fmt::Display for WithContext<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

impl<E: std::error::Error> fmt::Debug for WithContext<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.context, self.source)
    }
}

impl<E: std::error::Error + 'static> std::error::Error for WithContext<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

fn parse_config_value(
    map: &HashMap<&str, &str>,
    key: &str,
) -> Result<u32, WithContext<std::num::ParseIntError>> {
    let raw = map.get(key).copied().unwrap_or("0");
    raw.parse::<u32>().map_err(|e| WithContext {
        context: format!("parsing config key '{}'", key),
        source: e,
    })
}

// ---------------------------------------------------------------------------
// STRATEGY 4: panic! — when invariants are violated
// ---------------------------------------------------------------------------

/// A connection pool that must always be initialized before use.
struct ConnectionPool {
    connections: Vec<String>,
    max_size: usize,
}

impl ConnectionPool {
    fn new(max_size: usize) -> Self {
        assert!(max_size > 0, "ConnectionPool max_size must be > 0");
        Self {
            connections: Vec::new(),
            max_size,
        }
    }

    // The caller guarantees the pool is not empty before calling this.
    // It's a programming error to call take() on an empty pool — panic is correct.
    #[allow(dead_code)]
    fn take(&mut self) -> String {
        if self.connections.is_empty() {
            panic!(
                "BUG: ConnectionPool::take() called on empty pool — \
                 caller must check is_empty() first"
            );
        }
        self.connections.pop().unwrap()
    }

    // This, however, is a recoverable failure — the pool might be full due to
    // load, which is an expected operational condition, not a bug.
    fn add(&mut self, conn: String) -> Result<(), String> {
        if self.connections.len() >= self.max_size {
            Err(format!("pool is full ({} connections)", self.max_size))
        } else {
            self.connections.push(conn);
            Ok(())
        }
    }
}

/// Demonstrates the panic vs Result decision with commentary.
fn panic_vs_result_examples() {
    // CORRECT USE OF PANIC: startup validation
    // If this panics, the program should not start. No user request caused it.
    let _pool = ConnectionPool::new(10);
    // ConnectionPool::new(0);  // would panic: BUG in calling code

    // CORRECT USE OF RESULT: operational failure
    let mut pool = ConnectionPool::new(2);
    pool.add("conn-1".to_string()).unwrap();
    pool.add("conn-2".to_string()).unwrap();
    match pool.add("conn-3".to_string()) {
        Ok(()) => println!("  added connection"),
        Err(e) => println!("  expected: pool full — {}", e),
    }

    // WRONG: Using unwrap() on user-controlled data
    // This would panic if a user sends "abc" as a port number.
    // let port: u16 = user_input.parse().unwrap();  // DON'T DO THIS

    // RIGHT: Using ? or map_err on user-controlled data
    let user_input = "abc";
    match user_input.parse::<u16>() {
        Ok(p) => println!("  port: {}", p),
        Err(e) => println!("  bad port input: {}", e),
    }
}

// ---------------------------------------------------------------------------
// STRATEGY 5: Choosing between strategies — the decision tree
// ---------------------------------------------------------------------------

fn strategy_decision_guide() {
    println!("\n  Decision guide:");
    println!("  ─────────────────────────────────────────────────────────────");
    println!("  Q: Are you writing a library or an application?");
    println!("     Library  → use custom enum errors (thiserror in practice)");
    println!("     App      → use anyhow (in practice) or Box<dyn Error>  ");
    println!();
    println!("  Q: Do callers need to match on specific error variants?");
    println!("     Yes  → thiserror / custom enum");
    println!("     No   → anyhow / Box<dyn Error>");
    println!();
    println!("  Q: Is this a programming bug or an operational failure?");
    println!("     Bug       → panic! (wrong input from developer)");
    println!("     Failure   → Result (expected failure from environment)");
    println!();
    println!("  Q: Is this a temp prototype or production code?");
    println!("     Prototype → anyhow or .unwrap() freely");
    println!("     Production → thiserror for structured errors");
    println!("  ─────────────────────────────────────────────────────────────");
}

/*
// ---------------------------------------------------------------------------
// What these would look like with the actual crates (requires Cargo.toml):
// ---------------------------------------------------------------------------

// With thiserror (library error types):
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    FileMissing(String),

    #[error("parse error on line {line}: {message}")]
    ParseError { line: usize, message: String },

    #[error("invalid value '{value}' for key '{key}'")]
    InvalidValue { key: String, value: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}


// With anyhow (application error handling):
use anyhow::{Context, Result, bail, ensure};

fn load_config(path: &str) -> Result<HashMap<String, String>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config from {}", path))?;

    let mut map = HashMap::new();
    for (i, line) in contents.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }

        let (key, value) = line.split_once('=')
            .with_context(|| format!("line {}: expected 'key=value', got '{}'", i + 1, line))?;

        ensure!(!key.is_empty(), "line {}: key cannot be empty", i + 1);
        map.insert(key.trim().to_string(), value.trim().to_string());
    }
    Ok(map)
}

// Key anyhow features:
// - anyhow!("message")          — create an ad-hoc error
// - bail!("message")            — return Err(anyhow!(...))
// - ensure!(cond, "message")    — return Err if condition is false
// - .context("msg")             — wrap error with string (eager)
// - .with_context(|| "msg")     — wrap error with closure (lazy)
// - {:#} display                — prints "outer: inner: deepest" chain
// - .downcast_ref::<T>()        — recover concrete error type

*/

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Box<dyn Error> — catch-all strategy ===");
    match read_and_parse("/nonexistent/file.txt") {
        Ok(n) => println!("value: {}", n),
        Err(e) => {
            println!("handling boxed error:");
            handle_boxed_error(e);
        }
    }

    println!("\n=== Custom enum error — structured strategy ===");
    let err = ConfigError::InvalidValue {
        key: "max_connections".to_string(),
        value: "-1".to_string(),
    };
    println!("Display: {}", err);
    println!("Debug:   {:?}", err);
    println!("source:  {:?}", std::error::Error::source(&err));

    let io_err = ConfigError::Io(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "cannot read config.toml",
    ));
    println!("Io Display: {}", io_err);
    if let Some(src) = std::error::Error::source(&io_err) {
        println!("  source: {}", src);
    }

    println!("\n=== Context wrapping (stdlib approximation of anyhow) ===");
    let mut config_map = HashMap::new();
    config_map.insert("timeout", "five thousand");  // bad value

    match parse_config_value(&config_map, "timeout") {
        Ok(v) => println!("timeout: {}", v),
        Err(e) => {
            println!("error: {}", e);
            if let Some(src) = std::error::Error::source(&e) {
                println!("  caused by: {}", src);
            }
        }
    }

    println!("\n=== panic vs Result decision examples ===");
    panic_vs_result_examples();

    strategy_decision_guide();
}
