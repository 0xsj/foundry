# Strategy Pattern — Rust

## The Problem Strategy Solves

Every non-trivial system eventually needs to do the same thing in different ways. A notification service needs to send via email, SMS, or webhook. A data pipeline needs to compress with gzip, zstd, or lz4. A rate limiter needs to enforce limits using a token bucket, sliding window, or fixed window algorithm.

The naive approach is an `if`/`match` chain:

```rust
fn send_notification(channel: &str, message: &str) {
    match channel {
        "email" => { /* 30 lines of SMTP logic */ }
        "sms" => { /* 30 lines of Twilio logic */ }
        "webhook" => { /* 30 lines of HTTP POST logic */ }
        _ => panic!("unknown channel"),
    }
}
```

This breaks down fast. Every new channel requires modifying this function. Testing means running the whole thing. The function knows about every transport detail. The Strategy pattern fixes this by **extracting each algorithm into its own type** and making them interchangeable behind a shared interface.

In Rust, you have three distinct ways to implement this, and choosing the right one is a genuine architectural decision — not just syntax preference.

### Your notes
<!-- -->


---

## The Three Approaches in Rust

Rust gives you three mechanisms for the strategy pattern, each with different tradeoffs:

| Approach | Mechanism | Dispatch | Known at compile time? | Allocation | Best for |
|----------|-----------|----------|------------------------|------------|----------|
| **Generics** | `fn process<S: Strategy>(s: &S)` | Static (monomorphization) | Yes | Stack | Hot paths, known strategy set |
| **Trait objects** | `fn process(s: &dyn Strategy)` | Dynamic (vtable) | No | Heap (usually `Box`) | Plugin systems, runtime config |
| **Closures** | `fn process(s: impl Fn(Args) -> Ret)` | Static or dynamic | Depends | Varies | Simple one-off behaviors |

Let's build each one from a real scenario.

### Approach 1: Trait Objects — Dynamic Dispatch

This is the closest to how Go interfaces and TypeScript class hierarchies work. You define a trait, implement it for multiple types, and pass around `Box<dyn Trait>` or `&dyn Trait`.

```rust
use std::collections::HashMap;

trait NotificationStrategy {
    fn send(&self, recipient: &str, message: &str) -> Result<(), String>;
    fn name(&self) -> &str;
}

struct EmailNotifier {
    smtp_host: String,
}

impl NotificationStrategy for EmailNotifier {
    fn send(&self, recipient: &str, message: &str) -> Result<(), String> {
        println!("[EMAIL via {}] To: {} — {}", self.smtp_host, recipient, message);
        Ok(())
    }

    fn name(&self) -> &str {
        "email"
    }
}

struct WebhookNotifier {
    endpoint: String,
    secret: String,
}

impl NotificationStrategy for WebhookNotifier {
    fn send(&self, recipient: &str, message: &str) -> Result<(), String> {
        println!(
            "[WEBHOOK POST {}] Recipient: {} — {} (signed with {})",
            self.endpoint, recipient, message, &self.secret[..4]
        );
        Ok(())
    }

    fn name(&self) -> &str {
        "webhook"
    }
}

// The dispatcher holds strategies as trait objects
struct NotificationService {
    strategies: HashMap<String, Box<dyn NotificationStrategy>>,
}

impl NotificationService {
    fn new() -> Self {
        Self {
            strategies: HashMap::new(),
        }
    }

    fn register(&mut self, strategy: Box<dyn NotificationStrategy>) {
        self.strategies.insert(strategy.name().to_string(), strategy);
    }

    fn notify(&self, channel: &str, recipient: &str, msg: &str) -> Result<(), String> {
        let strategy = self.strategies.get(channel)
            .ok_or_else(|| format!("no strategy registered for '{}'", channel))?;
        strategy.send(recipient, msg)
    }
}
```

**When to use this:** You don't know the full set of strategies at compile time. Think plugin architectures, user-configured pipelines, or any time strategies are loaded from config/database.

**What it costs:** Each call goes through a vtable (pointer indirection). The strategies live on the heap via `Box`. The compiler can't inline across the trait boundary.

> **Coming from Go?** This is almost identical to Go's interface approach. Go's `interface{}` is always dynamically dispatched — there's no static alternative. In Rust, you opt into dynamic dispatch explicitly with `dyn`.

> **Coming from TypeScript?** This is like passing a class instance that implements an interface. The difference is Rust doesn't have inheritance — each implementor is a standalone struct, not a subclass.

### Approach 2: Generics — Static Dispatch

When you know the strategy at compile time, generics give you zero-cost abstraction. The compiler generates specialized code for each concrete type (monomorphization).

```rust
trait CompressionStrategy {
    fn compress(&self, data: &[u8]) -> Vec<u8>;
    fn name(&self) -> &str;
}

struct GzipCompression {
    level: u32,
}

impl CompressionStrategy for GzipCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        // In real code: use flate2 crate
        let mut result = format!("gzip-L{}:", self.level).into_bytes();
        result.extend_from_slice(data);
        result
    }

    fn name(&self) -> &str {
        "gzip"
    }
}

struct ZstdCompression {
    level: i32,
}

impl CompressionStrategy for ZstdCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        let mut result = format!("zstd-L{}:", self.level).into_bytes();
        result.extend_from_slice(data);
        result
    }

    fn name(&self) -> &str {
        "zstd"
    }
}

// Generic: the strategy type is baked into the pipeline at compile time
struct DataPipeline<C: CompressionStrategy> {
    compressor: C,
    buffer: Vec<Vec<u8>>,
}

impl<C: CompressionStrategy> DataPipeline<C> {
    fn new(compressor: C) -> Self {
        Self {
            compressor,
            buffer: Vec::new(),
        }
    }

    fn ingest(&mut self, data: &[u8]) {
        let compressed = self.compressor.compress(data);
        self.buffer.push(compressed);
    }

    fn flush(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.buffer)
    }
}

fn main() {
    // The compiler generates TWO versions of DataPipeline: one for Gzip, one for Zstd
    let mut gzip_pipeline = DataPipeline::new(GzipCompression { level: 6 });
    let mut zstd_pipeline = DataPipeline::new(ZstdCompression { level: 3 });

    gzip_pipeline.ingest(b"hello world");
    zstd_pipeline.ingest(b"hello world");

    println!("{:?}", String::from_utf8_lossy(&gzip_pipeline.flush()[0]));
    println!("{:?}", String::from_utf8_lossy(&zstd_pipeline.flush()[0]));
}
```

**When to use this:** The strategy is chosen at compile time or at initialization and doesn't change. High-performance paths where inlining matters. Think serialization formats, hash algorithms, allocator strategies.

**What it costs:** Binary size increases — the compiler generates a separate copy of `DataPipeline` for each strategy type. You can't mix different strategy types in the same collection (`Vec<DataPipeline<??>>` doesn't work without trait objects).

**The key insight:** `DataPipeline<GzipCompression>` and `DataPipeline<ZstdCompression>` are **different types**. You can't put them in the same `Vec` or swap them at runtime. If you need that, you need trait objects.

### Approach 3: Closures — Functional Strategy

For simple strategies that are just "a function with some behavior," closures are the lightest-weight option. This is closer to how you'd do it in JavaScript.

```rust
use std::time::Duration;

// The "strategy" is just a function signature
type RetryPolicy = Box<dyn Fn(u32) -> Option<Duration>>;

fn exponential_backoff(base_ms: u64, max_retries: u32) -> RetryPolicy {
    Box::new(move |attempt| {
        if attempt >= max_retries {
            return None;
        }
        let delay = base_ms * 2u64.pow(attempt);
        Some(Duration::from_millis(delay))
    })
}

fn constant_delay(ms: u64, max_retries: u32) -> RetryPolicy {
    Box::new(move |attempt| {
        if attempt >= max_retries {
            return None;
        }
        Some(Duration::from_millis(ms))
    })
}

fn no_retry() -> RetryPolicy {
    Box::new(|_| None)
}

struct HttpClient {
    retry_policy: RetryPolicy,
}

impl HttpClient {
    fn new(retry_policy: RetryPolicy) -> Self {
        Self { retry_policy }
    }

    fn request(&self, url: &str) -> Result<String, String> {
        let mut attempt = 0;
        loop {
            println!("Attempt {} for {}", attempt + 1, url);
            // Simulate: fail first 2 attempts
            if attempt >= 2 {
                return Ok(format!("Response from {}", url));
            }

            match (self.retry_policy)(attempt) {
                Some(delay) => {
                    println!("  Retrying after {:?}", delay);
                    // In real code: std::thread::sleep(delay);
                    attempt += 1;
                }
                None => return Err(format!("Max retries exceeded for {}", url)),
            }
        }
    }
}
```

**When to use this:** The strategy is a single behavior (one method). You want to define strategies inline without creating structs. Think sort comparators, retry policies, validation rules, event handlers.

**What it costs:** Closures that capture state need `Box<dyn Fn>` for storage (heap allocation + dynamic dispatch). If the closure doesn't capture anything, it's zero-cost. You lose the ability to name the strategy or attach metadata.

> **Coming from JavaScript?** This is exactly how you'd do it in JS — pass a function. The difference is Rust's closure traits (`Fn`, `FnMut`, `FnOnce`) tell you exactly what the closure can do with its captured state.

### Your notes
<!-- -->


---

## Under the Hood: Monomorphization vs Vtable Dispatch

Understanding what the compiler does with each approach changes how you think about the tradeoff.

### Monomorphization (Generics / Static Dispatch)

When you write `DataPipeline<C: CompressionStrategy>`, the compiler generates a **separate, fully specialized copy** of `DataPipeline` for each concrete type you use:

```
// What you write:
DataPipeline<GzipCompression>
DataPipeline<ZstdCompression>

// What the compiler generates (conceptually):
struct DataPipeline_GzipCompression {
    compressor: GzipCompression,
    buffer: Vec<Vec<u8>>,
}

impl DataPipeline_GzipCompression {
    fn ingest(&mut self, data: &[u8]) {
        // GzipCompression::compress is INLINED here
        let compressed = /* gzip-specific code directly embedded */;
        self.buffer.push(compressed);
    }
}

struct DataPipeline_ZstdCompression {
    compressor: ZstdCompression,
    buffer: Vec<Vec<u8>>,
}

impl DataPipeline_ZstdCompression {
    fn ingest(&mut self, data: &[u8]) {
        // ZstdCompression::compress is INLINED here
        let compressed = /* zstd-specific code directly embedded */;
        self.buffer.push(compressed);
    }
}
```

**Pros:**
- The compiler can inline the strategy's code into the caller
- No pointer indirection, no vtable lookup
- CPU branch prediction is happy — no indirect jumps
- Can optimize across the call boundary (constant folding, dead code elimination)

**Cons:**
- Binary size grows with each specialization
- Compile times increase (more code to optimize)
- Can't be used for runtime-determined strategies
- "Type explosion" — each combination is a distinct type

### Vtable Dispatch (Trait Objects / Dynamic Dispatch)

When you write `Box<dyn NotificationStrategy>`, the compiler creates a **vtable** — a table of function pointers:

```
// The vtable for EmailNotifier as NotificationStrategy:
[
    ptr_to_EmailNotifier::send,     // slot 0
    ptr_to_EmailNotifier::name,     // slot 1
    ptr_to_EmailNotifier::drop,     // destructor
    size_of::<EmailNotifier>(),     // size
    align_of::<EmailNotifier>(),    // alignment
]

// Box<dyn NotificationStrategy> is actually a "fat pointer":
// [pointer_to_data, pointer_to_vtable]
```

When you call `strategy.send(recipient, msg)`, the runtime:
1. Follows the vtable pointer
2. Looks up slot 0 (the `send` method)
3. Calls through the function pointer, passing the data pointer as `&self`

**Pros:**
- Single copy of code, small binary
- Strategies can be swapped at runtime
- Heterogeneous collections: `Vec<Box<dyn Strategy>>` works
- New strategies can be added without recompilation (plugin architectures)

**Cons:**
- Each call has pointer indirection (usually ~1-3ns overhead)
- Compiler cannot inline across the boundary
- CPU branch prediction struggles with indirect calls
- Heap allocation for `Box<dyn Trait>`

### When Does the Difference Matter?

In most business logic, **it doesn't**. The vtable overhead is noise compared to network calls, database queries, or disk I/O. Choose based on ergonomics.

It matters when:
- The strategy is called millions of times per second (inner loops of data processing)
- The strategy is trivially small (a comparator in a sort)
- You're building a library where users expect zero-cost abstractions
- You're in a `#[no_std]` environment where heap allocation isn't available

**Rule of thumb:** Start with trait objects for flexibility. Profile. Switch to generics only where the dispatch overhead shows up in benchmarks.

### Your notes
<!-- -->


---

## Object Safety: The Trait Object Gotcha

Not every trait can be made into a `dyn Trait`. The rules for "object safety" trip up even experienced Rust developers.

A trait is **object safe** if:
1. It does not have `Self: Sized` as a supertrait
2. All methods either:
   - Have a receiver (`&self`, `&mut self`, `self`, `Box<Self>`, etc.)
   - OR are explicitly opted out with `where Self: Sized`
3. No method has generic type parameters
4. No method returns `Self` (specifically `-> Self`, not `-> Box<Self>`)

```rust
// NOT object safe — generic method
trait BadStrategy {
    fn process<T: std::fmt::Debug>(&self, item: T);
    // Error: "the trait `BadStrategy` cannot be made into an object"
    // because `process` has a generic parameter `T`
}

// NOT object safe — returns Self
trait BadClone {
    fn duplicate(&self) -> Self;
    // Error: the compiler doesn't know the size of Self behind a dyn pointer
}

// Object safe — fixed types only
trait GoodStrategy {
    fn process(&self, item: &str) -> Result<Vec<u8>, String>;
}

// Object safe — generic method opted out
trait MostlyDynamic {
    fn process(&self, item: &str) -> Result<Vec<u8>, String>;

    // This method won't be available through &dyn MostlyDynamic,
    // but the trait is still object safe
    fn process_generic<T: std::fmt::Debug>(&self, item: T)
    where
        Self: Sized;
}
```

> **Coming from Go?** Go interfaces are always object safe by design — interface methods can't be generic. Rust gives you more power (generic trait methods) at the cost of this extra complexity.

### Your notes
<!-- -->


---

## Real-World Strategy Pattern in the Standard Library

Rust's standard library uses strategy patterns extensively. Recognizing them helps you write idiomatic code.

### `Iterator::sort_by` — Closure Strategy

```rust
let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6];

// The comparison function IS the strategy
data.sort_by(|a, b| b.cmp(a));  // descending

// With a key extraction strategy
data.sort_by_key(|x| std::cmp::Reverse(*x));
```

### `std::io::Read` / `std::io::Write` — Trait Object Strategy

```rust
use std::io::{self, Read, Write};

// This function accepts ANY reader — file, network, in-memory buffer
fn count_lines(reader: &mut dyn Read) -> io::Result<usize> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    Ok(buf.lines().count())
}

// This function accepts ANY writer
fn write_report(writer: &mut dyn Write, data: &[(&str, u64)]) -> io::Result<()> {
    for (label, value) in data {
        writeln!(writer, "{}: {}", label, value)?;
    }
    Ok(())
}
```

### `HashMap` — Generic Strategy (Hasher)

```rust
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

// HashMap's third type parameter is the hashing strategy
// Default is RandomState (SipHash) — secure but not the fastest
let default_map: HashMap<String, u64> = HashMap::new();

// You can swap in a faster hasher for non-adversarial data:
// use std::collections::hash_map::DefaultHasher;
// type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;
```

### `serde` — Trait-Based Serialization Strategy

The `serde` crate is arguably the most famous strategy pattern in the Rust ecosystem. `Serializer` and `Deserializer` are traits — JSON, TOML, YAML, MessagePack, and dozens of other formats each provide their own implementation:

```rust
// serde::Serialize is the "context" — it accepts any Serializer strategy
// #[derive(Serialize)]
// struct Config { name: String, port: u16 }
//
// let json = serde_json::to_string(&config)?;   // JSON strategy
// let toml = toml::to_string(&config)?;         // TOML strategy
// let yaml = serde_yaml::to_string(&config)?;   // YAML strategy
```

### Your notes
<!-- -->


---

## Cross-Language Comparison

| Aspect | Rust | Go | TypeScript |
|--------|------|----|------------|
| **Interface** | `trait Strategy { ... }` | `type Strategy interface { ... }` | `interface Strategy { ... }` or abstract class |
| **Dynamic dispatch** | `Box<dyn Strategy>`, `&dyn Strategy` | Always (interfaces are always dynamic) | Always (class instances, no static dispatch) |
| **Static dispatch** | `fn foo<S: Strategy>(s: S)` | Not available | Not available |
| **Closure strategy** | `Fn(Args) -> Ret` traits | `func(args) ret` first-class functions | `(args) => ret` arrow functions |
| **Object safety** | Explicit rules, compiler-enforced | Always safe (no generics in interfaces) | Not a concept |
| **Storage** | `Box<dyn>` (heap), generics (stack) | Interface values (heap escape analysis) | Garbage collected |
| **Performance control** | Full control over dispatch | No control | No control |
| **Adding strategies** | New struct + `impl Trait` | New struct + methods | New class `implements Interface` |

### Key Differences

**Rust vs Go:** Go gives you one dispatch mechanism (interfaces, always dynamic). Rust makes you choose — but that choice gives you zero-cost generics. Go's simplicity is a feature when dispatch cost doesn't matter. Rust's complexity is a feature when it does.

**Rust vs TypeScript:** TypeScript's strategies are always heap-allocated, garbage-collected objects. There's no way to express "this strategy is resolved at compile time." Rust's generic approach lets the compiler eliminate all abstraction overhead. On the other hand, TypeScript's `type` system lets you express union-type strategies elegantly — something Rust handles differently with enums.

**The enum alternative:** Rust has a fourth option that Go and TypeScript don't — `enum`-based dispatch:

```rust
enum CompressionKind {
    Gzip { level: u32 },
    Zstd { level: i32 },
    None,
}

impl CompressionKind {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        match self {
            CompressionKind::Gzip { level } => { /* ... */ todo!() }
            CompressionKind::Zstd { level } => { /* ... */ todo!() }
            CompressionKind::None => data.to_vec(),
        }
    }
}
```

This is **not** the strategy pattern — it's a closed set that requires modifying the enum to add variants. But it's faster (no indirection, no heap allocation, enum fits on the stack) and appropriate when the set of strategies is known and fixed. The compiler even warns you if you forget a variant in a `match`.

### Your notes
<!-- -->


---

## Decision Framework: Choosing Your Approach

Use this flowchart when deciding how to implement strategy in Rust:

1. **Is the strategy a single function?** (no state, no multiple methods)
   - Yes -> Use a closure (`Fn`, `FnMut`, `FnOnce`)
   - No -> Continue

2. **Is the set of strategies known and closed?** (you control all variants)
   - Yes, and performance matters -> Consider an `enum` with `match`
   - Yes, but extensibility matters -> Continue
   - No -> Continue

3. **Is the strategy determined at compile time?** (won't change at runtime)
   - Yes -> Use generics (`<S: Strategy>`)
   - No -> Use trait objects (`Box<dyn Strategy>` or `&dyn Strategy`)

4. **Do you need heterogeneous collections?** (`Vec` of mixed strategies)
   - Yes -> You must use trait objects
   - No -> Generics are fine

5. **Is this in a hot loop called millions of times?**
   - Yes -> Prefer generics or enums (measure first)
   - No -> Trait objects are fine

### Your notes
<!-- -->


---

## Preview: Related Patterns

The Strategy pattern connects to several other patterns you'll encounter:

- **Command pattern** (covered later): Similar structure, but Command encapsulates an action to execute later (with undo), while Strategy encapsulates an algorithm to use now.
- **State pattern** (covered later): Uses the same trait/struct mechanism but models state transitions, not interchangeable algorithms.
- **Builder pattern**: Often uses strategies to configure complex objects — "use this serialization strategy, this compression strategy, this retry strategy."
- **Dependency injection**: Strategy is a form of DI — you inject the algorithm rather than hardcoding it. In Rust, this is done through generics or trait objects rather than a DI framework.

### Your notes
<!-- -->
