// HashMap<K,V> — Operations, entry API, counting patterns, BTreeMap, HashSet
//
// Run: rustc hashmaps.rs && ./hashmaps

use std::collections::{BTreeMap, HashMap, HashSet};

fn main() {
    // -------------------------------------------------------------------------
    // Basic operations
    // -------------------------------------------------------------------------

    println!("=== Basic HashMap Operations ===");
    let mut config: HashMap<String, String> = HashMap::new();

    config.insert(String::from("host"), String::from("0.0.0.0"));
    config.insert(String::from("port"), String::from("8080"));
    config.insert(String::from("log_level"), String::from("info"));

    // get() uses the Borrow trait — &str works for String keys
    if let Some(host) = config.get("host") {
        println!("host: {host}");
    }

    // insert() returns the old value if the key existed
    let old = config.insert(String::from("port"), String::from("9090"));
    println!("replaced port: old={:?}, new={}", old, config["port"]);

    // contains_key, remove
    println!("has 'host': {}", config.contains_key("host"));
    let removed = config.remove("log_level");
    println!("removed log_level: {:?}", removed);
    println!("remaining keys: {}", config.len());

    // -------------------------------------------------------------------------
    // Entry API — the idiomatic way to insert-or-update
    // -------------------------------------------------------------------------

    println!("\n=== Entry API ===");

    // Scenario: counting HTTP status codes from an access log
    let status_codes = vec![200, 200, 404, 500, 200, 404, 200, 503, 500];

    let mut counts: HashMap<u16, u32> = HashMap::new();
    for code in &status_codes {
        // or_insert(0) inserts 0 if absent, returns &mut u32 either way
        let count = counts.entry(*code).or_insert(0);
        *count += 1;
    }

    // Collect and sort for consistent output
    let mut sorted_counts: Vec<(u16, u32)> = counts.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_counts.sort_by_key(|&(code, _)| code);
    println!("status code counts: {:?}", sorted_counts);

    // and_modify + or_insert: update if exists, set if not
    let mut route_hits: HashMap<&str, u64> = HashMap::new();
    let routes = vec!["/api/users", "/api/items", "/api/users", "/health"];
    for route in &routes {
        route_hits
            .entry(route)
            .and_modify(|hits| *hits += 1)
            .or_insert(1);
    }
    println!("route hits: {:?}", route_hits);

    // or_insert_with: expensive default, only computed when key is absent
    let mut cache: HashMap<&str, Vec<u8>> = HashMap::new();
    let data = cache.entry("config.json").or_insert_with(|| {
        // In real code this would read from disk — only called once
        vec![0x7b, 0x7d]  // "{}" as bytes
    });
    println!("cached data length: {}", data.len());

    // or_default: insert V::default() if absent (requires V: Default)
    let mut error_counts: HashMap<&str, u32> = HashMap::new();
    *error_counts.entry("timeout").or_default() += 5;
    println!("timeout errors: {}", error_counts["timeout"]);

    // -------------------------------------------------------------------------
    // Grouping with entry API
    // -------------------------------------------------------------------------

    println!("\n=== Grouping with Entry ===");

    // Group log messages by source service
    let logs: Vec<(&str, &str)> = vec![
        ("auth-service", "login attempt"),
        ("api-gateway", "request received"),
        ("auth-service", "login success"),
        ("db-proxy", "query executed"),
        ("api-gateway", "response sent"),
        ("auth-service", "token issued"),
    ];

    let mut by_service: HashMap<&str, Vec<&str>> = HashMap::new();
    for (service, message) in &logs {
        by_service.entry(service).or_insert_with(Vec::new).push(message);
    }

    let mut services: Vec<&&str> = by_service.keys().collect();
    services.sort();
    for service in services {
        println!("{}: {} messages", service, by_service[service].len());
    }

    // -------------------------------------------------------------------------
    // Iteration patterns
    // -------------------------------------------------------------------------

    println!("\n=== Iteration ===");

    let mut metrics: HashMap<String, f64> = HashMap::new();
    metrics.insert(String::from("cpu_percent"), 45.2);
    metrics.insert(String::from("memory_gb"), 2.1);
    metrics.insert(String::from("latency_ms"), 12.5);

    // Immutable — borrows the map
    println!("metrics (unordered):");
    for (key, value) in &metrics {
        println!("  {}: {:.1}", key, value);
    }

    // Mutable — scale all values
    for (_, value) in &mut metrics {
        *value *= 1.1;  // apply a 10% overhead factor
    }

    // Consuming — moves the map, can't use `metrics` after this
    let total: f64 = metrics.into_values().sum();
    println!("sum of all metrics: {:.2}", total);

    // -------------------------------------------------------------------------
    // BTreeMap: sorted iteration, range queries
    // -------------------------------------------------------------------------

    println!("\n=== BTreeMap (sorted) ===");

    let mut error_log: BTreeMap<u64, &str> = BTreeMap::new();
    error_log.insert(1708300100, "disk full on /var/log");
    error_log.insert(1708300050, "connection pool exhausted");
    error_log.insert(1708300200, "health check failed");
    error_log.insert(1708300080, "memory pressure");

    // Iteration is always sorted by key (timestamp order here)
    println!("errors in timestamp order:");
    for (ts, msg) in &error_log {
        println!("  {ts}: {msg}");
    }

    // Range query: events between two timestamps
    println!("\nevents between t=1708300060 and t=1708300150:");
    for (ts, msg) in error_log.range(1708300060..=1708300150) {
        println!("  {ts}: {msg}");
    }

    println!("first event: {:?}", error_log.first_key_value());
    println!("last event: {:?}", error_log.last_key_value());

    // -------------------------------------------------------------------------
    // HashSet: membership testing and set operations
    // -------------------------------------------------------------------------

    println!("\n=== HashSet ===");

    let allowed_origins: HashSet<&str> = [
        "https://app.example.com",
        "https://api.example.com",
        "https://admin.example.com",
    ].iter().copied().collect();

    let request_origin = "https://evil.example.org";
    println!("is origin allowed: {}", allowed_origins.contains(request_origin));
    println!("is app allowed: {}", allowed_origins.contains("https://app.example.com"));

    // Set operations — finding which features are enabled/disabled
    let deployed_features: HashSet<&str> = ["dark-mode", "new-checkout", "beta-api"].iter().copied().collect();
    let requested_features: HashSet<&str> = ["dark-mode", "ai-assistant", "beta-api", "streaming"].iter().copied().collect();

    let available: HashSet<_> = deployed_features.intersection(&requested_features).collect();
    let unavailable: HashSet<_> = requested_features.difference(&deployed_features).collect();

    let mut avail_sorted: Vec<_> = available.iter().collect();
    avail_sorted.sort();
    let mut unavail_sorted: Vec<_> = unavailable.iter().collect();
    unavail_sorted.sort();

    println!("available requested features: {:?}", avail_sorted);
    println!("unavailable features: {:?}", unavail_sorted);
}
