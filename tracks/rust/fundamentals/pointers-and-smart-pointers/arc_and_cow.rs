// Arc, Cow, Deref, and Drop
//
// Run: rustc arc_and_cow.rs && ./arc_and_cow

use std::sync::{Arc, Mutex};
use std::thread;
use std::borrow::Cow;
use std::ops::Deref;

// ---------- Arc<T>: Shared Ownership Across Threads ----------

// A read-only config loaded once and shared across worker threads.
// No mutation needed — Arc alone is sufficient.
#[derive(Debug)]
struct AppConfig {
    database_url: String,
    max_connections: u32,
    timeout_ms: u64,
    feature_flags: Vec<String>,
}

impl AppConfig {
    fn is_feature_enabled(&self, flag: &str) -> bool {
        self.feature_flags.iter().any(|f| f == flag)
    }
}

fn arc_readonly_demo() {
    println!("=== Arc<T>: Read-Only Config Across Threads ===");

    let config = Arc::new(AppConfig {
        database_url: String::from("postgres://db.internal:5432/app"),
        max_connections: 25,
        timeout_ms: 5000,
        feature_flags: vec![
            String::from("new-dashboard"),
            String::from("async-export"),
        ],
    });

    let handles: Vec<_> = (0..3).map(|worker_id| {
        let cfg = Arc::clone(&config);
        thread::spawn(move || {
            println!("worker {}: db={}, feature={}",
                worker_id,
                cfg.database_url,
                cfg.is_feature_enabled("new-dashboard")
            );
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }

    // config is still valid here — Arc kept it alive through all threads
    println!("config still alive, count: {}", Arc::strong_count(&config));
}

// ---------- Arc<Mutex<T>>: Shared Mutable State Across Threads ----------

// A shared rate limiter that tracks request counts per client.
// Multiple threads process requests concurrently; all share the same state.
#[derive(Debug)]
struct RateLimiter {
    counts: std::collections::HashMap<String, u64>,
    limit: u64,
}

impl RateLimiter {
    fn new(limit: u64) -> RateLimiter {
        RateLimiter {
            counts: std::collections::HashMap::new(),
            limit,
        }
    }

    // Returns true if the request is allowed, false if rate limited
    fn check_and_record(&mut self, client_id: &str) -> bool {
        let count = self.counts.entry(client_id.to_string()).or_insert(0);
        if *count >= self.limit {
            return false;
        }
        *count += 1;
        true
    }

    fn get_count(&self, client_id: &str) -> u64 {
        *self.counts.get(client_id).unwrap_or(&0)
    }
}

fn arc_mutex_demo() {
    println!("\n=== Arc<Mutex<T>>: Shared Mutable State ===");

    let limiter = Arc::new(Mutex::new(RateLimiter::new(3)));

    // 4 threads each making 2 requests as "client-A"
    // With limit=3, some requests will be rejected
    let handles: Vec<_> = (0..4).map(|thread_id| {
        let lim = Arc::clone(&limiter);
        thread::spawn(move || {
            let client = "client-A";
            let allowed = lim.lock().unwrap().check_and_record(client);
            println!("thread {}: request {}", thread_id, if allowed { "allowed" } else { "BLOCKED" });
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }

    let final_count = limiter.lock().unwrap().get_count("client-A");
    println!("final count for client-A: {}", final_count);
    // At most 3 requests allowed (limit=3), rest blocked
}

// ---------- Cow<'a, B>: Clone on Write ----------

// HTTP header normalization: most headers are already lowercase,
// so we avoid allocating in the common case.
fn normalize_header_name(name: &str) -> Cow<str> {
    if name.chars().all(|c| c.is_lowercase() || c == '-') {
        Cow::Borrowed(name)         // already normalized — no allocation
    } else {
        Cow::Owned(name.to_lowercase())  // allocate only when needed
    }
}

// Path sanitization: most paths from trusted sources are clean
fn sanitize_path(path: &str) -> Cow<str> {
    // Check if any cleaning is needed
    if !path.contains("//") && !path.contains("./") {
        return Cow::Borrowed(path);
    }
    // Only allocate when sanitization is required
    let cleaned = path
        .replace("//", "/")
        .replace("./", "");
    Cow::Owned(cleaned)
}

// API error messages: most are static strings, but some include dynamic context
fn validate_json_field<'a>(field: &str, value: &str) -> Result<(), Cow<'a, str>> {
    if value.is_empty() {
        return Err(Cow::Borrowed("field cannot be empty"));
    }
    if value.len() > 255 {
        return Err(Cow::Owned(format!(
            "field '{}' exceeds 255 characters (got {})", field, value.len()
        )));
    }
    Ok(())
}

fn cow_demo() {
    println!("\n=== Cow<'a, B> ===");

    // Header normalization
    let headers = ["content-type", "Authorization", "X-Request-ID", "accept"];
    for h in &headers {
        let normalized = normalize_header_name(h);
        let allocated = matches!(normalized, Cow::Owned(_));
        println!("  {:20} -> {:20} (allocated: {})", h, normalized, allocated);
    }

    // Path sanitization
    let paths = ["/api/users", "/api//users", "./api/users", "/api/orders"];
    println!();
    for p in &paths {
        let clean = sanitize_path(p);
        let allocated = matches!(clean, Cow::Owned(_));
        println!("  {:20} -> {:20} (allocated: {})", p, clean, allocated);
    }

    // Validation errors
    println!();
    match validate_json_field("name", "") {
        Err(e) => println!("  error: {}", e),
        Ok(_) => {}
    }
    match validate_json_field("description", &"x".repeat(300)) {
        Err(e) => println!("  error: {}", e),
        Ok(_) => {}
    }
    match validate_json_field("email", "alice@example.com") {
        Ok(_) => println!("  email is valid"),
        Err(e) => println!("  error: {}", e),
    }
}

// ---------- Deref: How Smart Pointers Feel Transparent ----------

// A typed wrapper that restricts what operations are available.
// We implement Deref so users can call inner methods directly.
struct ReadOnly<T>(T);

impl<T> Deref for ReadOnly<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
    // Note: no DerefMut — intentionally read-only
}

// A credentials holder that lets you use the credentials
// but prevents accidental mutation through the type system.
struct Credentials {
    api_key: String,
    api_secret: String,
}

impl Credentials {
    fn new(key: &str, secret: &str) -> Credentials {
        Credentials {
            api_key: key.to_string(),
            api_secret: secret.to_string(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}:{}", self.api_key, self.api_secret)
    }
}

fn deref_demo() {
    println!("\n=== Deref Coercion ===");

    // Deref coercion chains:
    let s: Box<String> = Box::new(String::from("hello world"));

    // &Box<String> -> &String -> &str (two coercions in chain)
    fn takes_str(s: &str) { println!("  str: '{}'", s); }
    takes_str(&s);   // automatically coerces

    // Method calls work transparently:
    println!("  len via Box: {}", s.len());         // Box -> String -> len()
    println!("  upper via Box: {}", s.to_uppercase()); // Box -> String -> to_uppercase()

    // ReadOnly<Credentials>: Deref gives access to Credentials methods
    let creds = ReadOnly(Credentials::new("key_abc", "secret_xyz"));

    // Deref coercion: &ReadOnly<Credentials> -> &Credentials
    println!("  auth header: {}", creds.auth_header());
    println!("  api key: {}", creds.api_key);   // field access through Deref

    // But mutation is blocked at the type level:
    // creds.api_key = String::from("new"); // compile error: cannot borrow as mutable
}

// ---------- Drop: Custom Cleanup ----------

// A file handle that always flushes and closes on drop.
// The Drop impl ensures no resources are leaked even when
// the function returns early due to an error.
struct ManagedConnection {
    id: u32,
    host: String,
    active: bool,
}

impl ManagedConnection {
    fn new(id: u32, host: &str) -> ManagedConnection {
        println!("  [conn {}] opened to {}", id, host);
        ManagedConnection {
            id,
            host: host.to_string(),
            active: true,
        }
    }

    fn execute(&self, query: &str) {
        if self.active {
            println!("  [conn {}] execute: {}", self.id, query);
        }
    }
}

impl Drop for ManagedConnection {
    fn drop(&mut self) {
        if self.active {
            println!("  [conn {}] closing connection to {}", self.id, self.host);
            self.active = false;
            // In real code: flush buffers, send FIN, release OS resources
        }
    }
}

fn drop_demo() {
    println!("\n=== Drop: Custom Cleanup ===");

    {
        let conn = ManagedConnection::new(1, "db.internal:5432");
        conn.execute("SELECT 1");
        // conn dropped here — Drop runs automatically
    }
    println!("  (scope ended — connection was closed above)");

    // std::mem::drop for explicit early cleanup
    let conn2 = ManagedConnection::new(2, "replica.internal:5432");
    conn2.execute("SELECT * FROM users LIMIT 10");
    std::mem::drop(conn2);   // explicit early drop
    println!("  (conn2 explicitly closed)");

    // Drop order: LIFO — last declared, first dropped
    let conn_a = ManagedConnection::new(10, "host-a");
    let conn_b = ManagedConnection::new(11, "host-b");
    let conn_c = ManagedConnection::new(12, "host-c");
    conn_a.execute("query a");
    conn_b.execute("query b");
    conn_c.execute("query c");
    // End of scope: conn_c dropped first, then conn_b, then conn_a
}

fn main() {
    arc_readonly_demo();
    arc_mutex_demo();
    cow_demo();
    deref_demo();
    drop_demo();
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_shared_across_threads() {
        let data = Arc::new(vec![1, 2, 3]);
        let d2 = Arc::clone(&data);
        let handle = thread::spawn(move || {
            assert_eq!(d2.len(), 3);
        });
        handle.join().unwrap();
        assert_eq!(data.len(), 3);  // still valid in main thread
    }

    #[test]
    fn test_arc_mutex_concurrent_increment() {
        let counter = Arc::new(Mutex::new(0u64));
        let handles: Vec<_> = (0..10).map(|_| {
            let c = Arc::clone(&counter);
            thread::spawn(move || {
                *c.lock().unwrap() += 1;
            })
        }).collect();
        for h in handles { h.join().unwrap(); }
        assert_eq!(*counter.lock().unwrap(), 10);
    }

    #[test]
    fn test_cow_no_allocation_when_clean() {
        let header = normalize_header_name("content-type");
        assert!(matches!(header, Cow::Borrowed(_)), "expected no allocation");
    }

    #[test]
    fn test_cow_allocates_when_needed() {
        let header = normalize_header_name("Content-Type");
        assert!(matches!(header, Cow::Owned(_)), "expected allocation");
        assert_eq!(header.as_ref(), "content-type");
    }

    #[test]
    fn test_sanitize_path_clean() {
        let result = sanitize_path("/api/users");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_sanitize_path_dirty() {
        let result = sanitize_path("/api//users");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result.as_ref(), "/api/users");
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2);
        assert!(limiter.check_and_record("client-1"));
        assert!(limiter.check_and_record("client-1"));
        assert!(!limiter.check_and_record("client-1"));  // blocked at limit
        assert!(limiter.check_and_record("client-2"));   // different client, independent
    }
}
