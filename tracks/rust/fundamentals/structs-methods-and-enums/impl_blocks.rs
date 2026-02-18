// impl Blocks — Rust
//
// Demonstrates: methods vs associated functions, self/&self/&mut self,
// multiple impl blocks, method chaining, converting between types.
//
// Run: rustc impl_blocks.rs && ./impl_blocks

// ---- A type with the full receiver spectrum ----

#[derive(Debug, Clone)]
struct RateLimiter {
    name: String,
    max_requests: u32,
    window_ms: u64,
    current_count: u32,
    window_start: u64,
}

// First impl block: construction and configuration
impl RateLimiter {
    // Associated function (no self). This is the canonical constructor.
    fn new(name: &str, max_requests: u32, window_ms: u64) -> RateLimiter {
        RateLimiter {
            name: name.to_string(),
            max_requests,
            window_ms,
            current_count: 0,
            window_start: 0,
        }
    }

    // Associated function: named constructor for common preset
    fn per_minute(name: &str, max_requests: u32) -> RateLimiter {
        RateLimiter::new(name, max_requests, 60_000)
    }

    // Associated function: named constructor for burst limiting
    fn burst(name: &str, max_requests: u32) -> RateLimiter {
        RateLimiter::new(name, max_requests, 1_000)
    }

    // self (consuming): returns a modified version. Enables chaining.
    // After calling this, the original binding is gone.
    fn with_max(mut self, max: u32) -> RateLimiter {
        self.max_requests = max;
        self
    }

    fn with_window(mut self, ms: u64) -> RateLimiter {
        self.window_ms = ms;
        self
    }
}

// Second impl block: runtime behavior
impl RateLimiter {
    // &mut self: mutates internal counters. Caller must hold `let mut`.
    fn allow(&mut self, now: u64) -> bool {
        if now >= self.window_start + self.window_ms {
            // New window: reset counter
            self.window_start = now;
            self.current_count = 0;
        }

        if self.current_count < self.max_requests {
            self.current_count += 1;
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.current_count = 0;
        self.window_start = 0;
    }

    // &self: read-only. Does not require mut binding to call.
    fn remaining(&self, now: u64) -> u32 {
        if now >= self.window_start + self.window_ms {
            return self.max_requests;  // window expired: full capacity
        }
        self.max_requests.saturating_sub(self.current_count)
    }

    fn is_exhausted(&self, now: u64) -> bool {
        self.remaining(now) == 0
    }

    fn description(&self) -> String {
        format!(
            "RateLimiter({}: {}/{}ms)",
            self.name, self.max_requests, self.window_ms
        )
    }
}

// Third impl block: conversions (consuming self to return a different type)
impl RateLimiter {
    // self: consumed; produces a snapshot of current state as a plain struct
    fn into_snapshot(self, now: u64) -> LimiterSnapshot {
        // Compute derived values before moving fields out of self.
        let remaining = self.remaining(now);
        let exhausted = self.is_exhausted(now);
        LimiterSnapshot {
            name: self.name,
            remaining,
            exhausted,
        }
    }
}

#[derive(Debug)]
struct LimiterSnapshot {
    name: String,
    remaining: u32,
    exhausted: bool,
}

// ---- Method chaining with consuming self ----

// A pipeline builder for a data processing job.
#[derive(Debug)]
struct PipelineBuilder {
    name: String,
    sources: Vec<String>,
    transforms: Vec<String>,
    sink: Option<String>,
    max_workers: u32,
}

impl PipelineBuilder {
    fn new(name: &str) -> PipelineBuilder {
        PipelineBuilder {
            name: name.to_string(),
            sources: Vec::new(),
            transforms: Vec::new(),
            sink: None,
            max_workers: 4,
        }
    }

    fn source(mut self, s: &str) -> PipelineBuilder {
        self.sources.push(s.to_string());
        self
    }

    fn transform(mut self, t: &str) -> PipelineBuilder {
        self.transforms.push(t.to_string());
        self
    }

    fn sink(mut self, s: &str) -> PipelineBuilder {
        self.sink = Some(s.to_string());
        self
    }

    fn workers(mut self, n: u32) -> PipelineBuilder {
        self.max_workers = n;
        self
    }

    // Terminal: consumes the builder and produces the final object.
    // After this call, the builder is gone.
    fn build(self) -> Result<Pipeline, String> {
        if self.sources.is_empty() {
            return Err(String::from("pipeline requires at least one source"));
        }
        let sink = self.sink.ok_or_else(|| String::from("pipeline requires a sink"))?;

        Ok(Pipeline {
            name: self.name,
            sources: self.sources,
            transforms: self.transforms,
            sink,
            max_workers: self.max_workers,
        })
    }
}

#[derive(Debug)]
struct Pipeline {
    name: String,
    sources: Vec<String>,
    transforms: Vec<String>,
    sink: String,
    max_workers: u32,
}

impl Pipeline {
    fn describe(&self) {
        println!("Pipeline '{}':", self.name);
        println!("  sources    : {:?}", self.sources);
        println!("  transforms : {:?}", self.transforms);
        println!("  sink       : {}", self.sink);
        println!("  workers    : {}", self.max_workers);
    }
}

// ---- self vs &self: why it matters ----

// This struct deliberately shows when each receiver is appropriate.
#[derive(Debug)]
struct Buffer {
    data: Vec<u8>,
    capacity: usize,
}

impl Buffer {
    fn with_capacity(cap: usize) -> Buffer {
        Buffer {
            data: Vec::with_capacity(cap),
            capacity: cap,
        }
    }

    // &mut self — must mutate
    fn push(&mut self, byte: u8) -> bool {
        if self.data.len() < self.capacity {
            self.data.push(byte);
            true
        } else {
            false
        }
    }

    // &self — read only
    fn len(&self) -> usize { self.data.len() }
    fn is_full(&self) -> bool { self.data.len() >= self.capacity }
    fn as_slice(&self) -> &[u8] { &self.data }

    // self — consuming. After calling drain(), this Buffer is gone.
    // The caller receives the Vec; the Buffer is dropped.
    fn drain(self) -> Vec<u8> {
        self.data
    }

    // self — consuming for type conversion
    fn into_string_lossy(self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }
}

fn main() {
    // ----- RateLimiter: multiple impl blocks -----
    println!("=== RateLimiter ===");

    // Associated function constructors
    let mut limiter = RateLimiter::per_minute("api:POST /checkout", 100);
    println!("{}", limiter.description());

    // &self reads
    println!("remaining at t=0: {}", limiter.remaining(0));

    // &mut self writes
    let t = 1000u64;
    for _ in 0..5 {
        limiter.allow(t);
    }
    println!("remaining after 5 calls: {}", limiter.remaining(t));
    println!("exhausted? {}", limiter.is_exhausted(t));

    // self-consuming builder syntax
    let custom = RateLimiter::new("custom", 10, 5000)
        .with_max(20)
        .with_window(10_000);
    println!("custom: {}", custom.description());

    // Consuming conversion
    let mut burst = RateLimiter::burst("login", 5);
    burst.allow(0);
    burst.allow(0);
    burst.allow(0);
    let snap = burst.into_snapshot(0);  // burst is moved here
    println!("snapshot: {:?}", snap);
    // burst.allow(0);  // compile error: value moved

    // ----- PipelineBuilder: chaining -----
    println!("\n=== PipelineBuilder ===");

    let pipeline = PipelineBuilder::new("user-events")
        .source("kafka://events.user.created")
        .source("kafka://events.user.updated")
        .transform("deduplicate")
        .transform("enrich-from-crm")
        .transform("normalize-timestamps")
        .sink("s3://datalake/users/")
        .workers(8)
        .build();

    match pipeline {
        Ok(p) => p.describe(),
        Err(e) => println!("build error: {}", e),
    }

    // Missing sink — should return Err
    let bad = PipelineBuilder::new("incomplete")
        .source("kafka://events")
        .build();
    println!("\nbad pipeline: {:?}", bad);

    // ----- Buffer: self vs &self vs &mut self -----
    println!("\n=== Buffer Receivers ===");

    let mut buf = Buffer::with_capacity(4);
    println!("empty: len={} full={}", buf.len(), buf.is_full());

    buf.push(b'h');
    buf.push(b'i');
    println!("after push: len={} full={}", buf.len(), buf.is_full());
    println!("slice: {:?}", buf.as_slice());

    // into_string_lossy consumes the Buffer
    let s = buf.into_string_lossy();
    println!("consumed as string: {:?}", s);
    // buf.len();  // compile error: buf was moved

    // drain is an alternative consuming operation
    let mut buf2 = Buffer::with_capacity(3);
    buf2.push(1); buf2.push(2); buf2.push(3);
    let raw = buf2.drain();  // buf2 moved
    println!("drained: {:?}", raw);
}
