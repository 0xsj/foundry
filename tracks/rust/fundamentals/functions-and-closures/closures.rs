// Closures — Rust
//
// Demonstrates: closure syntax, type inference, capture modes (borrow / mut borrow / move),
// the Fn/FnMut/FnOnce distinction, move closures for ownership transfer, and
// closures in standard library methods.
//
// Run: rustc closures.rs && ./closures

// ---- Closure syntax ----

fn closure_syntax_examples() {
    println!("=== Closure Syntax ===\n");

    // Full syntax: explicit types, braced body
    let add = |a: i32, b: i32| -> i32 { a + b };

    // Inferred types, expression body (no braces needed for single expression)
    let double = |x| x * 2;

    // Inferred types, braced body
    let describe = |n: i32| {
        if n > 0 { "positive" }
        else if n < 0 { "negative" }
        else { "zero" }
    };

    // No parameters
    let greeting = || String::from("Hello from a closure");

    println!("add(3, 4) = {}", add(3, 4));
    println!("double(7) = {}", double(7));
    println!("describe(-5) = {}", describe(-5));
    println!("greeting() = {}", greeting());

    // Type inference is locked on first use:
    // Once double is used with i32, it's fixed to i32.
    // double(3.0); // ERROR: expected integer, found floating-point number
}

// ---- Capture by immutable borrow (&T) ----

fn capture_immutable() {
    println!("\n=== Capture: Immutable Borrow ===\n");

    let threshold = 500u32; // On the stack, but closure borrows it

    // Closure reads threshold — captures &threshold
    let is_slow = |ms: u32| ms >= threshold;

    // We can still use threshold — borrow doesn't prevent reading
    println!("threshold = {}", threshold);
    println!("300ms is slow: {}", is_slow(300));
    println!("700ms is slow: {}", is_slow(700));

    // Multiple closures can borrow immutably at the same time
    let latency_class = |ms: u32| {
        if ms < threshold / 5 { "fast" }
        else if ms < threshold { "acceptable" }
        else { "slow" }
    };

    for ms in [50u32, 200, 600] {
        println!("{}ms -> {}", ms, latency_class(ms));
    }
}

// ---- Capture by mutable borrow (&mut T) ----

fn capture_mutable() {
    println!("\n=== Capture: Mutable Borrow ===\n");

    let mut error_count = 0u32;
    let mut total_count = 0u32;

    // Closure mutates error_count and total_count — captures &mut
    let mut record_result = |success: bool| {
        total_count += 1;
        if !success {
            error_count += 1;
        }
    };

    record_result(true);
    record_result(true);
    record_result(false);
    record_result(true);
    record_result(false);

    // While the closure `record_result` is alive, we can't read error_count directly.
    // Drop the closure first by letting it go out of scope (or using drop()).
    drop(record_result);

    // Now we can access the variables again
    println!("total: {}, errors: {}", total_count, error_count);
    let error_rate = error_count as f64 / total_count as f64 * 100.0;
    println!("error rate: {:.1}%", error_rate);
}

// ---- Capture by move (ownership transfer) ----

fn capture_by_move() {
    println!("\n=== Capture: Move ===\n");

    let request_id = String::from("req-abc-123");

    // 'move' forces ownership transfer — request_id moves into the closure
    let log_request = move || {
        println!("Processing request: {}", request_id);
        // request_id is owned by this closure
    };

    // println!("{}", request_id); // ERROR: request_id moved into closure

    log_request();
    log_request(); // Can call multiple times — closure just reads request_id

    // Copy types are copied, not moved, even with 'move'
    let retry_count: u32 = 3;
    let show_retries = move || println!("retries: {}", retry_count);
    show_retries();
    println!("retry_count still accessible: {}", retry_count); // OK! u32 is Copy
}

// ---- FnOnce: closure that consumes captured values ----

// A FnOnce closure can only be called once because it moves something out.
fn run_once<F: FnOnce() -> String>(f: F) -> String {
    f()
    // Calling f() again here would be a compile error:
    // f() // ERROR: use of moved value `f`
}

fn fn_once_demo() {
    println!("\n=== FnOnce: Consume on Call ===\n");

    let sensitive_token = String::from("tok-secret-xyz");

    // This closure moves `sensitive_token` OUT when called.
    // It cannot be FnMut or Fn because after the first call, the token is gone.
    let consume_token = || {
        // Returning the String moves it out of the closure environment
        // and drops it after the function returns.
        let token = sensitive_token; // move out of captured env
        format!("Redacted token (was {} chars)", token.len())
    };

    // First call: works
    let result = run_once(consume_token);
    println!("Result: {}", result);

    // Second call would fail — consume_token can only be called once
    // run_once(consume_token); // ERROR: use of moved value `consume_token`
}

// ---- FnMut: closure that mutates captured state ----

// FnMut closures can be called many times, but require exclusive access while running.
fn apply_n_times<F: FnMut(i32) -> i32>(mut f: F, value: i32, n: u32) -> i32 {
    let mut result = value;
    for _ in 0..n {
        result = f(result);
    }
    result
}

fn fn_mut_demo() {
    println!("\n=== FnMut: Mutating State ===\n");

    let mut invocation_count = 0u32;

    // Mutates invocation_count — this is FnMut (not Fn)
    let mut tracked_double = |x: i32| {
        invocation_count += 1;
        x * 2
    };

    let result = apply_n_times(&mut tracked_double, 1, 5);
    drop(tracked_double); // release mut borrow so we can read invocation_count

    println!("After 5 doublings starting from 1: {}", result); // 32
    println!("Closure was invoked {} times", invocation_count);

    // Stateful closure: running sum
    let mut running_total = 0i64;
    let mut accumulate = |v: i64| {
        running_total += v;
        running_total
    };

    let deltas = [10i64, -3, 7, 100, -50];
    for &d in &deltas {
        println!("  +{:4} -> running total: {}", d, accumulate(d));
    }
    drop(accumulate);
    println!("Final total: {}", running_total);
}

// ---- Fn: pure, read-only capture ----

// Fn closures can be shared and called from multiple places, even concurrently.
fn apply_to_all<F: Fn(i32) -> i32>(items: &[i32], f: &F) -> Vec<i32> {
    items.iter().map(|&x| f(x)).collect()
}

fn fn_demo() {
    println!("\n=== Fn: Read-Only Capture ===\n");

    let multiplier = 4;

    // Only reads multiplier — implements Fn (and also FnMut and FnOnce)
    let quadruple = |x: i32| x * multiplier;

    let values = [1, 2, 3, 4, 5];
    let result = apply_to_all(&values, &quadruple);
    println!("quadrupled: {:?}", result);

    // We can call it again — multiplier wasn't mutated or consumed
    let result2 = apply_to_all(&[10, 20], &quadruple);
    println!("quadrupled again: {:?}", result2);

    // multiplier is still accessible — closure only borrowed it
    println!("multiplier still: {}", multiplier);
}

// ---- Closures in standard library ----

fn stdlib_closures() {
    println!("\n=== Standard Library Closures ===\n");

    // --- filter + map + collect (iterator chain) ---
    let raw_events: Vec<&str> = vec![
        "user.login",
        "db.query",
        "user.logout",
        "db.connection_error",
        "user.purchase",
        "cache.miss",
    ];

    let user_events: Vec<String> = raw_events
        .iter()
        .filter(|&&e| e.starts_with("user."))    // FnMut(&&&str) -> bool
        .map(|&e| e.to_uppercase())               // FnMut &&str -> String
        .collect();

    println!("User events: {:?}", user_events);

    // --- fold: build a summary ---
    let latencies_ms = [120u64, 45, 380, 210, 900, 55, 73];
    let (sum, count, max) = latencies_ms.iter().fold(
        (0u64, 0u64, 0u64),
        |(sum, count, max), &ms| (sum + ms, count + 1, max.max(ms)),
    );
    println!("\nLatency stats: avg={}ms, max={}ms, count={}", sum / count, max, count);

    // --- sort_by_key ---
    let mut endpoints: Vec<(&str, u32)> = vec![
        ("/health", 1000),
        ("/api/users", 45),
        ("/api/orders", 312),
        ("/metrics", 8),
    ];
    endpoints.sort_by_key(|&(_, hits)| std::cmp::Reverse(hits));
    println!("\nEndpoints by traffic:");
    for (path, hits) in &endpoints {
        println!("  {} -> {} hits", path, hits);
    }

    // --- retain: remove elements in place ---
    let mut active_sessions: Vec<(&str, u64)> = vec![
        ("session-a", 1000),
        ("session-b", 950),
        ("session-c", 1100),
        ("session-d", 800),
    ];
    let current_time = 1050u64;
    let ttl = 100u64;
    active_sessions.retain(|(_, created_at)| current_time - created_at < ttl);
    println!("\nActive sessions after expiry: {:?}", active_sessions);

    // --- any and all ---
    let error_codes = [200u16, 200, 404, 200, 500];
    let has_error = error_codes.iter().any(|&c| c >= 400);
    let all_success = error_codes.iter().all(|&c| c < 400);
    println!("\nhas 4xx/5xx: {}, all success: {}", has_error, all_success);

    // --- find_map: find and transform in one pass ---
    let log_lines = [
        "INFO: startup complete",
        "DEBUG: loading config",
        "ERROR: connection refused to db:5432",
        "WARN: retry attempt 1",
    ];
    let first_error = log_lines
        .iter()
        .find_map(|&line| {
            if line.starts_with("ERROR:") {
                Some(line.trim_start_matches("ERROR: "))
            } else {
                None
            }
        });
    println!("\nFirst error: {:?}", first_error);
}

fn main() {
    closure_syntax_examples();
    capture_immutable();
    capture_mutable();
    capture_by_move();
    fn_once_demo();
    fn_mut_demo();
    fn_demo();
    stdlib_closures();
}
