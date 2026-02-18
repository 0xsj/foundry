// Custom Iterators: Range, Fibonacci, and IntoIterator
//
// Demonstrates: Implementing Iterator for custom types, size_hint(),
// IntoIterator for enabling for-loops, infinite iterators with take().
//
// Run: rustc custom_iter.rs && ./custom_iter

use std::fmt;

// ---------------------------------------------------------------------------
// Example 1: A bounded range iterator with exact size hints
// ---------------------------------------------------------------------------

/// Yields integers from `start` (inclusive) to `end` (exclusive).
/// Like std::ops::Range but built from scratch to show the mechanics.
struct CountUp {
    current: i64,
    end: i64,
}

impl CountUp {
    fn new(start: i64, end: i64) -> Self {
        CountUp {
            current: start,
            end,
        }
    }
}

impl Iterator for CountUp {
    type Item = i64;

    fn next(&mut self) -> Option<i64> {
        if self.current < self.end {
            let val = self.current;
            self.current += 1;
            Some(val)
        } else {
            None
        }
    }

    /// Override size_hint so collect() can pre-allocate.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = if self.current < self.end {
            (self.end - self.current) as usize
        } else {
            0
        };
        (remaining, Some(remaining))
    }
}

// Implement ExactSizeIterator since our size_hint is exact.
// This gives us .len() on the iterator.
impl ExactSizeIterator for CountUp {}

// ---------------------------------------------------------------------------
// Example 2: An infinite Fibonacci iterator
// ---------------------------------------------------------------------------

struct Fibonacci {
    a: u64,
    b: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { a: 0, b: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let current = self.a;
        self.a = self.b;
        // Use checked_add to detect overflow instead of panicking
        self.b = match current.checked_add(self.b) {
            Some(sum) => sum,
            None => return None, // Stop on overflow
        };
        Some(current)
    }

    // size_hint for infinite: lower bound is large, upper is None
    fn size_hint(&self) -> (usize, Option<usize>) {
        // We'll produce at least some values, but potentially many.
        // Since we stop on overflow, we can't say exactly.
        (1, None)
    }
}

impl fmt::Display for Fibonacci {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fibonacci(next={})", self.a)
    }
}

// ---------------------------------------------------------------------------
// Example 3: IntoIterator for a collection type
// ---------------------------------------------------------------------------

/// A ring buffer of sensor readings. Fixed capacity, overwrites oldest.
struct SensorReadings {
    buffer: Vec<f64>,
    label: String,
}

impl SensorReadings {
    fn new(label: &str) -> Self {
        SensorReadings {
            buffer: Vec::new(),
            label: label.to_string(),
        }
    }

    fn push(&mut self, value: f64) {
        self.buffer.push(value);
    }

    /// Borrow-based iteration — readings remain usable after the loop.
    fn iter(&self) -> std::slice::Iter<'_, f64> {
        self.buffer.iter()
    }
}

/// Consuming iteration — moves readings out of the struct.
/// After `for reading in sensor_readings`, the struct is gone.
impl IntoIterator for SensorReadings {
    type Item = f64;
    type IntoIter = std::vec::IntoIter<f64>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}

/// Borrow-based IntoIterator — enables `for reading in &sensor_readings`.
impl<'a> IntoIterator for &'a SensorReadings {
    type Item = &'a f64;
    type IntoIter = std::slice::Iter<'a, f64>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.iter()
    }
}

// ---------------------------------------------------------------------------
// Example 4: Iterator that yields structured data
// ---------------------------------------------------------------------------

/// Parses "key=value" pairs from a config string, one at a time.
struct ConfigEntries<'a> {
    remaining: &'a str,
}

impl<'a> ConfigEntries<'a> {
    fn new(input: &'a str) -> Self {
        ConfigEntries { remaining: input }
    }
}

impl<'a> Iterator for ConfigEntries<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<(&'a str, &'a str)> {
        loop {
            if self.remaining.is_empty() {
                return None;
            }

            // Find the next line
            let (line, rest) = match self.remaining.find('\n') {
                Some(pos) => (&self.remaining[..pos], &self.remaining[pos + 1..]),
                None => (self.remaining, ""),
            };
            self.remaining = rest;

            // Skip empty lines and comments
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Parse key=value
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let value = trimmed[eq_pos + 1..].trim();
                return Some((key, value));
            }
            // Skip malformed lines
        }
    }
}

// ---------------------------------------------------------------------------
// main — demonstrate all four examples
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Example 1: CountUp (bounded range) ===\n");

    let counter = CountUp::new(1, 11);
    println!("Length before iteration: {}", counter.len());

    let evens: Vec<i64> = CountUp::new(1, 21)
        .filter(|n| n % 2 == 0)
        .collect();
    println!("Even numbers 1..21: {:?}", evens);

    let sum: i64 = CountUp::new(1, 101).sum();
    println!("Sum of 1..101: {}", sum);

    println!("\n=== Example 2: Fibonacci (infinite) ===\n");

    let first_fifteen: Vec<u64> = Fibonacci::new().take(15).collect();
    println!("First 15 Fibonacci: {:?}", first_fifteen);

    // Find the first Fibonacci number above 1000
    let big = Fibonacci::new().find(|&n| n > 1000);
    println!("First Fibonacci > 1000: {:?}", big);

    // Sum of even Fibonacci numbers below 4,000,000 (Project Euler #2)
    let euler_2: u64 = Fibonacci::new()
        .take_while(|&n| n < 4_000_000)
        .filter(|n| n % 2 == 0)
        .sum();
    println!("Sum of even Fibonacci < 4M: {}", euler_2);

    println!("\n=== Example 3: SensorReadings (IntoIterator) ===\n");

    let mut readings = SensorReadings::new("thermostat-1");
    readings.push(22.5);
    readings.push(23.1);
    readings.push(21.8);
    readings.push(24.2);
    readings.push(22.9);

    // Borrow-based iteration: readings survives
    let avg: f64 = readings.iter().sum::<f64>() / readings.buffer.len() as f64;
    println!("Sensor '{}' average: {:.1}C", readings.label, avg);

    // for x in &readings uses our IntoIterator for &SensorReadings
    print!("All readings: ");
    for r in &readings {
        print!("{:.1} ", r);
    }
    println!();

    // Consuming iteration: readings is moved
    let above_23: Vec<f64> = readings
        .into_iter()
        .filter(|&r| r > 23.0)
        .collect();
    println!("Readings above 23C: {:?}", above_23);
    // readings is GONE here — can't use it

    println!("\n=== Example 4: ConfigEntries (structured data) ===\n");

    let config = "\
# Database config
host = localhost
port = 5432
database = myapp

# Auth
jwt_secret = super-secret-key
token_ttl = 3600
";

    let entries: Vec<(&str, &str)> = ConfigEntries::new(config).collect();
    for (key, value) in &entries {
        println!("  {} = {}", key, value);
    }
    println!("\nTotal config entries: {}", entries.len());

    // Use iterator adapters on config entries
    let has_secret = ConfigEntries::new(config)
        .any(|(key, _)| key.contains("secret"));
    println!("Contains secret key: {}", has_secret);

    // Collect into HashMap
    let map: std::collections::HashMap<&str, &str> =
        ConfigEntries::new(config).collect();
    println!("Database: {}", map.get("database").unwrap_or(&"<not set>"));
}
