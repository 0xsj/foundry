// Control Flow: Pattern Matching — Rust
//
// Demonstrates: match with literals, ranges, enum variants, destructuring,
// guards, @ bindings, nested patterns, if let, while let.
// Run with: rustc matching.rs && ./matching

// ---------- Basic match ----------

fn describe_port(port: u16) -> &'static str {
    // Literal and range patterns. ..= is inclusive range in patterns.
    // Patterns are tried top-to-bottom; first match wins.
    match port {
        0 => "reserved",
        1..=1023 => "well-known",
        1024..=49151 => "registered",
        49152..=65535 => "ephemeral",
    }
    // No `_` needed — the ranges cover all u16 values (0..=65535).
    // The compiler verifies exhaustiveness.
}

// ---------- Enum matching ----------

#[derive(Debug)]
enum DatabaseError {
    ConnectionFailed { host: String, port: u16 },
    QueryTimeout { query_id: u64, duration_ms: u64 },
    ConstraintViolation { table: String, constraint: String },
    NotFound,
    Unknown(String),
}

fn handle_db_error(err: &DatabaseError) -> (bool, &'static str) {
    // (retryable, message)
    match err {
        // Struct variant destructuring
        DatabaseError::ConnectionFailed { host, port } => {
            eprintln!("  db: cannot connect to {}:{}", host, port);
            (true, "connection failed")
        }

        // Guard: only retry if within reason
        DatabaseError::QueryTimeout { duration_ms, .. } if *duration_ms < 30_000 => {
            (true, "query timed out (retryable)")
        }
        DatabaseError::QueryTimeout { query_id, duration_ms } => {
            eprintln!("  db: query {} exceeded {}ms — aborting", query_id, duration_ms);
            (false, "query timed out (permanent)")
        }

        // Or-pattern: two variants with the same action
        DatabaseError::ConstraintViolation { table, constraint } => {
            eprintln!("  db: constraint '{}' violated on '{}'", constraint, table);
            (false, "constraint violation")
        }

        DatabaseError::NotFound => (false, "not found"),

        // Catch-all with binding
        DatabaseError::Unknown(msg) => {
            eprintln!("  db: unknown error: {}", msg);
            (false, "unknown error")
        }
    }
}

// ---------- Tuple matching ----------

#[derive(Debug, PartialEq)]
enum Role { Admin, Owner, User, Guest }

#[derive(Debug, PartialEq)]
enum Action { Read, Write, Delete, Admin }

fn can_perform(role: &Role, action: &Action) -> bool {
    // Match on a tuple — test two values simultaneously.
    // Much cleaner than nested if/else.
    match (role, action) {
        (Role::Admin, _) => true,                        // admins can do anything
        (Role::Owner, Action::Admin) => false,            // owners can't admin
        (Role::Owner, _) => true,                        // owners can do the rest
        (Role::User, Action::Read | Action::Write) => true, // or-pattern in tuple
        (Role::Guest, Action::Read) => true,
        _ => false,
    }
}

// ---------- @ bindings ----------

#[derive(Debug)]
struct RateLimitConfig {
    requests_per_second: u32,
    #[allow(dead_code)]
    burst_size: u32,
}

fn classify_rate_limit(config: &RateLimitConfig) -> &'static str {
    // @ binds the matched value so you can both test it and use it.
    match config.requests_per_second {
        0 => "disabled",
        rps @ 1..=10 => {
            // rps is bound here — we can use it
            if rps < 5 { "very restrictive" } else { "restrictive" }
        }
        rps @ 11..=1000 => {
            let _ = rps; // bound but not used in this branch
            "standard"
        }
        _ => "permissive",
    }
}

// ---------- Nested patterns ----------

#[derive(Debug)]
struct TlsConfig {
    enabled: bool,
    cert_path: Option<String>,
    min_version: Option<&'static str>,
}

#[derive(Debug)]
struct ServerConfig {
    host: String,
    port: u16,
    tls: Option<TlsConfig>,
}

fn describe_server(config: &ServerConfig) -> String {
    // Nested pattern matching on embedded structs and Options.
    match config {
        // TLS with cert — explicit path + version
        ServerConfig {
            tls: Some(TlsConfig { enabled: true, cert_path: Some(path), min_version: Some(ver) }),
            host,
            port,
            ..
        } => {
            format!("https://{}:{} (TLS {}, cert: {})", host, port, ver, path)
        }

        // TLS enabled but missing details
        ServerConfig {
            tls: Some(TlsConfig { enabled: true, .. }),
            host,
            port,
            ..
        } => {
            format!("https://{}:{} (TLS, default config)", host, port)
        }

        // TLS explicitly disabled
        ServerConfig {
            tls: Some(TlsConfig { enabled: false, .. }) | None,
            host,
            port,
            ..
        } => {
            format!("http://{}:{}", host, port)
        }
    }
}

// ---------- if let patterns ----------

#[derive(Debug)]
struct CacheEntry {
    value: String,
    ttl_remaining: Option<u64>,
    hit_count: u64,
}

fn cache_status(entry: &CacheEntry) -> String {
    // if let for simple single-branch matching.
    // Use when you only care about one variant.
    if let Some(ttl) = entry.ttl_remaining {
        if ttl == 0 {
            return format!("EXPIRED: {} (hits: {})", entry.value, entry.hit_count);
        }
        format!("LIVE: {} (ttl: {}s, hits: {})", entry.value, ttl, entry.hit_count)
    } else {
        format!("PERMANENT: {} (hits: {})", entry.value, entry.hit_count)
    }
}

// ---------- while let ----------

struct TokenStream {
    tokens: Vec<String>,
}

impl TokenStream {
    fn next_token(&mut self) -> Option<String> {
        if self.tokens.is_empty() {
            None
        } else {
            Some(self.tokens.remove(0))
        }
    }
}

fn tokenize_and_parse(stream: &mut TokenStream) -> Vec<String> {
    let mut parsed = Vec::new();

    // while let: process tokens until the stream is exhausted
    while let Some(token) = stream.next_token() {
        // Skip comments and whitespace tokens
        if token.starts_with("//") || token.trim().is_empty() {
            continue;
        }
        parsed.push(token);
    }
    parsed
}

// ---------- match arm ordering ----------

fn priority_label(score: i32) -> &'static str {
    // Order matters: more specific cases first, general cases last.
    // The compiler warns about unreachable arms if you put _ first.
    match score {
        i32::MIN..=-1 => "invalid",   // negative scores are invalid
        0 => "unset",
        1..=3 => "low",
        4..=6 => "medium",
        7..=9 => "high",
        10 => "critical",
        _ => "out of range",           // catches anything > 10
    }
}

// ---------- main ----------

fn main() {
    // -- Basic match --
    println!("=== Port Classification ===");
    for port in [0u16, 80, 443, 8080, 50000, 65535] {
        println!("  port {} -> {}", port, describe_port(port));
    }

    // -- Enum matching --
    println!("\n=== Database Errors ===");
    let errors = vec![
        DatabaseError::ConnectionFailed { host: "10.0.0.5".to_string(), port: 5432 },
        DatabaseError::QueryTimeout { query_id: 1001, duration_ms: 5_000 },
        DatabaseError::QueryTimeout { query_id: 1002, duration_ms: 45_000 },
        DatabaseError::ConstraintViolation { table: "users".to_string(), constraint: "email_unique".to_string() },
        DatabaseError::NotFound,
        DatabaseError::Unknown("disk full".to_string()),
    ];
    for err in &errors {
        let (retryable, msg) = handle_db_error(err);
        println!("  {:?} -> retryable={} msg={}", err, retryable, msg);
    }

    // -- Tuple matching --
    println!("\n=== Permissions ===");
    let cases = [
        (&Role::Admin, &Action::Delete),
        (&Role::Owner, &Action::Admin),
        (&Role::Owner, &Action::Write),
        (&Role::User, &Action::Write),
        (&Role::User, &Action::Delete),
        (&Role::Guest, &Action::Read),
        (&Role::Guest, &Action::Write),
    ];
    for (role, action) in cases {
        println!("  {:?} {:?} -> {}", role, action,
            if can_perform(role, action) { "allowed" } else { "denied" });
    }

    // -- @ bindings --
    println!("\n=== Rate Limit Classification ===");
    for rps in [0u32, 3, 8, 100, 2000] {
        let cfg = RateLimitConfig { requests_per_second: rps, burst_size: rps * 2 };
        println!("  {} rps -> {}", rps, classify_rate_limit(&cfg));
    }

    // -- Nested patterns --
    println!("\n=== Server Configuration ===");
    let configs = vec![
        ServerConfig {
            host: "api.example.com".to_string(),
            port: 443,
            tls: Some(TlsConfig {
                enabled: true,
                cert_path: Some("/etc/certs/api.pem".to_string()),
                min_version: Some("TLS 1.3"),
            }),
        },
        ServerConfig {
            host: "internal.example.com".to_string(),
            port: 8443,
            tls: Some(TlsConfig { enabled: true, cert_path: None, min_version: None }),
        },
        ServerConfig {
            host: "localhost".to_string(),
            port: 3000,
            tls: None,
        },
    ];
    for cfg in &configs {
        println!("  {}", describe_server(cfg));
    }

    // -- if let --
    println!("\n=== Cache Entries ===");
    let entries = vec![
        CacheEntry { value: "user:42".to_string(), ttl_remaining: Some(120), hit_count: 5 },
        CacheEntry { value: "user:99".to_string(), ttl_remaining: Some(0), hit_count: 2 },
        CacheEntry { value: "config:app".to_string(), ttl_remaining: None, hit_count: 100 },
    ];
    for entry in &entries {
        println!("  {}", cache_status(entry));
    }

    // -- while let --
    println!("\n=== Token Stream ===");
    let mut stream = TokenStream {
        tokens: vec![
            "let".to_string(),
            "x".to_string(),
            "=".to_string(),
            "// this is a comment".to_string(),
            "42".to_string(),
            ";".to_string(),
        ],
    };
    let parsed = tokenize_and_parse(&mut stream);
    println!("  parsed tokens: {:?}", parsed);

    // -- Match arm ordering --
    println!("\n=== Priority Labels ===");
    for score in [-5, 0, 2, 5, 8, 10, 15] {
        println!("  {} -> {}", score, priority_label(score));
    }
}
