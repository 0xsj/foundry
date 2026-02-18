// Data Processing Pipeline: Lazy Iterator Chains
//
// Demonstrates: Building lazy pipelines with iterator adapters, zero-cost
// abstraction (iterator chain vs hand-written loop), real-world log processing.
//
// Scenario: Process structured log lines from a server access log.
// Parse each line, filter by status code, extract metrics, aggregate.
//
// Run: rustc pipeline.rs && ./pipeline

use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct AccessLogEntry {
    method: String,
    path: String,
    status: u16,
    response_time_ms: u32,
    bytes_sent: u64,
}

impl fmt::Display for AccessLogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}ms {}B",
            self.method, self.path, self.status, self.response_time_ms, self.bytes_sent
        )
    }
}

/// Parse a log line in format: "METHOD /path STATUS TIME_MS BYTES"
fn parse_log_line(line: &str) -> Option<AccessLogEntry> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }
    Some(AccessLogEntry {
        method: parts[0].to_string(),
        path: parts[1].to_string(),
        status: parts[2].parse().ok()?,
        response_time_ms: parts[3].parse().ok()?,
        bytes_sent: parts[4].parse().ok()?,
    })
}

// ---------------------------------------------------------------------------
// Pipeline approach: lazy iterator chain
// ---------------------------------------------------------------------------

struct PipelineResult {
    error_count: usize,
    slow_requests: Vec<AccessLogEntry>,
    bytes_by_path: HashMap<String, u64>,
    avg_response_time_ms: f64,
}

fn analyze_with_pipeline(raw_log: &str) -> PipelineResult {
    // Step 1: Parse all valid entries lazily.
    // filter_map combines filter (skip None) + map (unwrap Some).
    // Nothing executes yet — this just builds an iterator type.
    let entries: Vec<AccessLogEntry> = raw_log
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(parse_log_line)
        .collect();

    // Step 2: Count errors using iterator — no intermediate collection
    let error_count = entries.iter().filter(|e| e.status >= 500).count();

    // Step 3: Find slow requests (>200ms), sorted by response time descending
    let mut slow_requests: Vec<AccessLogEntry> = entries
        .iter()
        .filter(|e| e.response_time_ms > 200)
        .cloned()
        .collect();
    slow_requests.sort_by(|a, b| b.response_time_ms.cmp(&a.response_time_ms));

    // Step 4: Aggregate bytes by path using fold
    let bytes_by_path: HashMap<String, u64> =
        entries.iter().fold(HashMap::new(), |mut acc, entry| {
            *acc.entry(entry.path.clone()).or_insert(0) += entry.bytes_sent;
            acc
        });

    // Step 5: Average response time
    let total_ms: u64 = entries.iter().map(|e| e.response_time_ms as u64).sum();
    let avg_response_time_ms = if entries.is_empty() {
        0.0
    } else {
        total_ms as f64 / entries.len() as f64
    };

    PipelineResult {
        error_count,
        slow_requests,
        bytes_by_path,
        avg_response_time_ms,
    }
}

// ---------------------------------------------------------------------------
// Hand-written loop approach (for comparison)
// ---------------------------------------------------------------------------

fn analyze_with_loops(raw_log: &str) -> PipelineResult {
    let mut entries = Vec::new();

    // Parse
    for line in raw_log.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(entry) = parse_log_line(trimmed) {
            entries.push(entry);
        }
    }

    // Count errors
    let mut error_count = 0;
    for entry in &entries {
        if entry.status >= 500 {
            error_count += 1;
        }
    }

    // Slow requests
    let mut slow_requests = Vec::new();
    for entry in &entries {
        if entry.response_time_ms > 200 {
            slow_requests.push(entry.clone());
        }
    }
    slow_requests.sort_by(|a, b| b.response_time_ms.cmp(&a.response_time_ms));

    // Bytes by path
    let mut bytes_by_path: HashMap<String, u64> = HashMap::new();
    for entry in &entries {
        *bytes_by_path.entry(entry.path.clone()).or_insert(0) += entry.bytes_sent;
    }

    // Average response time
    let mut total_ms: u64 = 0;
    for entry in &entries {
        total_ms += entry.response_time_ms as u64;
    }
    let avg_response_time_ms = if entries.is_empty() {
        0.0
    } else {
        total_ms as f64 / entries.len() as f64
    };

    PipelineResult {
        error_count,
        slow_requests,
        bytes_by_path,
        avg_response_time_ms,
    }
}

// ---------------------------------------------------------------------------
// Demonstrate: chaining without intermediate collections
// ---------------------------------------------------------------------------

/// Find the top N paths by total bytes, using a single iterator chain.
fn top_paths_by_bytes(raw_log: &str, n: usize) -> Vec<(String, u64)> {
    // Build a HashMap, then sort — all in one expression
    let mut path_bytes: Vec<(String, u64)> = raw_log
        .lines()
        .filter_map(parse_log_line)
        .fold(HashMap::new(), |mut acc, entry| {
            *acc.entry(entry.path).or_insert(0) += entry.bytes_sent;
            acc
        })
        .into_iter()
        .collect();

    path_bytes.sort_by(|a, b| b.1.cmp(&a.1));
    path_bytes.truncate(n);
    path_bytes
}

/// Extract unique HTTP methods seen in error responses.
fn error_methods(raw_log: &str) -> Vec<String> {
    let mut methods: Vec<String> = raw_log
        .lines()
        .filter_map(parse_log_line)
        .filter(|e| e.status >= 400)
        .map(|e| e.method)
        .collect::<std::collections::HashSet<String>>()
        .into_iter()
        .collect();
    methods.sort();
    methods
}

/// Compute percentile response time using sorted iterator.
fn response_time_percentile(raw_log: &str, percentile: f64) -> Option<u32> {
    let mut times: Vec<u32> = raw_log
        .lines()
        .filter_map(parse_log_line)
        .map(|e| e.response_time_ms)
        .collect();

    if times.is_empty() {
        return None;
    }

    times.sort();
    let idx = ((percentile / 100.0) * (times.len() - 1) as f64).round() as usize;
    Some(times[idx])
}

// ---------------------------------------------------------------------------
// Generate sample log data
// ---------------------------------------------------------------------------

fn generate_log(num_entries: usize) -> String {
    let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
    let paths = [
        "/api/users",
        "/api/orders",
        "/api/products",
        "/api/auth/login",
        "/api/health",
        "/api/webhooks",
        "/api/metrics",
        "/static/bundle.js",
        "/static/styles.css",
    ];
    let statuses = [200u16, 200, 200, 200, 201, 204, 301, 400, 404, 500, 502, 503];

    let mut lines = Vec::with_capacity(num_entries);
    for i in 0..num_entries {
        let method = methods[i % methods.len()];
        let path = paths[i % paths.len()];
        let status = statuses[i % statuses.len()];
        // Vary response times: most fast, some slow
        let time_ms = if i % 17 == 0 {
            300 + (i % 500) as u32
        } else if i % 7 == 0 {
            150 + (i % 200) as u32
        } else {
            10 + (i % 50) as u32
        };
        let bytes = 100 + (i * 37 % 10000) as u64;

        lines.push(format!("{} {} {} {} {}", method, path, status, time_ms, bytes));
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    // Small example for readable output
    let small_log = "\
GET /api/users 200 45 1024
POST /api/orders 201 120 512
GET /api/users 200 15 2048
DELETE /api/orders 500 350 128
PUT /api/products 200 80 768
GET /api/health 200 5 64
POST /api/auth/login 401 200 256
GET /api/users 503 450 0
PATCH /api/users 200 60 512
GET /api/orders 200 30 4096";

    println!("=== Pipeline Analysis (small dataset) ===\n");

    let result = analyze_with_pipeline(small_log);
    println!("Total errors (5xx): {}", result.error_count);
    println!("Average response time: {:.1}ms", result.avg_response_time_ms);
    println!("\nSlow requests (>200ms):");
    for req in &result.slow_requests {
        println!("  {} — {}ms", req.path, req.response_time_ms);
    }
    println!("\nBytes by path:");
    let mut sorted_paths: Vec<_> = result.bytes_by_path.iter().collect();
    sorted_paths.sort_by(|a, b| b.1.cmp(a.1));
    for (path, bytes) in &sorted_paths {
        println!("  {}: {} bytes", path, bytes);
    }

    println!("\n=== Advanced Pipeline Queries ===\n");

    let top = top_paths_by_bytes(small_log, 3);
    println!("Top 3 paths by bytes:");
    for (path, bytes) in &top {
        println!("  {}: {} bytes", path, bytes);
    }

    let methods = error_methods(small_log);
    println!("\nHTTP methods in error responses: {:?}", methods);

    if let Some(p95) = response_time_percentile(small_log, 95.0) {
        println!("P95 response time: {}ms", p95);
    }

    // Benchmark: iterator pipeline vs hand-written loops
    println!("\n=== Performance Comparison ===\n");

    let large_log = generate_log(100_000);
    let iterations = 50;

    // Warm up
    let _ = analyze_with_pipeline(&large_log);
    let _ = analyze_with_loops(&large_log);

    // Pipeline approach
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = analyze_with_pipeline(&large_log);
    }
    let pipeline_time = start.elapsed();

    // Hand-written loop approach
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = analyze_with_loops(&large_log);
    }
    let loop_time = start.elapsed();

    println!("100,000 log entries x {} iterations:", iterations);
    println!(
        "  Iterator pipeline: {:.2}ms avg",
        pipeline_time.as_millis() as f64 / iterations as f64
    );
    println!(
        "  Hand-written loop: {:.2}ms avg",
        loop_time.as_millis() as f64 / iterations as f64
    );
    println!(
        "  Ratio: {:.2}x",
        pipeline_time.as_secs_f64() / loop_time.as_secs_f64()
    );
    println!("\n  (With optimizations, these should be nearly identical.)");
    println!("  Compile with: rustc -O pipeline.rs && ./pipeline");
}
