# Control Flow — Rust

## Everything Is an Expression

Before diving into specific constructs, grasp this: in Rust, almost everything
is an expression that produces a value. `if`, `match`, and `loop` all return
values. There is no ternary operator (`? :`) because you don't need one —
`if/else` already does the job.

```rust
// In JavaScript/TypeScript you'd write: const label = x > 0 ? "positive" : "non-positive";
// In Go you'd write a var block and an if statement.
// In Rust:
let label = if x > 0 { "positive" } else { "non-positive" };
```

The last expression in a block (without a semicolon) is the block's value. This
is not just a nice feature — it's how the language works at the type level.

### Your notes
<!-- -->


---

## if / else

### Basics

```rust
let status_code = 404;

if status_code == 200 {
    println!("ok");
} else if status_code == 404 {
    println!("not found");
} else {
    println!("other: {}", status_code);
}
```

No parentheses around the condition. Braces are required (unlike C/Go where single
statements can skip them — Rust always requires braces).

The condition must be `bool`. Rust does not perform implicit truthiness checks:

```rust
let n = 1;
// if n { ... }   // ERROR: expected `bool`, found integer
if n != 0 { ... } // correct
```

This is different from Go (which also requires bool) and very different from
JavaScript (where any value is coerced).

### if as an Expression

When `if` returns a value, both branches must produce the same type:

```rust
fn classify_latency(ms: u64) -> &'static str {
    if ms < 100 {
        "fast"    // no semicolon — this is the branch's value
    } else if ms < 500 {
        "acceptable"
    } else {
        "slow"
    }
}
```

Notice: no `return` needed. The last expression in the function body (without a
semicolon) is the implicit return value. The `return` keyword exists but is only
needed for early returns.

The types of both branches must unify. This fails:

```rust
let x = if condition { 42 } else { "hello" }; // ERROR: incompatible types
```

### if let

`if let` is a convenience for pattern matching that only cares about one variant:

```rust
let maybe_timeout: Option<u64> = get_timeout_ms();

// Verbose match:
match maybe_timeout {
    Some(ms) => println!("timeout: {}ms", ms),
    None => {} // do nothing
}

// Idiomatic if let:
if let Some(ms) = maybe_timeout {
    println!("timeout: {}ms", ms);
}
```

`if let` with an `else`:

```rust
if let Some(user) = find_user(id) {
    process_request(user);
} else {
    return Err("user not found");
}
```

This is especially useful when you need the bound variable in the success path.

### let-else (Rust 1.65+)

For "bail out if the pattern doesn't match" scenarios, `let-else` is even more
concise:

```rust
fn process_event(event: &str) -> Result<(), String> {
    let Some(payload) = parse_payload(event) else {
        return Err("missing payload".to_string());
    };
    // payload is bound here and usable
    println!("processing: {}", payload);
    Ok(())
}
```

The `else` block must diverge (return, panic, break, continue, or `!`). This is
how Rust avoids having the Java/Go pattern of:

```go
// Go — verbose unwrap pattern
user, ok := getUser(id)
if !ok {
    return nil, errors.New("user not found")
}
// use user
```

### Your notes
<!-- -->


---

## loop

`loop` creates an infinite loop. It's the foundation — `while` and `for` are
built on top of it conceptually.

```rust
loop {
    let input = read_line();
    if input == "quit" {
        break;
    }
    process(input);
}
```

### break with a value

`loop` can return a value via `break`:

```rust
let mut attempts = 0;
let connection = loop {
    attempts += 1;
    match try_connect() {
        Ok(conn) => break conn,       // break with a value
        Err(_) if attempts >= 3 => break panic!("too many retries"),
        Err(e) => eprintln!("retry {}: {}", attempts, e),
    }
};
// connection is now the Ok(conn) value
```

This is unique to Rust among the languages in this curriculum. Go doesn't have
it (Go's `for` loops can't return values), and TypeScript `while` loops can't
either. The pattern eliminates the need for a variable declared outside the loop
just to capture a result.

### Labeled loops

When you have nested loops and need `break` or `continue` to target a specific
one, use labels:

```rust
'outer: for row in 0..height {
    for col in 0..width {
        if grid[row][col] == TARGET {
            println!("found at ({}, {})", row, col);
            break 'outer;  // breaks the outer loop
        }
    }
}
```

Labels are prefixed with `'` (the lifetime sigil — same syntax, different meaning).
Labels also work with `continue`:

```rust
'processing: for batch in batches {
    for item in batch {
        if item.is_poison_pill() {
            continue 'processing;  // skip to the next batch
        }
        process(item);
    }
}
```

In Go you have labeled `break` and `continue` with the same semantics. In JS/TS,
labeled `break`/`continue` also exist but are rarely used.

### Your notes
<!-- -->


---

## while

Standard while loop — repeats as long as the condition is true:

```rust
let mut backoff_ms = 100u64;
while backoff_ms < 10_000 {
    if try_connect().is_ok() {
        break;
    }
    sleep(backoff_ms);
    backoff_ms *= 2;
}
```

`while` is equivalent to `loop { if !condition { break; } ... }`. Use `while`
when you have a clear exit condition at the top; use `loop` when the exit
condition is in the middle or you want to break with a value.

### while let

Like `if let`, but loops:

```rust
// Process items from a channel until it's empty
while let Some(job) = job_queue.pop() {
    process(job);
}
```

Under the hood, this desugars to:

```rust
loop {
    match job_queue.pop() {
        Some(job) => process(job),
        None => break,
    }
}
```

`while let` is idiomatic when draining an iterator-like structure or processing
`Option` chains. A common production pattern:

```rust
while let Ok(line) = reader.read_line(&mut buf) {
    if line == 0 { break; }
    // process buf
}
```

### Your notes
<!-- -->


---

## for

Rust's `for` loop is always over an iterator. There is no C-style `for(;;)` loop.
If you need an index, you use `enumerate()`.

```rust
// Range: 0..5 means 0, 1, 2, 3, 4 (exclusive end)
for i in 0..5 {
    println!("{}", i);
}

// Inclusive range: 1..=5 means 1, 2, 3, 4, 5
for i in 1..=5 {
    println!("{}", i);
}
```

### Over Collections

```rust
let endpoints = vec!["api.example.com", "cdn.example.com", "db.example.com"];

// By reference (borrow the vec, iterate over &str)
for ep in &endpoints {
    println!("checking: {}", ep);
}
// endpoints is still usable here

// By value (consumes the vec)
for ep in endpoints {
    process(ep);
}
// endpoints has been moved — cannot use it here

// By mutable reference
let mut counters = vec![0u32; 5];
for c in &mut counters {
    *c += 1;
}
```

This is where Rust diverges sharply from Go and TypeScript:

| | Go | TypeScript | Rust |
|---|---|---|---|
| Over slice by value | `for _, v := range slice` (copies elements) | `for (const v of arr)` | `for v in vec` (moves elements) |
| Over slice by reference | `for _, v := range slice` (always copies for primitives) | N/A | `for v in &vec` |
| By mutable reference | N/A | N/A | `for v in &mut vec` |
| Index + value | `for i, v := range slice` | `for (const [i, v] of arr.entries())` | `for (i, v) in vec.iter().enumerate()` |

In Go, `range` always copies the value (Go values are not "moved"). In Rust,
iterating by value transfers ownership of each element.

### enumerate, zip, and other adapters

```rust
let tasks = vec!["build", "test", "deploy"];

for (i, task) in tasks.iter().enumerate() {
    println!("step {}: {}", i + 1, task);
}

// zip two iterators
let keys = vec!["host", "port", "name"];
let values = vec!["localhost", "8080", "api"];
for (k, v) in keys.iter().zip(values.iter()) {
    println!("{}={}", k, v);
}
```

### for vs iterators

`for` loops and iterator chains are equivalent in most cases, but iterators are
often more expressive:

```rust
// for loop:
let mut results = Vec::new();
for item in &items {
    if item.is_active() {
        results.push(item.name.clone());
    }
}

// Iterator chain (same semantics):
let results: Vec<&str> = items.iter()
    .filter(|item| item.is_active())
    .map(|item| item.name.as_str())
    .collect();
```

Iterators are lazy — nothing executes until `.collect()` or another consuming
adapter is called. We'll cover iterators in depth in their own module.

### Your notes
<!-- -->


---

## match

`match` is Rust's most powerful control flow construct. It is exhaustive — the
compiler requires you to cover every possible case. This is a fundamental
difference from every other language in this curriculum.

```rust
let status: u16 = 500;

let category = match status {
    200..=299 => "success",    // range pattern
    301 | 302 => "redirect",   // or-pattern
    400 => "bad request",
    404 => "not found",
    500..=599 => "server error",
    _ => "unknown",            // catch-all (required if not exhaustive)
};
```

### Exhaustiveness

The compiler checks that all possible values are covered. Remove the `_` arm
above and you get a compile error listing uncovered cases. This is a major safety
property — you can't accidentally forget a case.

```rust
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

fn requires_body(method: &HttpMethod) -> bool {
    match method {
        HttpMethod::Get | HttpMethod::Delete => false,
        HttpMethod::Post | HttpMethod::Put => true,
        // No _ needed — the compiler knows all variants are covered
    }
}
```

Add a new variant to `HttpMethod` and every `match` on it becomes a compile
error until you handle the new case. Compare this to Go's `switch` or
TypeScript's `switch` — adding a new value is silently unhandled.

### Pattern types

**Literal patterns:**
```rust
match response_code {
    0 => "success",
    1 => "partial",
    _ => "error",
}
```

**Struct/tuple destructuring:**
```rust
struct Point { x: i32, y: i32 }
let p = Point { x: 3, y: -2 };

match p {
    Point { x: 0, y: 0 } => println!("origin"),
    Point { x, y: 0 } => println!("on x-axis at {}", x),
    Point { x: 0, y } => println!("on y-axis at {}", y),
    Point { x, y } => println!("at ({}, {})", x, y),
}
```

**Enum variants with data:**
```rust
enum Event {
    Connect { client_id: u32, ip: String },
    Disconnect { client_id: u32 },
    Message { from: u32, body: String },
    Heartbeat,
}

match event {
    Event::Connect { client_id, ip } => {
        println!("client {} connected from {}", client_id, ip);
    }
    Event::Disconnect { client_id } => {
        println!("client {} disconnected", client_id);
    }
    Event::Message { from, body } => {
        println!("[{}]: {}", from, body);
    }
    Event::Heartbeat => {} // nothing to do
}
```

**Bindings with `@`:**

Bind the matched value to a name while also testing it with a pattern:

```rust
match response_code {
    code @ 200..=299 => println!("success: {}", code),
    code @ 400..=499 => println!("client error: {}", code),
    code => println!("other: {}", code),
}
```

The `@` lets you both test (`200..=299`) and name (`code`) the matched value.

**Guards:**

Add a condition after the pattern with `if`:

```rust
match job {
    Job { priority, .. } if priority > 8 => process_urgent(job),
    Job { retry_count, .. } if retry_count >= 3 => discard(job),
    _ => enqueue(job),
}
```

Guards are evaluated after the pattern matches. A guard failure means the arm
is skipped and the next arm is tried. This is different from putting a check
inside the arm body — guards participate in pattern matching order.

**Nested patterns:**
```rust
match config {
    Config { server: Some(ServerConfig { port: 443, .. }), .. } => {
        println!("HTTPS server");
    }
    Config { server: Some(ServerConfig { port, .. }), .. } => {
        println!("HTTP server on port {}", port);
    }
    Config { server: None, .. } => {
        println!("no server configured");
    }
}
```

**Tuple matching:**
```rust
match (auth_valid, rate_limit_ok) {
    (true, true) => handle_request(),
    (false, _) => return Err("unauthorized"),
    (true, false) => return Err("rate limited"),
}
```

### Match arm order matters

Patterns are tried top-to-bottom and the first match wins. More specific patterns
must come before more general ones:

```rust
match value {
    0 => println!("zero"),
    1..=9 => println!("single digit"),
    n if n < 0 => println!("negative"),  // guard
    _ => println!("large"),
}
```

If you put `_` first, it would match everything and none of the other arms would
be reachable — the compiler warns about this.

### match vs Go switch vs TS switch

| Feature | Rust match | Go switch | TypeScript switch |
|---|---|---|---|
| Exhaustiveness | Compiler-enforced | No | No |
| Fall-through | Never | No (by default) | Yes (`break` needed) |
| Patterns | Rich (destructuring, guards, bindings) | Expression equality | Expression equality |
| Returns a value | Yes | No | No |
| Enum matching | Full variant + data extraction | No | Discriminated unions (type narrowing) |

### Your notes
<!-- -->


---

## Early Returns and the ? Operator

### Early returns with `return`

Functions in Rust implicitly return the last expression. Use `return` for early
exit:

```rust
fn validate_config(config: &Config) -> Result<(), String> {
    if config.port == 0 {
        return Err("port cannot be 0".to_string());
    }
    if config.host.is_empty() {
        return Err("host cannot be empty".to_string());
    }
    Ok(())
}
```

### The ? operator (preview)

`?` is syntactic sugar for "return the error if this is Err, otherwise unwrap
the Ok value." It's used in functions that return `Result`:

```rust
fn load_config(path: &str) -> Result<Config, String> {
    let contents = read_file(path)?;     // returns Err early if file read fails
    let config = parse_config(&contents)?; // returns Err early if parse fails
    validate_config(&config)?;
    Ok(config)
}
```

Without `?`, each line would need:

```rust
let contents = match read_file(path) {
    Ok(c) => c,
    Err(e) => return Err(e),
};
```

`?` is control flow — it's essentially an early return with a match on `Result`.
The full error handling module covers this in depth.

### Your notes
<!-- -->


---

## Comparison Summary

### Go vs Rust Control Flow

Go has only `for` (no `while`, `loop`) and `switch`. Rust has dedicated `loop`,
`while`, `for`, and `match`. The semantics differ significantly:

| Concept | Go | Rust |
|---|---|---|
| Infinite loop | `for { }` | `loop { }` |
| While condition | `for condition { }` | `while condition { }` |
| C-style for | `for i := 0; i < n; i++` | Not available — use `0..n` |
| For over slice | `for i, v := range slice` | `for v in &slice` or `for (i, v) in slice.iter().enumerate()` |
| Switch/match | `switch value { case x: ... }` | `match value { x => ... }` |
| Switch exhaustiveness | Not enforced | Compiler-enforced |
| If as expression | No | Yes |
| If with initializer | `if err := f(); err != nil` | `if let Ok(v) = f()` (different but analogous) |

### TypeScript vs Rust Control Flow

| Concept | TypeScript | Rust |
|---|---|---|
| Ternary | `x > 0 ? "pos" : "neg"` | Not available — use `if x > 0 { "pos" } else { "neg" }` |
| Switch exhaustiveness | Not enforced (can use never type tricks) | Compiler-enforced |
| Switch fall-through | Yes (use `break`) | Never |
| for-of | `for (const v of iterable)` | `for v in iterable` |
| for-in | `for (const k in obj)` | Not available (HashMap: use `.keys()`) |
| Optional chaining | `obj?.field` | `if let Some(v) = opt` or `.as_ref().map(...)` |

### Your notes
<!-- -->
