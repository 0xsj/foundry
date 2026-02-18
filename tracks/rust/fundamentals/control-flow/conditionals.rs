// Control Flow: Conditionals — Rust
//
// Demonstrates: if/else as expressions, if let, let-else, nested conditions.
// Run with: rustc conditionals.rs && ./conditionals

// ---------- if/else as expressions ----------

fn classify_http_status(code: u16) -> &'static str {
    // if/else returns a value — no ternary needed.
    // All branches must be the same type (&'static str here).
    if code >= 500 {
        "server error"
    } else if code >= 400 {
        "client error"
    } else if code >= 300 {
        "redirect"
    } else if code >= 200 {
        "success"
    } else {
        "informational"
    }
}

fn retry_delay_ms(attempt: u32) -> u64 {
    // Single-expression if/else — equivalent to a ternary in JS/Go.
    // TypeScript: attempt < 5 ? 100 * 2u64.pow(attempt) : 30_000
    let base: u64 = if attempt < 5 { 100 } else { 30_000 };
    base * 2u64.pow(attempt.min(5))
}

// ---------- if let ----------

#[derive(Debug)]
struct HealthCheck {
    service: &'static str,
    latency_ms: Option<u64>,
    error: Option<&'static str>,
}

fn report_health(check: &HealthCheck) {
    // if let extracts the Some value without a full match.
    // Equivalent to: if (check.latency_ms !== undefined) { ... } in TypeScript.
    if let Some(ms) = check.latency_ms {
        if ms > 500 {
            println!("[SLOW] {} responded in {}ms", check.service, ms);
        } else {
            println!("[OK]   {} responded in {}ms", check.service, ms);
        }
    }

    // if let with else — handle both Some and None meaningfully.
    if let Some(err) = check.error {
        println!("[ERR]  {} failed: {}", check.service, err);
    } else {
        println!("[PASS] {} has no errors", check.service);
    }
}

// ---------- let-else (Rust 1.65+) ----------

// Simulate a config value that may or may not exist.
fn get_env(key: &str) -> Option<&'static str> {
    match key {
        "HOST" => Some("0.0.0.0"),
        "PORT" => Some("8080"),
        _ => None,
    }
}

fn parse_port(s: &str) -> Result<u16, String> {
    s.parse::<u16>().map_err(|e| format!("invalid port '{}': {}", s, e))
}

fn build_bind_address() -> String {
    // let-else: extract the value or bail out early.
    // The else block must diverge (return/panic/break).
    // This replaces the Go pattern:
    //   host, ok := os.LookupEnv("HOST")
    //   if !ok { log.Fatal("HOST not set") }
    let Some(host) = get_env("HOST") else {
        return String::from("HOST not configured — using fallback");
    };

    let Some(port_str) = get_env("PORT") else {
        return format!("{}:80", host); // fallback port
    };

    let Ok(port) = parse_port(port_str) else {
        return format!("{}:80", host); // invalid port, fallback
    };

    format!("{}:{}", host, port)
}

// ---------- Nested conditions ----------

#[derive(Debug)]
enum AuthResult {
    Allowed,
    Denied(&'static str),
}

fn check_access(user_role: &str, endpoint: &str, method: &str) -> AuthResult {
    // Nested if/else with complex conditions.
    // A real system would use a policy engine, but this shows the pattern.
    if user_role == "admin" {
        // Admins can do anything
        AuthResult::Allowed
    } else if endpoint.starts_with("/internal") {
        // Internal endpoints are admin-only
        AuthResult::Denied("internal endpoints require admin role")
    } else if method == "DELETE" && user_role != "owner" {
        AuthResult::Denied("DELETE requires owner role")
    } else if user_role == "readonly" && method != "GET" {
        AuthResult::Denied("readonly role can only perform GET")
    } else {
        AuthResult::Allowed
    }
}

// ---------- main ----------

fn main() {
    // -- if/else as expression --
    println!("=== HTTP Status Classification ===");
    for code in [200u16, 201, 301, 404, 500, 503] {
        println!("  {} -> {}", code, classify_http_status(code));
    }

    println!("\n=== Retry Delays ===");
    for attempt in 0..7 {
        println!("  attempt {} -> {}ms", attempt, retry_delay_ms(attempt));
    }

    // -- if let --
    println!("\n=== Health Checks ===");
    let checks = vec![
        HealthCheck { service: "api",      latency_ms: Some(42),   error: None },
        HealthCheck { service: "db",       latency_ms: Some(750),  error: None },
        HealthCheck { service: "cache",    latency_ms: None,       error: Some("connection refused") },
        HealthCheck { service: "metrics",  latency_ms: Some(12),   error: None },
    ];
    for check in &checks {
        report_health(check);
    }

    // -- let-else --
    println!("\n=== Bind Address ===");
    println!("  {}", build_bind_address());

    // -- Nested conditions --
    println!("\n=== Access Control ===");
    let cases = [
        ("admin",    "/internal/metrics", "GET"),
        ("user",     "/internal/metrics", "GET"),
        ("user",     "/api/posts",        "DELETE"),
        ("owner",    "/api/posts",        "DELETE"),
        ("readonly", "/api/posts",        "POST"),
        ("readonly", "/api/posts",        "GET"),
    ];
    for (role, endpoint, method) in cases {
        match check_access(role, endpoint, method) {
            AuthResult::Allowed => {
                println!("  ALLOW  {} {} {} ", role, method, endpoint);
            }
            AuthResult::Denied(reason) => {
                println!("  DENY   {} {} {} — {}", role, method, endpoint, reason);
            }
        }
    }
}
