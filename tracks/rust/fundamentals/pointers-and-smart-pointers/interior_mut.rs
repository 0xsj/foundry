// Interior Mutability — RefCell, Cell, and Rc<RefCell<T>>
//
// Run: rustc interior_mut.rs && ./interior_mut

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::collections::HashMap;

// ---------- Cell<T>: Simple Interior Mutability for Copy Types ----------

// Request metrics counter — must be updatable through &self because
// the metrics object is typically shared via &reference (e.g., Arc or Rc).
// Cell<u64> lets us increment counters without needing &mut self.
struct RequestMetrics {
    total: Cell<u64>,
    errors: Cell<u64>,
    total_duration_ms: Cell<u64>,
}

impl RequestMetrics {
    fn new() -> RequestMetrics {
        RequestMetrics {
            total: Cell::new(0),
            errors: Cell::new(0),
            total_duration_ms: Cell::new(0),
        }
    }

    // Note: &self, not &mut self — Cell allows mutation through shared refs
    fn record_request(&self, duration_ms: u64, success: bool) {
        self.total.set(self.total.get() + 1);
        self.total_duration_ms.set(self.total_duration_ms.get() + duration_ms);
        if !success {
            self.errors.set(self.errors.get() + 1);
        }
    }

    fn error_rate(&self) -> f64 {
        let total = self.total.get();
        if total == 0 { return 0.0; }
        self.errors.get() as f64 / total as f64
    }

    fn avg_duration_ms(&self) -> f64 {
        let total = self.total.get();
        if total == 0 { return 0.0; }
        self.total_duration_ms.get() as f64 / total as f64
    }
}

fn cell_demo() {
    println!("=== Cell<T> ===");

    let metrics = RequestMetrics::new();

    // These look like read-only calls, but they mutate internal state
    metrics.record_request(45, true);
    metrics.record_request(120, true);
    metrics.record_request(800, false);
    metrics.record_request(60, true);

    println!("total requests: {}", metrics.total.get());
    println!("error rate: {:.1}%", metrics.error_rate() * 100.0);
    println!("avg duration: {:.1}ms", metrics.avg_duration_ms());
    // 0.0% -> only 1 error in 4 -> 25%
    // avg = (45+120+800+60)/4 = 256.25ms
}

// ---------- RefCell<T>: Runtime Borrow Checking ----------

// A simple in-memory cache with LRU-like eviction.
// The key challenge: get(&self) modifies the cache (updates access order),
// but from the caller's perspective it's a read operation.
// RefCell lets us mutate through &self.
struct LruCache {
    capacity: usize,
    store: RefCell<HashMap<String, String>>,
    // Tracks insertion order for eviction (simplified: just a list)
    order: RefCell<Vec<String>>,
}

impl LruCache {
    fn new(capacity: usize) -> LruCache {
        LruCache {
            capacity,
            store: RefCell::new(HashMap::new()),
            order: RefCell::new(vec![]),
        }
    }

    // &self — looks read-only, but internally modifies access order
    fn get(&self, key: &str) -> Option<String> {
        if self.store.borrow().contains_key(key) {
            // Move key to end of order (most recently used)
            let mut order = self.order.borrow_mut();
            order.retain(|k| k != key);
            order.push(key.to_string());
            // Release order borrow before taking store borrow
            drop(order);
            self.store.borrow().get(key).cloned()
        } else {
            None
        }
    }

    fn put(&self, key: &str, value: &str) {
        let mut store = self.store.borrow_mut();
        let mut order = self.order.borrow_mut();

        if store.contains_key(key) {
            order.retain(|k| k != key);
        } else if store.len() >= self.capacity {
            // Evict least recently used (front of order list)
            if let Some(lru_key) = order.first().cloned() {
                store.remove(&lru_key);
                order.retain(|k| k != &lru_key);
            }
        }

        store.insert(key.to_string(), value.to_string());
        order.push(key.to_string());
    }

    fn len(&self) -> usize {
        self.store.borrow().len()
    }
}

fn refcell_demo() {
    println!("\n=== RefCell<T> ===");

    let cache = LruCache::new(3);

    cache.put("route:/api/users", "response_200_alice");
    cache.put("route:/api/posts", "response_200_list");
    cache.put("route:/api/orders", "response_200_orders");

    println!("cache size: {}", cache.len());  // 3

    // Access /api/users — makes it most recently used
    println!("users: {:?}", cache.get("route:/api/users"));

    // Insert a 4th entry — evicts least recently used (/api/posts)
    cache.put("route:/api/products", "response_200_products");

    println!("posts (evicted): {:?}", cache.get("route:/api/posts"));  // None
    println!("products (new): {:?}", cache.get("route:/api/products")); // Some(...)
    println!("cache size: {}", cache.len());  // still 3
}

// ---------- The Rc<RefCell<T>> Pattern: Shared Mutable State ----------

// A job queue where multiple producers can push work items and
// multiple consumers can drain them. All within a single thread
// (e.g., an async runtime's scheduler or a single-threaded event loop).
//
// Rc<RefCell<T>> = multiple owners + mutable access

type JobQueue = Rc<RefCell<Vec<String>>>;

fn make_queue() -> JobQueue {
    Rc::new(RefCell::new(vec![]))
}

struct Producer {
    name: String,
    queue: JobQueue,
}

impl Producer {
    fn new(name: &str, queue: &JobQueue) -> Producer {
        Producer {
            name: name.to_string(),
            queue: Rc::clone(queue),
        }
    }

    fn submit(&self, job: &str) {
        self.queue.borrow_mut().push(format!("[{}] {}", self.name, job));
    }
}

struct Consumer {
    queue: JobQueue,
}

impl Consumer {
    fn new(queue: &JobQueue) -> Consumer {
        Consumer { queue: Rc::clone(queue) }
    }

    fn drain(&self) -> Vec<String> {
        // Take all jobs — borrow_mut gives us exclusive access
        std::mem::take(&mut *self.queue.borrow_mut())
    }

    fn peek_count(&self) -> usize {
        self.queue.borrow().len()
    }
}

fn rc_refcell_demo() {
    println!("\n=== Rc<RefCell<T>> Pattern ===");

    let queue = make_queue();

    // Multiple producers share the same queue
    let ingest = Producer::new("ingest-worker", &queue);
    let retry  = Producer::new("retry-worker", &queue);
    let consumer = Consumer::new(&queue);

    ingest.submit("process:webhook:event_123");
    ingest.submit("process:webhook:event_124");
    retry.submit("retry:email:user_456");

    println!("queue depth: {}", consumer.peek_count());  // 3

    // Consumer drains all pending jobs
    let jobs = consumer.drain();
    println!("processed {} jobs:", jobs.len());
    for job in &jobs {
        println!("  {}", job);
    }

    println!("queue depth after drain: {}", consumer.peek_count());  // 0

    // Producers can still submit after drain
    ingest.submit("process:webhook:event_125");
    println!("queue depth: {}", consumer.peek_count());  // 1

    // Ownership check: queue, ingest, retry, consumer all share the Rc
    println!("Rc strong count: {}", Rc::strong_count(&queue));  // 4
}

// ---------- RefCell Panic Demo ----------
// Uncomment to see the runtime panic in action.
// The borrow checker would normally catch this at compile time
// for standard borrows. RefCell defers it to runtime.

#[allow(dead_code)]
fn refcell_panic_demo() {
    let data = RefCell::new(42);

    let _r1 = data.borrow();      // immutable borrow: active
    let _r2 = data.borrow();      // second immutable borrow: fine

    // This would PANIC: thread 'main' panicked at 'already borrowed: BorrowMutError'
    // let _w = data.borrow_mut();

    // Safer: try_borrow_mut returns Err instead of panicking
    match data.try_borrow_mut() {
        Ok(_) => println!("got mutable borrow"),
        Err(e) => println!("borrow failed: {}", e),
    }
}

fn main() {
    cell_demo();
    refcell_demo();
    rc_refcell_demo();
    refcell_panic_demo();
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_mutation_through_shared_ref() {
        let metrics = RequestMetrics::new();
        let r: &RequestMetrics = &metrics;  // shared reference

        r.record_request(100, true);
        r.record_request(200, false);

        assert_eq!(metrics.total.get(), 2);
        assert_eq!(metrics.errors.get(), 1);
    }

    #[test]
    fn test_lru_eviction() {
        let cache = LruCache::new(2);
        cache.put("a", "1");
        cache.put("b", "2");
        // Access "a" — makes it most recently used
        cache.get("a");
        // Insert "c" — should evict "b" (least recently used)
        cache.put("c", "3");

        assert_eq!(cache.get("a"), Some(String::from("1")));
        assert_eq!(cache.get("b"), None);  // evicted
        assert_eq!(cache.get("c"), Some(String::from("3")));
    }

    #[test]
    fn test_rc_refcell_shared_mutation() {
        let queue = make_queue();
        let p1 = Producer::new("p1", &queue);
        let p2 = Producer::new("p2", &queue);
        let c = Consumer::new(&queue);

        p1.submit("job-a");
        p2.submit("job-b");

        assert_eq!(c.peek_count(), 2);
        let drained = c.drain();
        assert_eq!(drained.len(), 2);
        assert_eq!(c.peek_count(), 0);
    }

    #[test]
    fn test_refcell_try_borrow_during_active_borrow() {
        let data = RefCell::new(0);
        let _r = data.borrow();  // active immutable borrow

        // try_borrow_mut should fail, not panic
        assert!(data.try_borrow_mut().is_err());
        // try_borrow should succeed (multiple immutable borrows are fine)
        assert!(data.try_borrow().is_ok());
    }
}
