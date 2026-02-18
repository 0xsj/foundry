// generic_types.rs — Generic structs, enums, methods, PhantomData
//
// Run: rustc generic_types.rs && ./generic_types

use std::fmt::Display;
use std::marker::PhantomData;

// ---------- 1. Generic struct: a typed result cache entry ----------

// The struct stores both the input key and the computed output.
// No bounds on the struct definition — bounds only appear where behavior is used.
struct CacheEntry<K, V> {
    key: K,
    value: V,
    hit_count: u32,
}

impl<K, V> CacheEntry<K, V> {
    fn new(key: K, value: V) -> Self {
        CacheEntry { key, value, hit_count: 0 }
    }

    fn get(&mut self) -> &V {
        self.hit_count += 1;
        &self.value
    }
}

// Add a method only when both K and V implement Display.
// Conditional impl blocks — only available for types satisfying the bounds.
impl<K: Display, V: Display> CacheEntry<K, V> {
    fn describe(&self) -> String {
        format!("key={} value={} hits={}", self.key, self.value, self.hit_count)
    }
}

// ---------- 2. Generic enum ----------

// A value that's either ready or pending computation.
// More descriptive than bool, more semantic than Option.
enum ComputedValue<T> {
    Ready(T),
    Pending { task_id: u64, estimated_ms: u32 },
}

impl<T: Display> ComputedValue<T> {
    fn describe(&self) -> String {
        match self {
            ComputedValue::Ready(v) => format!("ready: {}", v),
            ComputedValue::Pending { task_id, estimated_ms } =>
                format!("pending task={} (~{}ms)", task_id, estimated_ms),
        }
    }

    fn unwrap_or(self, default: T) -> T {
        match self {
            ComputedValue::Ready(v) => v,
            ComputedValue::Pending { .. } => default,
        }
    }
}

// ---------- 3. PhantomData: type-safe identifiers ----------

// The problem: UserId(42) and ProductId(42) have the same runtime representation.
// Without type safety, you can pass one where the other is expected.
// PhantomData<Entity> makes them distinct types with zero overhead.

struct Id<Entity> {
    value: u64,
    _marker: PhantomData<Entity>,  // zero-sized; only affects type checker
}

// These are marker types — they are never instantiated, only used as type parameters.
struct User;
struct Product;
struct Order;

// Type aliases for ergonomic use
type UserId    = Id<User>;
type ProductId = Id<Product>;
type OrderId   = Id<Order>;

impl<Entity> Id<Entity> {
    fn new(value: u64) -> Self {
        Id { value, _marker: PhantomData }
    }

    fn value(&self) -> u64 {
        self.value
    }
}

// Different Entity types mean these are different functions — type safety enforced at compile time.
fn find_user_name(id: UserId) -> String {
    format!("user-{}", id.value())
}

fn find_product_name(id: ProductId) -> String {
    format!("product-{}", id.value())
}

// ---------- 4. Generic struct with bounds on methods ----------

// A min-heap priority queue (simplified, using Vec for demonstration).
struct PriorityQueue<T> {
    items: Vec<T>,
}

impl<T> PriorityQueue<T> {
    fn new() -> Self {
        PriorityQueue { items: Vec::new() }
    }

    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.items.len()
    }
}

// The Ord bound is only needed for operations that compare elements.
// Splitting into multiple impl blocks is idiomatic.
impl<T: Ord> PriorityQueue<T> {
    fn push(&mut self, item: T) {
        self.items.push(item);
        // Sort descending — largest at position 0 for O(1) peek/pop.
        // A real heap would use bubble-up; this keeps the example readable.
        self.items.sort_by(|a, b| b.cmp(a));
    }

    fn pop(&mut self) -> Option<T> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items.remove(0))
        }
    }

    fn peek(&self) -> Option<&T> {
        self.items.first()
    }
}

// ---------- 5. Generic struct implementing a trait ----------

// A wrapper that adds retry logic to anything that can be "retried".
// The trait itself is generic — it expresses what "retry" means for T.
trait Retryable {
    type Output;
    type Error: Display;
    fn attempt(&mut self) -> Result<Self::Output, Self::Error>;
}

struct WithRetry<T: Retryable> {
    inner: T,
    max_attempts: u32,
}

impl<T: Retryable> WithRetry<T> {
    fn new(inner: T, max_attempts: u32) -> Self {
        WithRetry { inner, max_attempts }
    }

    fn run(&mut self) -> Result<T::Output, String> {
        for attempt in 1..=self.max_attempts {
            match self.inner.attempt() {
                Ok(output) => return Ok(output),
                Err(e) => {
                    if attempt == self.max_attempts {
                        return Err(format!("failed after {} attempts: {}", attempt, e));
                    }
                    println!("attempt {} failed: {}", attempt, e);
                }
            }
        }
        unreachable!()
    }
}

// Concrete implementation of Retryable for testing
struct FlakySensor {
    calls: u32,
    succeeds_on: u32,
}

impl Retryable for FlakySensor {
    type Output = f64;
    type Error = String;

    fn attempt(&mut self) -> Result<f64, String> {
        self.calls += 1;
        if self.calls >= self.succeeds_on {
            Ok(98.6)
        } else {
            Err(format!("sensor not ready (call {})", self.calls))
        }
    }
}

// ---------- Main ----------

fn main() {
    // 1. Generic struct: CacheEntry
    let mut entry: CacheEntry<String, Vec<u8>> = CacheEntry::new(
        String::from("session:abc123"),
        vec![0x01, 0x02, 0x03],
    );
    let _ = entry.get();
    let _ = entry.get();

    // describe() only available when K: Display, V: Display
    let mut string_entry = CacheEntry::new("greeting", "hello");
    let _ = string_entry.get();
    println!("{}", string_entry.describe());

    // 2. Generic enum: ComputedValue
    let ready: ComputedValue<String> = ComputedValue::Ready(String::from("42.5"));
    let pending: ComputedValue<String> = ComputedValue::Pending { task_id: 9001, estimated_ms: 200 };

    println!("{}", ready.describe());
    println!("{}", pending.describe());

    let value = ComputedValue::Ready(100u32).unwrap_or(0);
    println!("unwrapped: {}", value);

    // 3. PhantomData typed IDs
    let user_id:    UserId    = Id::new(1);
    let product_id: ProductId = Id::new(1);
    let _order_id:  OrderId   = Id::new(1);

    println!("{}", find_user_name(user_id));
    println!("{}", find_product_name(product_id));

    // These would be compile errors — uncomment to verify:
    // find_user_name(product_id);  // ERROR: expected Id<User>, found Id<Product>
    // find_user_name(order_id);    // ERROR: expected Id<User>, found Id<Order>

    // All three have value=1 but are distinct types at compile time
    println!(
        "sizes: user_id={}, product_id={}, order_id={}",
        std::mem::size_of::<UserId>(),
        std::mem::size_of::<ProductId>(),
        std::mem::size_of::<OrderId>(),
    );
    // PhantomData is zero-sized; all three are just u64 at runtime

    // 4. PriorityQueue
    let mut pq: PriorityQueue<u32> = PriorityQueue::new();
    pq.push(50);
    pq.push(10);
    pq.push(90);
    pq.push(30);

    println!("peek: {:?}", pq.peek());  // 90
    while let Some(item) = pq.pop() {
        println!("popped: {}", item);   // 90, 50, 30, 10
    }

    // 5. WithRetry wrapping Retryable
    let sensor = FlakySensor { calls: 0, succeeds_on: 3 };
    let mut retrier = WithRetry::new(sensor, 5);

    match retrier.run() {
        Ok(temp) => println!("temperature: {}", temp),
        Err(e)   => println!("error: {}", e),
    }
}
