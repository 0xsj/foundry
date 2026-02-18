// Higher-Order Functions — Rust
//
// Demonstrates: functions accepting closures (impl Fn / impl FnMut / impl FnOnce),
// returning closures (impl Fn vs Box<dyn Fn>), function composition, and the
// practical difference between generic bounds and trait objects.
//
// Run: rustc higher_order.rs && ./higher_order

// ---- Accepting closures: impl Fn in argument position ----

// `impl Fn(T) -> U` is syntactic sugar for a generic parameter.
// The compiler generates a monomorphized version for each concrete closure type.
// No heap allocation, no virtual dispatch — zero overhead.
fn transform<T, U>(value: T, f: impl Fn(T) -> U) -> U {
    f(value)
}

// Accepting multiple closures: each impl Fn is a distinct type parameter.
// You can pass the same closure type or different ones.
fn filter_and_map<T, U>(
    items: &[T],
    predicate: impl Fn(&T) -> bool,
    mapper: impl Fn(&T) -> U,
) -> Vec<U> {
    items
        .iter()
        .filter(|x| predicate(x))
        .map(|x| mapper(x))
        .collect()
}

fn accepting_closures_demo() {
    println!("=== Accepting Closures ===\n");

    // transform with different closure types
    let doubled = transform(21, |x| x * 2);
    let uppercased = transform("hello", |s: &str| s.to_uppercase());
    println!("doubled: {}", doubled);
    println!("uppercased: {}", uppercased);

    // filter_and_map: filtering strings, mapping to length
    let endpoints = ["GET /health", "POST /api/login", "GET /api/users", "DELETE /api/session"];
    let get_paths: Vec<String> = filter_and_map(
        &endpoints,
        |e| e.starts_with("GET"),
        |e| e.trim_start_matches("GET ").to_string(),
    );
    println!("GET paths: {:?}", get_paths);
}

// ---- Accepting FnMut: when the closure needs to mutate ----

// Iterator::fold takes FnMut because it accumulates state.
// Writing our own version:
fn fold_custom<T, B, F: FnMut(B, T) -> B>(items: Vec<T>, init: B, mut f: F) -> B {
    let mut acc = init;
    for item in items {
        acc = f(acc, item);
    }
    acc
}

// A function that calls a closure many times — needs mut because closure might mutate
fn retry<F: FnMut() -> bool>(mut operation: F, max_attempts: u32) -> bool {
    for attempt in 1..=max_attempts {
        if operation() {
            println!("  Succeeded on attempt {}", attempt);
            return true;
        }
        println!("  Attempt {} failed, retrying...", attempt);
    }
    false
}

fn fn_mut_accepting_demo() {
    println!("\n=== Accepting FnMut ===\n");

    // fold_custom: accumulate a sum
    let numbers = vec![1i64, 2, 3, 4, 5];
    let sum = fold_custom(numbers, 0, |acc, x| acc + x);
    println!("sum via fold_custom: {}", sum);

    // fold_custom: build a string
    let words = vec!["one", "two", "three"];
    let joined = fold_custom(words, String::new(), |mut acc, w| {
        if !acc.is_empty() {
            acc.push_str(", ");
        }
        acc.push_str(w);
        acc
    });
    println!("joined: {}", joined);

    // retry: closure mutates its own counter to simulate a flaky operation
    let mut attempt_count = 0u32;
    let flaky_operation = || {
        attempt_count += 1;
        attempt_count >= 3  // succeeds on the 3rd attempt
    };

    println!("Retrying flaky operation:");
    let succeeded = retry(flaky_operation, 5);
    println!("  Final: succeeded={}", succeeded);
}

// ---- Accepting FnOnce: one-shot callbacks ----

// Some operations are inherently one-shot: the callback fires and the value is consumed.
fn with_resource<R, T, F: FnOnce(R) -> T>(resource: R, callback: F) -> T {
    callback(resource)
    // resource is consumed — no way to accidentally reuse it
}

fn fn_once_accepting_demo() {
    println!("\n=== Accepting FnOnce ===\n");

    // Process an owned value and consume it
    let config_data = String::from(r#"{"timeout":30,"retries":3}"#);

    let parsed_timeout = with_resource(config_data, |data| {
        // In real code: serde_json::from_str(&data).unwrap()
        if data.contains("timeout") { 30u32 } else { 10u32 }
    });
    // config_data is gone — moved into callback
    println!("parsed timeout: {}s", parsed_timeout);

    // One-shot initialization pattern
    let connection_string = String::from("postgres://localhost/mydb");
    let pool_name = with_resource(connection_string, |cs| {
        format!("pool[{}]", &cs[..20.min(cs.len())])
    });
    println!("pool name: {}", pool_name);
}

// ---- Returning closures: impl Fn in return position ----

// impl Fn in return position: the function returns a single concrete closure type.
// The move keyword is almost always required — captured values must outlive the function scope.
fn make_prefixer(prefix: String) -> impl Fn(&str) -> String {
    // prefix is moved into the closure — it outlives this function call
    move |s| format!("{}{}", prefix, s)
}

fn make_rate_checker(limit_per_second: f64) -> impl Fn(f64) -> bool {
    move |rate| rate <= limit_per_second
}

// Compose two functions into one
fn compose<A, B, C>(
    f: impl Fn(A) -> B + 'static,
    g: impl Fn(B) -> C + 'static,
) -> impl Fn(A) -> C {
    // Both closures are moved into this outer closure
    // 'static bounds ensure they don't contain non-static borrows
    move |x| g(f(x))
}

fn returning_impl_fn_demo() {
    println!("\n=== Returning impl Fn ===\n");

    let add_prefix = make_prefixer(String::from("srv-"));
    println!("{}", add_prefix("database"));  // srv-database
    println!("{}", add_prefix("cache"));     // srv-cache
    println!("{}", add_prefix("queue"));     // srv-queue

    let is_within_limit = make_rate_checker(100.0);
    for rate in [50.0f64, 100.0, 101.0, 200.0] {
        println!("rate {:.0} req/s: within limit = {}", rate, is_within_limit(rate));
    }

    // Function composition
    let parse = |s: &str| s.parse::<i32>().unwrap_or(0);
    let negate = |n: i32| -n;
    let parse_and_negate = compose(parse, negate);
    println!("\nparse_and_negate(\"42\") = {}", parse_and_negate("42"));  // -42
    println!("parse_and_negate(\"-7\") = {}", parse_and_negate("-7")); // 7
}

// ---- Returning Box<dyn Fn>: when the concrete type varies ----

// When different branches return different closure types, impl Fn doesn't work
// (the concrete type must be the same across all paths). Box<dyn Fn> erases the type.
fn make_formatter(format: &str) -> Box<dyn Fn(&str) -> String> {
    match format {
        "upper"  => Box::new(|s: &str| s.to_uppercase()),
        "lower"  => Box::new(|s: &str| s.to_lowercase()),
        "snake"  => Box::new(|s: &str| s.replace(' ', "_").to_lowercase()),
        "kebab"  => Box::new(|s: &str| s.replace(' ', "-").to_lowercase()),
        _        => Box::new(|s: &str| s.to_string()),  // identity
    }
}

// Storing heterogeneous closures in a Vec — requires Box<dyn Fn>
fn build_validation_pipeline(rules: Vec<Box<dyn Fn(&str) -> Result<(), String>>>)
    -> impl Fn(&str) -> Result<(), Vec<String>>
{
    move |input| {
        let errors: Vec<String> = rules
            .iter()
            .filter_map(|rule| rule(input).err())
            .collect();
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

fn returning_box_dyn_demo() {
    println!("\n=== Returning Box<dyn Fn> ===\n");

    // Different formatters, same interface
    let formats = ["upper", "lower", "snake", "kebab", "passthrough"];
    let sample = "Hello World From Rust";
    for fmt in formats {
        let formatter = make_formatter(fmt);
        println!("  {:12} -> {}", fmt, formatter(sample));
    }

    // Validation pipeline: heterogeneous rules stored in Vec<Box<dyn Fn>>
    let rules: Vec<Box<dyn Fn(&str) -> Result<(), String>>> = vec![
        Box::new(|s: &str| {
            if s.is_empty() { Err(String::from("must not be empty")) }
            else { Ok(()) }
        }),
        Box::new(|s: &str| {
            if s.len() < 3 { Err(format!("must be at least 3 chars, got {}", s.len())) }
            else { Ok(()) }
        }),
        Box::new(|s: &str| {
            if s.contains(' ') { Err(String::from("must not contain spaces")) }
            else { Ok(()) }
        }),
        Box::new(|s: &str| {
            if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') { Ok(()) }
            else { Err(String::from("must be alphanumeric or underscore")) }
        }),
    ];

    let validate = build_validation_pipeline(rules);

    println!("\nValidation pipeline:");
    for input in ["", "ok", "hello world", "valid_input_123", "bad!char"] {
        match validate(input) {
            Ok(()) => println!("  {:20} -> valid", format!("\"{}\"", input)),
            Err(errs) => println!("  {:20} -> invalid: {}", format!("\"{}\"", input), errs.join("; ")),
        }
    }
}

// ---- Function pointer vs closure trait comparison ----

// fn() is a concrete type: zero size, Copy, no captures.
// impl Fn() is a trait bound: monomorphized, can have captures.
// &dyn Fn() is a fat pointer: vtable, runtime dispatch, size = 2 pointers.
// Box<dyn Fn()> is heap-allocated fat pointer.

fn takes_fn_pointer(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn takes_impl_fn(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn takes_dyn_fn(f: &dyn Fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn fn_ptr_vs_trait_demo() {
    println!("\n=== fn Pointer vs impl Fn vs dyn Fn ===\n");

    fn triple(x: i32) -> i32 { x * 3 }
    let factor = 4;
    let capturing_closure = |x: i32| x * factor;
    let non_capturing = |x: i32| x * 5;

    // fn pointer: only works with non-capturing closures and named functions
    println!("fn pointer, triple(7)  = {}", takes_fn_pointer(triple, 7));
    println!("fn pointer, non_cap(7) = {}", takes_fn_pointer(non_capturing, 7));
    // takes_fn_pointer(capturing_closure, 7); // ERROR: closure captures, not a fn pointer

    // impl Fn: works with everything — named functions, capturing and non-capturing closures
    println!("impl Fn, triple(7)      = {}", takes_impl_fn(triple, 7));
    println!("impl Fn, capturing(7)  = {}", takes_impl_fn(capturing_closure, 7));
    println!("impl Fn, non_cap(7)    = {}", takes_impl_fn(non_capturing, 7));

    // &dyn Fn: runtime dispatch — useful when type must be erased
    println!("dyn Fn, triple(7)      = {}", takes_dyn_fn(&triple, 7));
    println!("dyn Fn, capturing(7)   = {}", takes_dyn_fn(&capturing_closure, 7));
    println!("dyn Fn, non_cap(7)     = {}", takes_dyn_fn(&non_capturing, 7));

    // Storing mixed callables in a Vec — requires trait objects
    let transformers: Vec<Box<dyn Fn(i32) -> i32>> = vec![
        Box::new(triple),
        Box::new(capturing_closure),
        Box::new(|x| x + 100),
    ];
    let results: Vec<i32> = transformers.iter().map(|f| f(10)).collect();
    println!("\nMixed transformer Vec, applied to 10: {:?}", results);
}

fn main() {
    accepting_closures_demo();
    fn_mut_accepting_demo();
    fn_once_accepting_demo();
    returning_impl_fn_demo();
    returning_box_dyn_demo();
    fn_ptr_vs_trait_demo();
}
