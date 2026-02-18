# Expert Review: Generic Cache Implementation

## Critical Issues

### 1. `PhantomData<M>` strategy marker adds complexity with zero behavior

**Location:** `Cache<K, V, E, M>` struct definition, `LruMarker`, `FifoMarker`

```rust
pub struct Cache<K, V, E, M>
where
    ...
{
    // ...
    _strategy: PhantomData<M>,
}

pub struct LruMarker;
pub struct FifoMarker;
```

**Problem:** The `M` type parameter is supposed to represent a cache eviction strategy
(LRU vs FIFO), but it's never used to influence behavior. The `insert` method uses
`find_oldest_key()` regardless of whether `M` is `LruMarker` or `FifoMarker`. The marker
is pure ceremony — it makes the type signature more complex without delivering on its promise.

Phantom type parameters are useful when they carry meaning the compiler enforces. Classic
examples:

- `PhantomData<T>` in a raw pointer wrapper to express ownership (`*const T` doesn't imply
  ownership but wrapping it in a struct with `PhantomData<T>` does)
- State machine markers where different `M` types enable/disable methods via `impl` blocks
  (`Connection<Connected>` vs `Connection<Disconnected>`)

Here, `LruMarker` and `FifoMarker` don't gate any methods. A `Cache<..., LruMarker>` and
a `Cache<..., FifoMarker>` behave identically. The phantom parameter is misleading — callers
see "LRU" in the type and assume the cache uses LRU eviction. It doesn't.

**Fix:** Remove `M` entirely. If different eviction strategies are needed later, implement
them as a trait (similar to `Expiry`) that the cache delegates to:

```rust
pub trait EvictionPolicy<K> {
    fn select_victim(&self, entries: &HashMap<K, CacheEntry<V>>) -> Option<K>;
}

pub struct Cache<K, V, E> {
    entries: HashMap<K, CacheEntry<V>>,
    expiry_policy: E,
    // If you want pluggable eviction, add it as a real field with behavior:
    // eviction_policy: Box<dyn EvictionPolicy<K>>,
    max_capacity: usize,
    hits: u64,
    misses: u64,
}
```

**Concept:** Phantom type parameters should enforce invariants at the type level. If the
phantom type doesn't change what methods are available or how they behave, it's dead weight.
Remove it and add it back when there's real behavior to gate.

---

### 2. `Expiry<E>` trait has a useless generic parameter — should use no generics

**Location:** `trait Expiry<E>`

```rust
pub trait Expiry<E> {
    fn is_expired(&self, inserted_at: Instant) -> bool;
    fn default_ttl(&self) -> Duration;
}
```

**Problem:** The `E` parameter represents the cache entry type, but neither `is_expired`
nor `default_ttl` ever receives an `E` value. The parameter is completely unused in the
trait methods. This forces implementors to write blanket impls or impl for every concrete
entry type separately:

```rust
// Must impl for every E, even though E is never used
impl<E: Sized> Expiry<E> for TtlExpiry { ... }
```

This is the opposite of the associated type vs generic decision. Here, the right answer
is *neither* — the trait simply doesn't need the parameter at all.

**Fix:** Remove the generic parameter entirely:

```rust
pub trait Expiry {
    fn is_expired(&self, inserted_at: Instant) -> bool;
    fn default_ttl(&self) -> Duration;
}

impl Expiry for TtlExpiry {
    fn is_expired(&self, inserted_at: Instant) -> bool {
        inserted_at.elapsed() > self.ttl
    }
    fn default_ttl(&self) -> Duration {
        self.ttl
    }
}
```

Now there's no spurious generic, no blanket impl needed, and the trait signature honestly
describes what it requires.

**When WOULD entry-aware expiry make sense?** If some entries should live longer than
others based on their content (e.g., a `SlidingExpiry` that reads the entry's last-access
timestamp). In that case the method signature would be
`fn is_expired(&self, entry: &E) -> bool` — the parameter would be *used*. But that's not
what's happening here.

**Concept:** Generic parameters on traits should be driven by method signatures. If no method
receives or returns the generic type, the parameter is unnecessary. Adding unused generics
is a common "just in case" mistake that ripples through every impl and every caller.

---

## Major Concerns

### 3. Trait bounds on struct definition — should be on `impl` blocks only

**Location:** `struct Cache<K, V, E, M> where K: Hash + Eq + Clone + Debug, V: Clone + Debug, E: Expiry<V>`

```rust
pub struct Cache<K, V, E, M>
where
    K: std::hash::Hash + Eq + Clone + fmt::Debug,
    V: Clone + fmt::Debug,
    E: Expiry<V>,
{
    entries: HashMap<K, CacheEntry<V>>,
    ...
}
```

**Problem:** Rust allows trait bounds on struct definitions, but the Rust API Guidelines
and standard library convention is to avoid them. Bounds on structs "leak" — any code that
merely mentions `Cache<K, V, E, M>` (even in a type alias, a function signature, or a
struct field) must satisfy all bounds, even if it never calls any methods.

Example of the friction:

```rust
// This fails to compile because K doesn't satisfy Hash + Eq + Clone + Debug
// even though we're just storing the cache, not using it
struct AppState<K, V, E, M> {
    cache: Option<Cache<K, V, E, M>>,  // ERROR: bounds not satisfied
}
```

The standard library follows this pattern: `HashMap<K, V>` has no bounds on its struct
definition, even though `insert` requires `K: Hash + Eq`. You can declare
`HashMap<MyKey, MyValue>` anywhere — the bounds are only checked when you call methods.

**Fix:** Remove bounds from the struct, add them to `impl` blocks:

```rust
pub struct Cache<K, V, E> {
    entries: HashMap<K, CacheEntry<V>>,
    expiry_policy: E,
    max_capacity: usize,
    hits: u64,
    misses: u64,
}

impl<K, V, E> Cache<K, V, E>
where
    K: std::hash::Hash + Eq + Clone,
    E: Expiry,
{
    pub fn new(expiry_policy: E, max_capacity: usize) -> Self { ... }
    pub fn get(&mut self, key: &K) -> Option<&V> { ... }
    pub fn insert(&mut self, key: K, value: V) { ... }
    // ...
}

// Methods that need Debug can have stricter bounds in a separate impl block
impl<K, V, E> Cache<K, V, E>
where
    K: std::hash::Hash + Eq + Clone + fmt::Debug,
    V: fmt::Debug,
    E: Expiry,
{
    pub fn debug_entries(&self) { ... }
}
```

Now `Cache` can be mentioned in type positions without requiring all bounds upfront.
Methods that need `Clone` only require it when called.

**Concept:** In Rust, trait bounds on structs are almost always wrong. Put bounds on `impl`
blocks instead. This is the "bound on impl, not on type" pattern. `HashMap`, `Vec`, `BTreeMap`
all follow this — check their definitions in the standard library.

---

### 4. `cache_stats` is generic but doesn't need to be — monomorphization bloat

**Location:** `fn cache_stats<K, V, E, M>`

```rust
pub fn cache_stats<K, V, E, M>(cache: &Cache<K, V, E, M>) -> String
where
    K: std::hash::Hash + Eq + Clone + fmt::Debug,
    V: Clone + fmt::Debug,
    E: Expiry<V>,
{
    format!(
        "Cache stats: {} entries, {:.1}% hit rate ({} hits, {} misses)",
        cache.len(),
        cache.hit_rate() * 100.0,
        cache.hits,
        cache.misses,
    )
}
```

**Problem:** This function is generic over `<K, V, E, M>` but only reads `len()`,
`hit_rate()`, `hits`, and `misses` — all of which are `usize`, `f64`, and `u64`. None of
them depend on `K`, `V`, `E`, or `M`.

Every unique instantiation of `Cache` that calls `cache_stats` generates a separate copy
of this function in the binary. A `cache_stats::<String, Endpoint, TtlExpiry, LruMarker>`
and a `cache_stats::<u64, Config, TtlExpiry, FifoMarker>` are two identical functions
formatting the same three numbers. This is textbook monomorphization bloat.

**Fix:** Extract a non-generic inner function or a method:

```rust
// Option A: Method on Cache (simplest)
impl<K, V, E> Cache<K, V, E> {
    pub fn stats(&self) -> String {
        format!(
            "Cache stats: {} entries, {:.1}% hit rate ({} hits, {} misses)",
            self.len(),
            self.hit_rate() * 100.0,
            self.hits,
            self.misses,
        )
    }
}

// Option B: Non-generic helper struct
pub struct CacheStats {
    pub entries: usize,
    pub hit_rate: f64,
    pub hits: u64,
    pub misses: u64,
}

impl fmt::Display for CacheStats { ... }

// Method on Cache returns stats without monomorphizing the formatting
impl<K, V, E> Cache<K, V, E> {
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.len(),
            hit_rate: self.hit_rate(),
            hits: self.hits,
            misses: self.misses,
        }
    }
}
```

Option B is the "generic outer, concrete inner" pattern — the generic code extracts concrete
values, then delegates to non-generic code for the actual work. The standard library uses this
pattern extensively (e.g., `Vec::push` delegates to a non-generic `RawVec::grow`).

**Concept:** When a generic function's body doesn't actually use the type parameters, it's
generating redundant code. Extract the non-generic parts into a concrete function or method.
This keeps binary size down and improves instruction cache behavior.

---

## Minor Suggestions

### 5. `evict_expired` collects keys into a `Vec` — could use `retain` instead

**Location:** `fn evict_expired(&mut self)`

```rust
fn evict_expired(&mut self) {
    let expired_keys: Vec<K> = self
        .entries
        .iter()
        .filter(|(_, entry)| self.expiry_policy.is_expired(entry.inserted_at))
        .map(|(k, _)| k.clone())
        .collect();

    for key in expired_keys {
        self.entries.remove(&key);
    }
}
```

`HashMap::retain` does this in one pass without allocating a temporary Vec:

```rust
fn evict_expired(&mut self) {
    let policy = &self.expiry_policy;
    self.entries.retain(|_, entry| !policy.is_expired(entry.inserted_at));
}
```

This requires extracting `expiry_policy` into a local reference to avoid borrowing `self`
mutably (through `retain`) and immutably (through `self.expiry_policy`) at the same time.
It's a common Rust pattern.

### 6. `K: Clone` is only needed for `evict_expired` and `find_oldest_key`

The `Clone` bound on `K` is used in two places: cloning keys for the expired-keys vector
and for `find_oldest_key`. If `evict_expired` uses `retain` (no key cloning needed) and
`find_oldest_key` returns the key by reference, `K: Clone` can be dropped entirely. Fewer
bounds = more flexible API.

---

## Positive Feedback

1. **Clean separation of expiry policy from cache logic.** Making expiry a trait rather than
   hardcoding TTL is the right call. A `SlidingWindowExpiry` or `NoExpiry` policy can be
   plugged in without touching `Cache`. This is solid strategy pattern usage.

2. **`hit_rate()` with zero-division guard.** Returning `0.0` when total is zero is correct
   and avoids a subtle divide-by-zero bug. Small but important.

3. **Type aliases for concrete cache configurations.** `WebhookCache` and `ConfigCache` hide
   the verbose generic parameters from callers. This is the right ergonomic pattern — define
   the generic core once, then expose concrete aliases for common use cases.

4. **`CacheEntry` keeps value and metadata together.** Storing `inserted_at` alongside the
   value in a `CacheEntry` struct is cleaner than using parallel `HashMap`s for values and
   timestamps. It keeps related data co-located and makes eviction logic straightforward.

---

## Summary

| # | Severity | Issue | Rust Concept |
|---|----------|-------|--------------|
| 1 | Critical | `PhantomData<M>` marker adds type complexity but no behavior | Phantom types, type-level state machines |
| 2 | Critical | `Expiry<E>` has unused generic parameter — should be non-generic | When traits need generics vs not |
| 3 | Major | Trait bounds on struct definition instead of impl blocks | Bounds on impl, not type (Rust convention) |
| 4 | Major | `cache_stats` is fully generic but uses no type parameters | Monomorphization bloat, generic outer/concrete inner |
| 5 | Minor | `evict_expired` allocates a Vec; `retain` avoids it | `HashMap::retain`, borrow splitting |
| 6 | Minor | `K: Clone` only needed due to current eviction approach | Minimal bounds principle |

## Related Concepts

- [[fundamentals/rust/generics]] — monomorphization, trait bounds, PhantomData
- [[fundamentals/rust/interfaces-and-traits]] — associated types vs generic traits
- [[patterns/strategy]] — the Expiry trait is a strategy pattern
- [[pitfalls/rust-bounds-on-struct]] — why bounds belong on impl blocks, not structs
- [[pitfalls/rust-phantom-without-purpose]] — PhantomData that doesn't enforce an invariant
