// result_basics.rs — Result<T,E>, Option<T>, the ? operator, and combinators
//
// Run: rustc result_basics.rs && ./result_basics
//
// This file demonstrates idiomatic Result/Option usage — the day-to-day vocabulary
// you'll use in almost every Rust function that can fail.

use std::collections::HashMap;
use std::num::ParseIntError;

// ---------------------------------------------------------------------------
// 1. Returning Result from a function
// ---------------------------------------------------------------------------

/// Parse a port from a string. Returns Err if the string is not a valid u16
/// or if the value is zero (ports start at 1).
fn parse_port(s: &str) -> Result<u16, String> {
    let port: u16 = s
        .trim()
        .parse()
        .map_err(|e: ParseIntError| format!("'{}' is not a valid port: {}", s, e))?;

    if port == 0 {
        return Err(String::from("port 0 is not a valid port number"));
    }

    Ok(port)
}

// ---------------------------------------------------------------------------
// 2. The ? operator — early return on error, with From conversion
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Parse(ParseIntError),
    MissingKey(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(e)         => write!(f, "I/O error: {}", e),
            AppError::Parse(e)      => write!(f, "parse error: {}", e),
            AppError::MissingKey(k) => write!(f, "missing key: '{}'", k),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self { AppError::Io(e) }
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self { AppError::Parse(e) }
}

/// Look up a key in a map and parse its value as a u32.
/// ? converts ParseIntError -> AppError via From automatically.
fn get_u32(map: &HashMap<&str, &str>, key: &str) -> Result<u32, AppError> {
    let value = map.get(key)
        .ok_or_else(|| AppError::MissingKey(key.to_string()))?;  // Option -> Result

    let n: u32 = value.parse()?;  // ParseIntError -> AppError::Parse via From
    Ok(n)
}

// ---------------------------------------------------------------------------
// 3. Combinators: map, map_err, and_then, or_else
// ---------------------------------------------------------------------------

/// Combinators let you transform results without unpacking them manually.
fn combinator_examples() {
    // map: transform the Ok value
    let doubled: Result<i32, &str> = Ok(5).map(|n| n * 2);
    println!("map Ok(5) * 2 = {:?}", doubled);  // Ok(10)

    let still_err: Result<i32, &str> = Err("bad").map(|n: i32| n * 2);
    println!("map Err = {:?}", still_err);  // Err("bad") — unchanged

    // map_err: transform the Err value
    let mapped_err: Result<i32, String> = Err(404i32).map_err(|code| format!("HTTP {}", code));
    println!("map_err = {:?}", mapped_err);  // Err("HTTP 404")

    // and_then: chain a fallible operation (flatMap)
    let chained: Result<u32, String> = "42"
        .parse::<u32>()
        .map_err(|e| e.to_string())
        .and_then(|n| if n > 100 { Err("too large".into()) } else { Ok(n * 2) });
    println!("and_then = {:?}", chained);  // Ok(84)

    // or_else: recover from an error
    let recovered: Result<i32, String> = Err("transient".to_string())
        .or_else(|_| Ok::<i32, String>(0));
    println!("or_else = {:?}", recovered);  // Ok(0)

    // ok: discard the error, convert to Option
    let maybe: Option<i32> = Ok::<i32, &str>(42).ok();
    println!("ok() = {:?}", maybe);  // Some(42)

    let nothing: Option<i32> = Err::<i32, &str>("nope").ok();
    println!("err.ok() = {:?}", nothing);  // None
}

// ---------------------------------------------------------------------------
// 4. Option combinators
// ---------------------------------------------------------------------------

struct Config {
    values: HashMap<String, String>,
}

impl Config {
    fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    /// Returns the timeout in milliseconds, or 5000 as a default.
    fn timeout_ms(&self) -> u64 {
        self.get("timeout_ms")
            .and_then(|s| s.parse::<u64>().ok())  // None if parse fails
            .unwrap_or(5000)                        // default value
    }

    /// Returns the log level, or None if not set or unrecognized.
    fn log_level(&self) -> Option<&str> {
        self.get("log_level")
            .filter(|&level| matches!(level, "debug" | "info" | "warn" | "error"))
    }
}

// ---------------------------------------------------------------------------
// 5. unwrap variants — when each is appropriate
// ---------------------------------------------------------------------------

fn unwrap_variants() {
    // unwrap() — fine in tests, or for values that truly cannot fail
    let port: u16 = "8080".parse().unwrap();
    println!("port: {}", port);

    // expect() — always better than unwrap() in production code
    // The message explains WHY you expected success
    let admin_port: u16 = "9090"
        .parse()
        .expect("hardcoded port string must be valid");
    println!("admin_port: {}", admin_port);

    // unwrap_or — provide a default without computing it (eager)
    let timeout: u64 = "bad".parse().unwrap_or(5000);
    println!("timeout: {}", timeout);  // 5000

    // unwrap_or_else — provide a default lazily (only computed on Err)
    let value: String = None.unwrap_or_else(|| {
        // This closure only runs when the Option is None
        "default".to_string()
    });
    println!("value: {}", value);

    // unwrap_or_default — use the type's Default impl
    let count: i32 = "bad".parse().unwrap_or_default();
    println!("count: {}", count);  // 0 (i32's default)
}

// ---------------------------------------------------------------------------
// 6. Collecting Results from iterators
// ---------------------------------------------------------------------------

fn collect_results() {
    let inputs = vec!["80", "443", "8080"];

    // collect::<Result<Vec<_>, _>>: all succeed or first error wins
    let ports: Result<Vec<u16>, _> = inputs.iter()
        .map(|s| s.parse::<u16>())
        .collect();
    println!("all valid: {:?}", ports);  // Ok([80, 443, 8080])

    let with_bad = vec!["80", "not_a_port", "8080"];
    let result: Result<Vec<u16>, _> = with_bad.iter()
        .map(|s| s.parse::<u16>())
        .collect();
    println!("with bad: {:?}", result);  // Err(ParseIntError)

    // filter_map + ok(): skip failures silently
    let valid: Vec<u16> = with_bad.iter()
        .filter_map(|s| s.parse::<u16>().ok())
        .collect();
    println!("filter_map ok: {:?}", valid);  // [80, 8080]
}

// ---------------------------------------------------------------------------
// 7. Converting between Option and Result
// ---------------------------------------------------------------------------

fn option_to_result_examples() {
    let map: HashMap<&str, &str> = [("host", "localhost")].iter().cloned().collect();

    // ok_or: convert Option to Result with an eager error value
    let host: Result<&&str, &str> = map.get("host").ok_or("host key not found");
    println!("ok_or: {:?}", host);  // Ok("localhost")

    // ok_or_else: convert Option to Result with a lazy error closure
    let port: Result<&&str, String> = map.get("port")
        .ok_or_else(|| format!("key 'port' not found in {} entries", map.len()));
    println!("ok_or_else: {:?}", port);  // Err("key 'port' not found in 1 entries")

    // option.transpose(): Option<Result<T, E>> -> Result<Option<T>, E>
    let parsed: Option<Result<u16, _>> = Some("8080".parse::<u16>());
    let transposed: Result<Option<u16>, _> = parsed.transpose();
    println!("transpose: {:?}", transposed);  // Ok(Some(8080))
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== parse_port ===");
    println!("{:?}", parse_port("8080"));    // Ok(8080)
    println!("{:?}", parse_port("99999"));   // Err — out of range for u16
    println!("{:?}", parse_port("0"));       // Err("port 0 is not a valid port number")
    println!("{:?}", parse_port("abc"));     // Err("'abc' is not a valid port: ...")

    println!("\n=== get_u32 with ? operator ===");
    let mut map = HashMap::new();
    map.insert("timeout", "5000");
    map.insert("retries", "3");
    map.insert("bad_value", "notanumber");

    println!("{:?}", get_u32(&map, "timeout"));    // Ok(5000)
    println!("{:?}", get_u32(&map, "missing"));    // Err(MissingKey("missing"))
    println!("{:?}", get_u32(&map, "bad_value"));  // Err(Parse(...))

    println!("\n=== combinators ===");
    combinator_examples();

    println!("\n=== Config option combinators ===");
    let config = Config {
        values: [
            ("timeout_ms".to_string(), "3000".to_string()),
            ("log_level".to_string(), "info".to_string()),
        ]
        .iter()
        .cloned()
        .collect(),
    };
    println!("timeout_ms: {}", config.timeout_ms());
    println!("log_level: {:?}", config.log_level());

    let empty_config = Config { values: HashMap::new() };
    println!("default timeout_ms: {}", empty_config.timeout_ms());

    println!("\n=== unwrap variants ===");
    unwrap_variants();

    println!("\n=== collecting Results ===");
    collect_results();

    println!("\n=== Option <-> Result conversions ===");
    option_to_result_examples();
}
