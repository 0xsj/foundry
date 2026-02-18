// Enums — Rust
//
// Demonstrates: unit variants, tuple variants, struct variants,
// Option<T>, Result<T,E>, pattern matching, nested enums,
// if let / while let, match guards, exhaustiveness.
//
// Run: rustc enums.rs && ./enums

// ---- Basic Enum ----

// C-like: no associated data. Declaration order determines PartialOrd.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRIT"),
        }
    }
}

// ---- Enums with Data ----

// Each variant carries different data:
// - unit (no data)
// - tuple (positional fields)
// - struct (named fields)
#[derive(Debug)]
enum DeploymentEvent {
    // Unit variant — the event itself is the whole message
    BuildStarted,

    // Tuple variant — one piece of associated data
    BuildCompleted(String),       // image tag

    // Tuple variant — multiple fields
    TestFailed(String, u32),      // test name, failure count

    // Struct variant — named fields for clarity
    DeploymentRolledOut {
        service: String,
        version: String,
        replicas: u32,
        region: String,
    },

    // Nested enum: events can have a severity attached
    Alert {
        message: String,
        severity: Severity,
    },
}

fn describe_event(event: &DeploymentEvent) {
    match event {
        DeploymentEvent::BuildStarted => {
            println!("build started");
        }

        // Tuple variant: bind the tag
        DeploymentEvent::BuildCompleted(tag) => {
            println!("build completed: image={}", tag);
        }

        // Tuple variant: bind both fields
        DeploymentEvent::TestFailed(test, count) => {
            println!("test '{}' failed {} times", test, count);
        }

        // Struct variant: destructure named fields
        DeploymentEvent::DeploymentRolledOut { service, version, replicas, region } => {
            println!(
                "deployed {service} v{version} x{replicas} in {region}"
            );
        }

        // Match guard: conditional match arm
        DeploymentEvent::Alert { severity, message } if *severity >= Severity::Error => {
            println!("[URGENT] {}: {}", severity, message);
        }

        DeploymentEvent::Alert { severity, message } => {
            println!("[{}] {}", severity, message);
        }
    }
}

// ---- Option<T> ----

// Find a service's primary endpoint. Returns None if not registered.
fn primary_endpoint(service: &str) -> Option<String> {
    match service {
        "auth" => Some(String::from("auth.internal:8081")),
        "api" => Some(String::from("api.internal:8080")),
        _ => None,
    }
}

fn option_demo() {
    println!("=== Option<T> ===");

    // Pattern 1: match (explicit, handles both cases)
    match primary_endpoint("api") {
        Some(addr) => println!("api endpoint: {}", addr),
        None => println!("api not registered"),
    }

    // Pattern 2: if let (concise when you only care about Some)
    if let Some(addr) = primary_endpoint("auth") {
        println!("auth endpoint: {}", addr);
    }

    // Pattern 3: unwrap_or (provide a fallback)
    let addr = primary_endpoint("metrics").unwrap_or(String::from("localhost:9090"));
    println!("metrics endpoint: {}", addr);

    // Pattern 4: map (transform the inner value without touching None)
    let upper = primary_endpoint("api").map(|a| a.to_uppercase());
    println!("uppercased: {:?}", upper);

    // Pattern 5: and_then (chain operations that might fail)
    let port: Option<u16> = primary_endpoint("auth")
        .and_then(|addr| addr.split(':').last()?.parse::<u16>().ok());
    println!("auth port: {:?}", port);

    // Chaining with ? inside a function:
    fn get_port(service: &str) -> Option<u16> {
        let addr = primary_endpoint(service)?;  // returns None if service unknown
        let port_str = addr.split(':').last()?; // returns None if no ':'
        port_str.parse::<u16>().ok()            // returns None if not a number
    }
    println!("auth port via ?: {:?}", get_port("auth"));
    println!("unknown port via ?: {:?}", get_port("unknown"));
}

// ---- Result<T, E> ----

#[derive(Debug)]
enum ConfigError {
    MissingField(String),
    InvalidValue { field: String, value: String, reason: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingField(name) => write!(f, "missing required field '{}'", name),
            ConfigError::InvalidValue { field, value, reason } => {
                write!(f, "invalid value '{}' for field '{}': {}", value, field, reason)
            }
        }
    }
}

fn parse_port(s: &str) -> Result<u16, ConfigError> {
    let n: u32 = s.parse().map_err(|_| {
        ConfigError::InvalidValue {
            field: String::from("port"),
            value: s.to_string(),
            reason: String::from("must be a number"),
        }
    })?;

    if n > 65535 {
        return Err(ConfigError::InvalidValue {
            field: String::from("port"),
            value: s.to_string(),
            reason: format!("{} exceeds maximum port (65535)", n),
        });
    }

    Ok(n as u16)
}

fn parse_config(port_str: &str, host: Option<&str>) -> Result<(String, u16), ConfigError> {
    let host = host.ok_or_else(|| ConfigError::MissingField(String::from("host")))?;
    let port = parse_port(port_str)?;  // ? propagates the error up
    Ok((host.to_string(), port))
}

fn result_demo() {
    println!("\n=== Result<T, E> ===");

    // Match on Result:
    match parse_port("8080") {
        Ok(p) => println!("port: {}", p),
        Err(e) => println!("error: {}", e),
    }

    match parse_port("99999") {
        Ok(p) => println!("port: {}", p),
        Err(e) => println!("error: {}", e),
    }

    match parse_port("abc") {
        Ok(p) => println!("port: {}", p),
        Err(e) => println!("error: {}", e),
    }

    // ? propagation:
    match parse_config("443", Some("api.internal")) {
        Ok((host, port)) => println!("config ok: {}:{}", host, port),
        Err(e) => println!("config error: {}", e),
    }

    match parse_config("443", None) {
        Ok((host, port)) => println!("config ok: {}:{}", host, port),
        Err(e) => println!("config error: {}", e),
    }

    // map / map_err / unwrap_or_else:
    let double_port = parse_port("3000").map(|p| p * 2);
    println!("doubled: {:?}", double_port);

    let result_str = parse_port("xyz").map_err(|e| e.to_string());
    println!("as string err: {:?}", result_str);
}

// ---- Nested Enums ----

#[derive(Debug)]
enum Transport {
    Http { host: String, port: u16, tls: bool },
    Grpc { target: String },
    Unix(String),  // socket path
}

#[derive(Debug)]
enum RetryPolicy {
    NoRetry,
    Fixed { attempts: u32, delay_ms: u64 },
    Exponential { max_attempts: u32, base_delay_ms: u64, max_delay_ms: u64 },
}

#[derive(Debug)]
struct ServiceConfig {
    name: String,
    transport: Transport,
    retry: RetryPolicy,
}

fn describe_config(cfg: &ServiceConfig) {
    print!("{}: transport=", cfg.name);

    match &cfg.transport {
        Transport::Http { host, port, tls } => {
            let scheme = if *tls { "https" } else { "http" };
            print!("{}://{}:{}", scheme, host, port);
        }
        Transport::Grpc { target } => print!("grpc://{}", target),
        Transport::Unix(path) => print!("unix:{}", path),
    }

    print!(", retry=");

    match &cfg.retry {
        RetryPolicy::NoRetry => print!("none"),
        RetryPolicy::Fixed { attempts, delay_ms } => {
            print!("fixed({} x {}ms)", attempts, delay_ms);
        }
        RetryPolicy::Exponential { max_attempts, base_delay_ms, max_delay_ms } => {
            print!("exp({} max, {}..{}ms)", max_attempts, base_delay_ms, max_delay_ms);
        }
    }

    println!();
}

// ---- while let and exhaustiveness ----

fn event_loop_demo() {
    println!("\n=== Event Loop (while let) ===");

    let mut events = vec![
        Some(DeploymentEvent::BuildStarted),
        Some(DeploymentEvent::BuildCompleted(String::from("sha256:abc123"))),
        None,  // represents a gap / no-op
        Some(DeploymentEvent::Alert {
            message: String::from("memory pressure at 90%"),
            severity: Severity::Warning,
        }),
    ];

    // while let: continue looping as long as pop() returns Some
    while let Some(maybe_event) = events.pop() {
        if let Some(event) = maybe_event {
            describe_event(&event);
        } else {
            println!("(no-op slot)");
        }
    }
}

fn main() {
    // Basic enum + match
    println!("=== Severity Ordering ===");
    let s = Severity::Warning;
    println!("{} >= Error? {}", s, s >= Severity::Error);
    println!("{} >= Info? {}", s, s >= Severity::Info);

    // Enum with data
    println!("\n=== Deployment Events ===");
    let events = vec![
        DeploymentEvent::BuildStarted,
        DeploymentEvent::BuildCompleted(String::from("v1.4.2")),
        DeploymentEvent::TestFailed(String::from("test_checkout_flow"), 3),
        DeploymentEvent::DeploymentRolledOut {
            service: String::from("checkout"),
            version: String::from("1.4.2"),
            replicas: 6,
            region: String::from("us-east-1"),
        },
        DeploymentEvent::Alert {
            message: String::from("CPU spike during rollout"),
            severity: Severity::Critical,
        },
        DeploymentEvent::Alert {
            message: String::from("deprecated API called"),
            severity: Severity::Warning,
        },
    ];

    for event in &events {
        describe_event(event);
    }

    // Option
    option_demo();

    // Result
    result_demo();

    // Nested enums
    println!("\n=== Service Configs ===");
    let configs = vec![
        ServiceConfig {
            name: String::from("auth-service"),
            transport: Transport::Http {
                host: String::from("auth.internal"),
                port: 443,
                tls: true,
            },
            retry: RetryPolicy::Exponential {
                max_attempts: 3,
                base_delay_ms: 100,
                max_delay_ms: 5000,
            },
        },
        ServiceConfig {
            name: String::from("metrics-collector"),
            transport: Transport::Unix(String::from("/run/metrics.sock")),
            retry: RetryPolicy::NoRetry,
        },
        ServiceConfig {
            name: String::from("grpc-gateway"),
            transport: Transport::Grpc {
                target: String::from("grpc.internal:50051"),
            },
            retry: RetryPolicy::Fixed { attempts: 2, delay_ms: 500 },
        },
    ];

    for cfg in &configs {
        describe_config(cfg);
    }

    // while let demo
    event_loop_demo();
}
