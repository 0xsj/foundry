// Vec<T> — Creation, capacity, slicing, drain, retain, borrowing rules
//
// Run: rustc vectors.rs && ./vectors

fn main() {
    // -------------------------------------------------------------------------
    // Creation
    // -------------------------------------------------------------------------

    let empty: Vec<i32> = Vec::new();                 // len=0, capacity=0 (no allocation)
    let from_macro = vec![10, 20, 30];                // macro: initialized with values
    let repeated = vec![0u8; 8];                      // 8 copies of 0
    let preallocated: Vec<String> = Vec::with_capacity(100); // len=0, capacity=100

    println!("=== Creation ===");
    println!("empty: len={}, cap={}", empty.len(), empty.capacity());
    println!("from macro: {:?}", from_macro);
    println!("repeated: {:?}", repeated);
    println!("preallocated: len={}, cap={}", preallocated.len(), preallocated.capacity());

    // -------------------------------------------------------------------------
    // Push, pop, capacity growth
    // -------------------------------------------------------------------------

    println!("\n=== Capacity Growth ===");
    let mut v: Vec<i32> = Vec::new();
    for i in 0..8 {
        let before = v.capacity();
        v.push(i);
        let after = v.capacity();
        if after != before {
            println!("push({i}): capacity grew {} -> {}", before, after);
        }
    }
    // Typical output shows growth at 0->4->8 (roughly 2x strategy)

    let last = v.pop();   // returns Option<i32>
    println!("pop: {:?}  v now has {} elements", last, v.len());

    // -------------------------------------------------------------------------
    // Indexing: [] vs .get()
    // -------------------------------------------------------------------------

    println!("\n=== Indexing ===");
    let scores = vec![85, 92, 78, 95, 60];

    let first = scores[0];                   // panics if out of bounds
    println!("scores[0] = {first}");

    match scores.get(10) {                   // safe — returns Option
        Some(s) => println!("scores[10] = {s}"),
        None => println!("scores[10] = None (out of bounds, no panic)"),
    }

    // -------------------------------------------------------------------------
    // Slices: borrowing a contiguous view
    // -------------------------------------------------------------------------

    println!("\n=== Slices ===");
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let middle: &[i32] = &data[2..6];         // borrows elements 2, 3, 4, 5
    let all: &[i32] = &data[..];              // borrows entire vec

    println!("middle slice: {:?}", middle);
    println!("sum via slice fn: {}", sum_slice(middle));
    println!("sum of all: {}", sum_slice(all));

    // You can pass the whole vec where a slice is expected — automatic coercion
    let total = sum_slice(&data);
    println!("total: {total}");

    // -------------------------------------------------------------------------
    // drain: remove a range and iterate over removed elements
    // -------------------------------------------------------------------------

    println!("\n=== drain ===");
    let mut events = vec!["login", "view", "click", "logout", "login", "click"];

    // Remove the first 3 and process them (e.g., flush to a log sink)
    let batch: Vec<&str> = events.drain(0..3).collect();
    println!("drained batch: {:?}", batch);
    println!("remaining: {:?}", events);
    // events is still valid and owns the remaining elements

    // -------------------------------------------------------------------------
    // retain: in-place filter
    // -------------------------------------------------------------------------

    println!("\n=== retain ===");
    let mut request_sizes: Vec<u32> = vec![128, 8192, 512, 65536, 256, 4096];

    // Keep only requests under 1 KB — reject oversized payloads
    request_sizes.retain(|&size| size <= 1024);
    println!("requests under 1 KB: {:?}", request_sizes);

    let mut error_codes = vec![200, 404, 200, 500, 503, 200, 302];
    error_codes.retain(|&code| code >= 400);  // keep only error/redirect codes
    println!("non-success codes: {:?}", error_codes);

    // -------------------------------------------------------------------------
    // Borrow rules: cannot hold reference and mutate
    // -------------------------------------------------------------------------

    println!("\n=== Borrowing Rules ===");
    let mut jobs: Vec<String> = Vec::new();
    jobs.push(String::from("encode-video-1"));
    jobs.push(String::from("encode-video-2"));

    // Pattern: copy the data out (String supports Clone) before mutating
    let first_job = jobs[0].clone();     // clone the String, releasing the borrow
    jobs.push(String::from("encode-video-3")); // now we can mutate
    println!("first job: {}, total: {}", first_job, jobs.len());

    // Pattern: use indices instead of references
    let idx = 0;
    jobs.push(String::from("encode-video-4"));
    println!("job at 0: {}", jobs[idx]);  // index after all mutations are done

    // -------------------------------------------------------------------------
    // swap_remove: O(1) removal (doesn't preserve order)
    // -------------------------------------------------------------------------

    println!("\n=== swap_remove ===");
    let mut ids = vec![101, 102, 103, 104, 105];
    let removed = ids.swap_remove(1);   // removes 102, swaps last element (105) into its place
    println!("removed: {removed}, ids: {:?}", ids);
    // ids is now [101, 105, 103, 104] — order changed, but O(1) vs O(n) for remove()

    // -------------------------------------------------------------------------
    // sort and dedup
    // -------------------------------------------------------------------------

    println!("\n=== sort and dedup ===");
    let mut timestamps = vec![1700, 1200, 1500, 1200, 1700, 1800];
    timestamps.sort();
    println!("sorted: {:?}", timestamps);

    timestamps.dedup();   // removes CONSECUTIVE duplicates — must sort first
    println!("deduped: {:?}", timestamps);

    // Sort by a key field
    let mut log_entries: Vec<(&str, u32)> = vec![
        ("worker-3", 1705),
        ("worker-1", 1700),
        ("worker-2", 1702),
    ];
    log_entries.sort_by_key(|&(_, ts)| ts);
    println!("sorted by timestamp: {:?}", log_entries);
}

/// Functions should accept &[T] slices, not &Vec<T>.
/// This way the caller can pass a Vec, a subslice, a fixed-size array — anything contiguous.
fn sum_slice(data: &[i32]) -> i32 {
    data.iter().sum()
}
