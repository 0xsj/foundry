// Iteration — iter/into_iter/iter_mut, collect, from_iter, chaining
//
// Run: rustc iterating.rs && ./iterating

use std::collections::{HashMap, HashSet};

fn main() {
    // -------------------------------------------------------------------------
    // The three iteration modes
    // -------------------------------------------------------------------------

    println!("=== iter() vs into_iter() vs iter_mut() ===");

    let services = vec![
        String::from("auth"),
        String::from("api"),
        String::from("worker"),
    ];

    // iter() — immutable borrow, yields &T, collection survives
    println!("services (via iter):");
    for name in services.iter() {          // name: &String
        println!("  {name}");
    }
    println!("services still alive: {} items", services.len());

    // iter_mut() — mutable borrow, yields &mut T, allows in-place modification
    let mut response_times_ms: Vec<u64> = vec![120, 85, 340, 60, 210];
    let threshold = 200;
    for t in response_times_ms.iter_mut() { // t: &mut u64
        if *t > threshold {
            *t = threshold;  // cap at threshold
        }
    }
    println!("\ncapped response times: {:?}", response_times_ms);

    // into_iter() — moves elements out, collection is consumed
    let job_ids: Vec<u32> = vec![1001, 1002, 1003];
    println!("\nprocessing jobs (consumed):");
    for id in job_ids.into_iter() {        // id: u32 (moved out)
        println!("  dispatching job {id}");
    }
    // job_ids is gone — cannot use after this point

    // for x in &collection desugars to for x in collection.iter()
    // for x in collection    desugars to for x in collection.into_iter()
    // Both are equivalent to the explicit forms above

    // -------------------------------------------------------------------------
    // Common iterator adapters
    // -------------------------------------------------------------------------

    println!("\n=== Iterator Adapters ===");

    let requests: Vec<(&str, u16, bool)> = vec![  // (path, status, is_error)
        ("/api/users", 200, false),
        ("/api/items", 404, true),
        ("/health", 200, false),
        ("/api/login", 500, true),
        ("/api/logout", 200, false),
        ("/api/data", 503, true),
    ];

    // filter: keep only error responses
    let errors: Vec<_> = requests.iter()
        .filter(|(_, _, is_err)| *is_err)
        .collect();
    println!("error requests: {}", errors.len());

    // map: extract just the status codes
    let status_codes: Vec<u16> = requests.iter()
        .map(|(_, status, _)| *status)
        .collect();
    println!("status codes: {:?}", status_codes);

    // filter_map: filter and transform in one step
    let error_paths: Vec<&str> = requests.iter()
        .filter_map(|(path, _, is_err)| if *is_err { Some(*path) } else { None })
        .collect();
    println!("error paths: {:?}", error_paths);

    // enumerate: get index alongside value
    println!("\nrequests with index:");
    for (i, (path, status, _)) in requests.iter().enumerate() {
        println!("  [{i}] {path} -> {status}");
    }

    // chain: concatenate two iterators
    let prod_hosts = vec!["prod-1", "prod-2"];
    let canary_hosts = vec!["canary-1"];
    let all_hosts: Vec<&&str> = prod_hosts.iter().chain(canary_hosts.iter()).collect();
    println!("\nall hosts: {:?}", all_hosts);

    // take and skip
    let events: Vec<i32> = (1..=10).collect();
    let page_2: Vec<&i32> = events.iter().skip(3).take(3).collect(); // items 4,5,6
    println!("page 2 (skip 3, take 3): {:?}", page_2);

    // -------------------------------------------------------------------------
    // Consumers: fold, sum, count, any, all, find
    // -------------------------------------------------------------------------

    println!("\n=== Consumers ===");

    let latencies_ms: Vec<u64> = vec![45, 112, 8, 230, 67, 44, 198, 12];

    let count = latencies_ms.len();
    let total: u64 = latencies_ms.iter().sum();
    let avg = total / count as u64;
    let max = latencies_ms.iter().max().unwrap();
    let p99_violators = latencies_ms.iter().filter(|&&ms| ms > 200).count();

    println!("count: {count}, avg: {avg}ms, max: {max}ms, p99 violators: {p99_violators}");

    let any_critical = latencies_ms.iter().any(|&ms| ms > 500);
    let all_healthy = latencies_ms.iter().all(|&ms| ms < 1000);
    println!("any critical: {any_critical}, all healthy: {all_healthy}");

    let first_slow = latencies_ms.iter().find(|&&ms| ms > 100);
    println!("first slow request: {:?}", first_slow);

    // fold: general reduction with an accumulator
    let report = latencies_ms.iter().fold(String::from("latencies: ["), |mut acc, &ms| {
        acc.push_str(&ms.to_string());
        acc.push(',');
        acc
    });
    println!("{}", &report[..report.len()-1]); // trim trailing comma

    // -------------------------------------------------------------------------
    // collect: turning iterators into collections
    // -------------------------------------------------------------------------

    println!("\n=== collect ===");

    let raw_tags = vec!["  rust ", "go", " typescript  ", "rust", "go", "rust"];

    // Collect into Vec<String> (trimmed, owned)
    let tags_trimmed: Vec<String> = raw_tags.iter()
        .map(|s| s.trim().to_string())
        .collect();
    println!("trimmed: {:?}", tags_trimmed);

    // Collect into HashSet to deduplicate
    let unique_tags: HashSet<String> = raw_tags.iter()
        .map(|s| s.trim().to_string())
        .collect();
    let mut sorted: Vec<String> = unique_tags.into_iter().collect();
    sorted.sort();
    println!("unique: {:?}", sorted);

    // Collect pairs into HashMap
    let kv_pairs = vec![("timeout", 30u32), ("retries", 3), ("pool_size", 10)];
    let settings: HashMap<&str, u32> = kv_pairs.into_iter().collect();
    println!("settings: retries={}", settings["retries"]);

    // Turbofish syntax: specify type on collect instead of the binding
    let doubled = vec![1u32, 2, 3, 4, 5]
        .iter()
        .map(|&x| x * 2)
        .collect::<Vec<u32>>();
    println!("doubled: {:?}", doubled);

    // Collect into a String from chars
    let label: String = "env".chars()
        .map(|c| c.to_uppercase().next().unwrap())
        .collect();
    println!("label: {label}");

    // -------------------------------------------------------------------------
    // Collecting Results: short-circuit on first error
    // -------------------------------------------------------------------------

    println!("\n=== Collecting Results ===");

    let raw_ports = vec!["8080", "9090", "not-a-number", "3000"];
    let ports: Result<Vec<u16>, _> = raw_ports.iter()
        .map(|s| s.parse::<u16>())
        .collect();
    println!("parsing mixed ports: {:?}", ports);  // Err at "not-a-number"

    let valid_ports = vec!["8080", "9090", "3000"];
    let ports: Result<Vec<u16>, _> = valid_ports.iter()
        .map(|s| s.parse::<u16>())
        .collect();
    println!("parsing valid ports: {:?}", ports);  // Ok([8080, 9090, 3000])

    // -------------------------------------------------------------------------
    // flat_map and flatten
    // -------------------------------------------------------------------------

    println!("\n=== flat_map / flatten ===");

    // Each service has a list of active connections — flatten into one list
    let service_connections: Vec<(&str, Vec<u32>)> = vec![
        ("auth", vec![1001, 1002]),
        ("api", vec![2001, 2002, 2003]),
        ("worker", vec![3001]),
    ];

    let all_connection_ids: Vec<u32> = service_connections.iter()
        .flat_map(|(_, conns)| conns.iter().copied())
        .collect();
    println!("all connection ids: {:?}", all_connection_ids);

    // Split log lines into words
    let log_lines = vec!["connection refused", "disk full on /var", "timeout 30s"];
    let all_words: Vec<&str> = log_lines.iter()
        .flat_map(|line| line.split(' '))
        .collect();
    println!("all words: {:?}", all_words);

    // -------------------------------------------------------------------------
    // zip and unzip
    // -------------------------------------------------------------------------

    println!("\n=== zip ===");

    let keys = vec!["a", "b", "c"];
    let values = vec![1u32, 2, 3];

    // zip two iterators into pairs
    let pairs: Vec<(&&str, &u32)> = keys.iter().zip(values.iter()).collect();
    println!("zipped: {:?}", pairs);

    // zip into a HashMap
    let map: HashMap<&str, u32> = keys.into_iter().zip(values.into_iter()).collect();
    println!("as map: {:?}", map);
}
