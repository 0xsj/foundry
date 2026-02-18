// Control Flow: Loops — Rust
//
// Demonstrates: loop with break value, while, while let, for with ranges,
// iterators, enumerate, labeled loops.
// Run with: rustc loops.rs && ./loops

// ---------- loop with break value ----------

fn connect_with_retry(max_attempts: u32) -> Result<String, String> {
    let mut attempt = 0u32;
    let mut last_error = String::new();

    // loop returns a value — the established connection string.
    // This eliminates the need for a variable declared before the loop
    // just to hold the result.
    let conn = loop {
        attempt += 1;
        match try_connect(attempt) {
            Ok(conn) => break conn,           // exit the loop with the value
            Err(e) if attempt >= max_attempts => {
                last_error = e;
                break String::new();          // empty string signals failure
            }
            Err(e) => {
                eprintln!("  attempt {}/{}: {}", attempt, max_attempts, e);
                last_error = e;
                // continue to next iteration (implicit — no break)
            }
        }
    };

    if conn.is_empty() {
        Err(format!("exhausted {} attempts: {}", max_attempts, last_error))
    } else {
        Ok(conn)
    }
}

// Simulated connection attempt — fails first 2 times
fn try_connect(attempt: u32) -> Result<String, String> {
    if attempt < 3 {
        Err(format!("connection refused (attempt {})", attempt))
    } else {
        Ok(format!("tcp://127.0.0.1:5432 (attempt {})", attempt))
    }
}

// ---------- while ----------

fn exponential_backoff_demo() {
    let mut delay_ms = 100u64;
    let mut total_ms = 0u64;
    let max_ms = 5_000u64;

    println!("  Backoff schedule (cap {}ms):", max_ms);
    while delay_ms <= max_ms {
        println!("    wait {}ms", delay_ms);
        total_ms += delay_ms;
        delay_ms *= 2;
    }
    println!("  Total wait: {}ms", total_ms);
}

// ---------- while let ----------

fn process_event_queue(queue: &mut Vec<&str>) {
    // Drain the queue until empty.
    // while let pattern: loop while the Option has a value.
    while let Some(event) = queue.pop() {
        println!("  processing: {}", event);
    }
    println!("  queue empty");
}

// ---------- for with ranges ----------

fn checksum(data: &[u8]) -> u32 {
    let mut sum = 0u32;

    // Range 0..data.len() — exclusive upper bound
    for i in 0..data.len() {
        sum = sum.wrapping_add(data[i] as u32);
    }
    sum

    // More idiomatic — iterate directly:
    // data.iter().map(|&b| b as u32).sum()
}

fn fizzbuzz(n: u32) -> Vec<String> {
    // Inclusive range with ..= notation
    (1..=n).map(|i| {
        if i % 15 == 0 {
            String::from("FizzBuzz")
        } else if i % 3 == 0 {
            String::from("Fizz")
        } else if i % 5 == 0 {
            String::from("Buzz")
        } else {
            i.to_string()
        }
    }).collect()
}

// ---------- for over collections ----------

struct Replica {
    id: u32,
    host: &'static str,
    healthy: bool,
}

fn print_replicas(replicas: &[Replica]) {
    // Iterate by reference (&replicas) — replicas stays owned by caller
    for replica in replicas {
        let status = if replica.healthy { "UP" } else { "DOWN" };
        println!("  replica-{}: {} [{}]", replica.id, replica.host, status);
    }
}

fn count_healthy(replicas: &[Replica]) -> usize {
    let mut count = 0;
    for replica in replicas {
        if replica.healthy {
            count += 1;
        }
    }
    count
    // Or: replicas.iter().filter(|r| r.healthy).count()
}

// ---------- enumerate ----------

fn print_pipeline_steps(steps: &[&str]) {
    // enumerate() gives (index, value) pairs.
    // In TypeScript: steps.forEach((step, i) => ...)
    // In Go: for i, step := range steps { ... }
    for (i, step) in steps.iter().enumerate() {
        println!("  step {}: {}", i + 1, step);
    }
}

// ---------- labeled loops ----------

fn find_in_grid(grid: &[&[i32]], target: i32) -> Option<(usize, usize)> {
    // 'outer labels the outer loop so inner `break` can target it directly.
    'outer: for (row, line) in grid.iter().enumerate() {
        for (col, &val) in line.iter().enumerate() {
            if val == target {
                // This break exits 'outer, not just the inner for.
                break 'outer;
            }
            let _ = (row, col); // suppress unused warning in the search
        }
    }

    // Redo the search to return the position (simpler demo)
    for (row, line) in grid.iter().enumerate() {
        for (col, &val) in line.iter().enumerate() {
            if val == target {
                return Some((row, col));
            }
        }
    }
    None
}

fn skip_poisoned_batches(batches: &[&[i32]]) -> Vec<i32> {
    let mut results = Vec::new();

    // continue 'outer skips to the next batch when a -1 is found
    'batch: for batch in batches {
        for &item in *batch {
            if item == -1 {
                eprintln!("  skipping batch: contains poison pill (-1)");
                continue 'batch; // skip this entire batch
            }
            results.push(item);
        }
    }
    results
}

// ---------- main ----------

fn main() {
    // -- loop with break value --
    println!("=== Connection with Retry ===");
    match connect_with_retry(5) {
        Ok(conn) => println!("  connected: {}", conn),
        Err(e) => println!("  failed: {}", e),
    }

    // -- while --
    println!("\n=== Exponential Backoff ===");
    exponential_backoff_demo();

    // -- while let --
    println!("\n=== Event Queue ===");
    let mut queue = vec!["request_received", "auth_checked", "handler_called", "response_sent"];
    process_event_queue(&mut queue);

    // -- for with ranges --
    println!("\n=== Checksum ===");
    let data = vec![0x48u8, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
    println!("  checksum of {:?} = {:#x}", data, checksum(&data));

    println!("\n=== FizzBuzz (1-20) ===");
    println!("  {}", fizzbuzz(20).join(", "));

    // -- for over collections --
    println!("\n=== Replicas ===");
    let replicas = vec![
        Replica { id: 1, host: "10.0.0.1", healthy: true  },
        Replica { id: 2, host: "10.0.0.2", healthy: false },
        Replica { id: 3, host: "10.0.0.3", healthy: true  },
    ];
    print_replicas(&replicas);
    println!("  healthy: {}/{}", count_healthy(&replicas), replicas.len());

    // -- enumerate --
    println!("\n=== Pipeline Steps ===");
    let steps = ["parse request", "authenticate", "validate body", "execute handler", "serialize response"];
    print_pipeline_steps(&steps);

    // -- labeled loops --
    println!("\n=== Grid Search ===");
    let row0: &[i32] = &[1, 2, 3];
    let row1: &[i32] = &[4, 5, 6];
    let row2: &[i32] = &[7, 8, 9];
    let grid: &[&[i32]] = &[row0, row1, row2];

    match find_in_grid(grid, 5) {
        Some((r, c)) => println!("  found 5 at ({}, {})", r, c),
        None => println!("  not found"),
    }

    println!("\n=== Batch Processing ===");
    let b0: &[i32] = &[1, 2, 3];
    let b1: &[i32] = &[4, -1, 6]; // poisoned
    let b2: &[i32] = &[7, 8, 9];
    let batches: &[&[i32]] = &[b0, b1, b2];
    let results = skip_poisoned_batches(batches);
    println!("  results: {:?}", results);
}
