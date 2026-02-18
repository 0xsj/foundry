// generic_functions.rs — Generic functions, trait bounds, where clauses, turbofish
//
// Run: rustc generic_functions.rs && ./generic_functions

use std::fmt::{Debug, Display};

// ---------- 1. Basic generic function ----------

// Without any bounds, T is opaque — you can only move it around.
fn wrap_in_vec<T>(value: T) -> Vec<T> {
    vec![value]
}

// ---------- 2. Single trait bound ----------

// T: Display means we can format T with {}
// Without this bound, the println! would fail to compile.
fn log_value<T: Display>(label: &str, value: T) {
    println!("[{}] {}", label, value);
}

// ---------- 3. Multiple bounds with + ----------

// T must implement both Display and Clone.
// The compiler verifies both at every call site.
fn log_and_clone<T: Display + Clone>(value: T) -> T {
    println!("cloning: {}", value);
    value.clone()
}

// ---------- 4. where clauses for complex bounds ----------

// Inline bounds become hard to read when there are many.
// where clauses improve readability with no semantic difference.
fn serialize_debug<T>(items: &[T]) -> Vec<String>
where
    T: Debug + Display,
{
    items
        .iter()
        .map(|item| format!("display={} debug={:?}", item, item))
        .collect()
}

// ---------- 5. Multiple type parameters ----------

// K and V are independent type parameters, each with their own bounds.
fn build_summary<K, V>(key: K, value: V) -> String
where
    K: Display,
    V: Debug,
{
    format!("key={} value={:?}", key, value)
}

// ---------- 6. Returning a generic type ----------

// The return type is also generic — the caller decides what T is.
// T: Default provides the zero value for any type (0, "", false, etc.)
fn zero_vec<T: Default>(len: usize) -> Vec<T> {
    (0..len).map(|_| T::default()).collect()
}

// ---------- 7. Turbofish syntax ----------

// parse() is generic: it can parse into any T: FromStr.
// Type inference works when the target type is known from context.
// When it isn't, turbofish ::<Type> makes it explicit.
fn parse_with_default<T>(s: &str, default: T) -> T
where
    T: std::str::FromStr + Clone,
{
    s.parse().unwrap_or(default)
}

// ---------- 8. Real-world example: a typed config reader ----------

// A config reader that can convert any string value into the desired type.
// The where clause is placed after the signature for readability.
struct Config {
    values: std::collections::HashMap<String, String>,
}

impl Config {
    fn new() -> Self {
        Config {
            values: std::collections::HashMap::new(),
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    // Generic get: parses the stored string into whatever type T the caller wants.
    // T: FromStr means T can be parsed from a string.
    // T: Display means we can log the default value.
    fn get<T>(&self, key: &str, default: T) -> T
    where
        T: std::str::FromStr + Clone + Display,
    {
        match self.values.get(key) {
            Some(s) => s.parse().unwrap_or_else(|_| {
                println!("warn: could not parse '{}' for key '{}', using default: {}", s, key, default);
                default
            }),
            None => default,
        }
    }
}

// ---------- 9. Largest in a slice (classic generic example) ----------

// T: PartialOrd enables the > operator.
// This works for i32, f64, char, String — anything that supports ordering.
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list.iter() {
        if item > largest {
            largest = item;
        }
    }
    largest
}

// ---------- Main ----------

fn main() {
    // 1. Basic generic — no bounds
    let wrapped = wrap_in_vec(42u32);
    println!("wrapped: {:?}", wrapped);

    // 2. Single bound
    log_value("timeout_ms", 5000u32);
    log_value("service", "payments");

    // 3. Multiple bounds
    let s = String::from("hello");
    let s2 = log_and_clone(s.clone());
    println!("original: {}, clone: {}", s, s2);

    // 4. where clause
    let numbers = vec![1i32, 2, 3];
    let summaries = serialize_debug(&numbers);
    for summary in &summaries {
        println!("{}", summary);
    }

    // 5. Multiple type parameters
    let summary = build_summary("user_id", vec![1u32, 2, 3]);
    println!("{}", summary);

    // 6. Returning generic type — turbofish needed because T is only in return position
    let zeros: Vec<i32> = zero_vec(5);
    println!("zeros: {:?}", zeros);

    // 7. Turbofish
    //   Option A: type annotation on the binding
    let timeout: u32 = parse_with_default("30", 10u32);
    //   Option B: turbofish at the call site (same result)
    let retry_count = parse_with_default::<u32>("bad-value", 3);
    println!("timeout: {}, retry_count: {}", timeout, retry_count);

    // 8. Typed config reader
    let mut cfg = Config::new();
    cfg.set("max_connections", "100");
    cfg.set("host", "0.0.0.0");
    cfg.set("debug", "true");

    let max_conn:    u32    = cfg.get("max_connections", 10);
    let host:        String = cfg.get("host", String::from("localhost"));
    let debug:       bool   = cfg.get("debug", false);
    let missing_key: u32    = cfg.get("missing", 42);

    println!("max_connections={}, host={}, debug={}, missing={}", max_conn, host, debug, missing_key);

    // 9. Largest
    let numbers = vec![34i32, 50, 25, 100, 65];
    println!("largest: {}", largest(&numbers));

    let chars = vec!['y', 'm', 'a', 'q'];
    println!("largest char: {}", largest(&chars));
}
