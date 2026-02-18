// Functions — Rust
//
// Demonstrates: function definitions, expression returns, multiple return values,
// early returns as guard clauses, diverging functions (-> !), function pointers,
// and patterns in parameters.
//
// Run: rustc functions.rs && ./functions

// ---- Basic function syntax ----

// Returns the last expression. No semicolon = return value.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Returns unit () — return type omitted.
fn log_message(level: &str, msg: &str) {
    println!("[{}] {}", level, msg);
}

// If/else as an expression — last value of each branch is returned.
fn classify_latency_ms(ms: u64) -> &'static str {
    if ms < 100 {
        "fast"
    } else if ms < 500 {
        "acceptable"
    } else {
        "slow"
    }
}

// ---- Multiple return values via tuple ----

// Returns (host, port) as a tuple.
fn parse_address(addr: &str) -> (String, u16) {
    // Real parsing omitted for brevity — split on ":"
    let parts: Vec<&str> = addr.splitn(2, ':').collect();
    if parts.len() == 2 {
        let host = parts[0].to_string();
        let port: u16 = parts[1].parse().unwrap_or(80);
        (host, port)
    } else {
        (addr.to_string(), 80)
    }
}

// ---- Early returns as guard clauses ----

// Early returns handle invalid states up front. Happy path is the last expression.
fn compute_retry_delay_ms(attempt: u32, base_ms: u64, max_ms: u64) -> Result<u64, String> {
    if base_ms == 0 {
        return Err(String::from("base_ms must be greater than 0"));
    }
    if attempt > 32 {
        return Err(format!("attempt {} exceeds maximum of 32", attempt));
    }
    if max_ms < base_ms {
        return Err(format!("max_ms ({}) must be >= base_ms ({})", max_ms, base_ms));
    }

    // Happy path: exponential backoff
    let delay = base_ms.saturating_mul(1u64 << attempt).min(max_ms);
    Ok(delay)
}

// ---- Patterns in parameters ----

// Destructure a tuple parameter directly.
fn format_coordinate(&(lat, lon): &(f64, f64)) -> String {
    format!("{:.4}°N, {:.4}°E", lat, lon)
}

// Ignore a parameter explicitly.
fn process_with_id(id: u64, _payload: &[u8]) -> String {
    format!("processed id={}", id)
}

// ---- Function pointers ----

// A function pointer: fn(i32) -> i32.
// All non-capturing closures and regular functions coerce to fn pointers.
fn apply_transform(values: &[i32], transform: fn(i32) -> i32) -> Vec<i32> {
    values.iter().map(|&v| transform(v)).collect()
}

fn double(x: i32) -> i32 {
    x * 2
}

fn square(x: i32) -> i32 {
    x * x
}

// ---- Diverging functions ----

// -> ! means this function never returns.
// Useful for fatal errors, infinite loops, or process exit.
fn fatal(msg: &str) -> ! {
    // In real code: eprintln!("FATAL: {}", msg); std::process::exit(1);
    // We use panic! here to keep the demo runnable.
    panic!("fatal: {}", msg);
}

// ! coerces to any type. This lets diverging expressions appear in match arms.
fn get_required_env(key: &str) -> String {
    // We'll use a simple map for demo purposes (no std::env in single-file compile)
    let env = [("API_KEY", "secret-abc"), ("PORT", "8080")];
    env.iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| fatal(&format!("required env var '{}' not set", key)))
        // unwrap_or_else takes FnOnce() -> T. fatal() returns ! which coerces to T.
}

// ---- Shadowing in function bodies ----

// Shadowing lets you transform a value and reuse the name.
// Common pattern: parse a string, shadow with the parsed value.
fn parse_and_validate_port(raw: &str) -> Result<u16, String> {
    let port = raw.trim();                // &str — trimmed view
    let port: u16 = port               // u16 — shadows the &str
        .parse()
        .map_err(|_| format!("'{}' is not a valid port number", raw))?;

    if port < 1024 {
        return Err(format!("port {} is reserved (< 1024)", port));
    }

    Ok(port)
}

// ---- Const functions ----
// (Brief preview — full generics/const module covers this in depth)
//
// const fn can be evaluated at compile time when called in const context.
const fn max_buffer_size(base: usize, overhead: usize) -> usize {
    base + overhead
}

const MAX_PACKET_SIZE: usize = max_buffer_size(1400, 40); // computed at compile time

fn main() {
    // -- add --
    println!("add(3, 4) = {}", add(3, 4));

    // -- log_message --
    log_message("INFO", "server starting");

    // -- classify_latency_ms --
    for ms in [50u64, 300, 800] {
        println!("{}ms is {}", ms, classify_latency_ms(ms));
    }

    // -- parse_address --
    let (host, port) = parse_address("db.internal:5432");
    println!("\nparse_address: host={}, port={}", host, port);

    let (host2, port2) = parse_address("localhost");
    println!("parse_address (no port): host={}, port={}", host2, port2);

    // -- compute_retry_delay_ms --
    println!("\nRetry delays:");
    for attempt in [0u32, 1, 3, 5] {
        match compute_retry_delay_ms(attempt, 100, 10_000) {
            Ok(delay) => println!("  attempt {}: {}ms", attempt, delay),
            Err(e) => println!("  attempt {}: error: {}", attempt, e),
        }
    }
    match compute_retry_delay_ms(0, 0, 1000) {
        Ok(_) => println!("  unexpected ok"),
        Err(e) => println!("  validation error: {}", e),
    }

    // -- patterns in parameters --
    let coord = (51.5074, -0.1278);
    println!("\n{}", format_coordinate(&coord));
    println!("{}", process_with_id(42, b"payload-data"));

    // -- function pointers --
    let values = [1, 2, 3, 4, 5];
    let doubled = apply_transform(&values, double);
    let squared = apply_transform(&values, square);
    println!("\ndoubled: {:?}", doubled);
    println!("squared: {:?}", squared);

    // Non-capturing closure coerces to fn pointer:
    let tripled = apply_transform(&values, |x| x * 3);
    println!("tripled: {:?}", tripled);

    // -- parse_and_validate_port --
    println!("\nPort parsing:");
    for raw in ["8080", " 3000 ", "80", "abc", "65535"] {
        match parse_and_validate_port(raw) {
            Ok(p) => println!("  '{}' -> port {}", raw, p),
            Err(e) => println!("  '{}' -> error: {}", raw, e),
        }
    }

    // -- get_required_env --
    println!("\nRequired env:");
    println!("  API_KEY = {}", get_required_env("API_KEY"));
    println!("  PORT = {}", get_required_env("PORT"));

    // -- const fn --
    println!("\nMAX_PACKET_SIZE (const) = {}", MAX_PACKET_SIZE);
}
