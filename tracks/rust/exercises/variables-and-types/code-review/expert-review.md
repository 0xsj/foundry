# Expert Review: In-Memory Cache

## Critical Issues

### 1. `unwrap()` on user-provided strings (lines 43-44)

**Location:** `CacheConfig::from_strings`

```rust
let default_ttl: u64 = ttl_str.parse().unwrap();
let max_entries: usize = max_str.parse().unwrap();
```

**Problem:** `unwrap()` panics on invalid input. If TTL or max_entries come from
a config file, environment variable, or CLI flag, any non-numeric value crashes
the entire process. This is a production reliability issue -- config parsing is
one of the most common places to receive unexpected input.

**Fix:** Return a `Result` and let the caller decide how to handle errors:

```rust
pub fn from_strings(
    ttl_str: &str,
    max_str: &str,
    name: &str,
) -> Result<CacheConfig, String> {
    let default_ttl: u64 = ttl_str
        .parse()
        .map_err(|e| format!("invalid TTL '{}': {}", ttl_str, e))?;
    let max_entries: usize = max_str
        .parse()
        .map_err(|e| format!("invalid max_entries '{}': {}", max_str, e))?;

    Ok(CacheConfig {
        default_ttl,
        max_entries,
        name: name.to_string(),
    })
}
```

**Concept:** `Result<T, E>` is Rust's primary error handling mechanism. `unwrap()`
should only be used when you can *prove* the value is always valid (e.g., a
hardcoded regex pattern), never on user input.

---

### 2. Returning reference to local data -- and the wrong workaround (lines 70-82)

**Location:** `CacheEntry::display_value`

```rust
// Original attempt (commented out):
//   pub fn display_value(&self) -> &str {
//       let formatted = format!("{} (expires: {})", self.value, self.expires_at);
//       &formatted  // ERROR: returns reference to local variable
//   }
//
// "Fixed" by returning owned String
pub fn display_value(&self) -> String {
    let formatted = format!("{} (expires: {})", self.value, self.expires_at);
    formatted
}
```

**Problem:** The original code tried to return a `&str` reference to a `String`
created inside the function. That `String` is dropped when the function returns,
so the reference would dangle. The compiler correctly rejects this.

The "fix" of returning `String` works but is a band-aid. The author doesn't
understand *why* the original failed. The real lesson is:

- You cannot return a reference to data created inside a function (it will be
  dropped when the function returns)
- If you need to return a reference, it must point to data that outlives the
  function call (e.g., a field of `&self`)
- If you must create new data, returning an owned `String` is the correct
  approach -- but the comment frames it as a "workaround" rather than the right
  design

**Recommendation:** Accept that `display_value` must return `String` since it
creates new data via `format!`. Remove the commented-out code and the
"WORKAROUND" / "TODO" comments. If you just need the raw value, add a separate
method:

```rust
/// Returns a reference to the stored value (no allocation).
pub fn value(&self) -> &str {
    &self.value
}

/// Returns a formatted display string (allocates).
pub fn display_value(&self) -> String {
    format!("{} (expires: {})", self.value, self.expires_at)
}
```

**Concept:** Lifetime rule -- a reference returned from a function must be tied
to the lifetime of an input reference (usually `&self`). Data created inside the
function cannot be returned by reference.

---

## Major Concerns

### 3. `String` parameters where `&str` would suffice (lines 95, 106)

**Location:** `Cache::insert` and `Cache::get`

```rust
pub fn insert(&mut self, key: String, value: String, current_time: u64) { ... }
pub fn get(&mut self, key: String, current_time: u64) -> Option<&str> { ... }
```

**Problem:** Both methods take `String` (owned), forcing callers to allocate even
when they have a `&str`:

```rust
// Caller must allocate just to look up a key:
cache.get(String::from("host"), 1000);

// With &str parameter, this would work directly:
cache.get("host", 1000);
```

**Fix for `insert`:** The key needs to be owned (it goes into the HashMap), but
accept `&str` and convert internally. The value also needs to be owned for
`CacheEntry`:

```rust
pub fn insert(&mut self, key: &str, value: &str, current_time: u64) {
    // ...
    let expires_at = current_time + self.config.default_ttl;
    self.entries.insert(key.to_string(), CacheEntry::new(value, expires_at));
}
```

**Fix for `get`:** The key is only used for lookup, never stored:

```rust
pub fn get(&mut self, key: &str, current_time: u64) -> Option<&str> {
    // HashMap::get accepts any type that the key can Borrow to,
    // so &str works directly for String keys
    if let Some(entry) = self.entries.get(key) {
        if entry.expires_at <= current_time {
            self.entries.remove(key);
            self.miss_count += 1;
            return None;
        }
    }
    // ...
}
```

**Concept:** Accept the most general borrowed type (`&str`) in function
parameters. Only use `String` when the function truly needs ownership. This
avoids forcing callers to allocate. The HashMap lookup works with `&str` because
`String` implements `Borrow<str>`.

---

### 4. Broken manual `Clone` impl (lines 147-160)

**Location:** `impl Clone for CacheSnapshot`

```rust
impl Clone for CacheSnapshot {
    fn clone(&self) -> CacheSnapshot {
        let mut new_keys = Vec::with_capacity(self.keys.len());
        for key in &self.keys {
            new_keys.push(key.clone());
        }
        CacheSnapshot {
            keys: new_keys,
            timestamp: self.timestamp,
            entry_count: 0,  // BUG: should be self.entry_count
        }
    }
}
```

**Problem:** `entry_count` is hardcoded to `0` instead of `self.entry_count`.
This is a copy-paste mistake that `#[derive(Clone)]` would have prevented. The
entire manual implementation is unnecessary -- all fields (`Vec<String>`, `u64`,
`usize`) implement `Clone`, so derive works.

**Fix:** Replace the manual impl with derive:

```rust
#[derive(Debug, Clone)]
pub struct CacheSnapshot {
    pub keys: Vec<String>,
    pub timestamp: u64,
    pub entry_count: usize,
}
```

**Concept:** Prefer `#[derive(Clone)]` over manual implementations. Manual
impls are error-prone and harder to maintain. Only write manual `Clone` when
you need custom behavior (e.g., incrementing a reference count, deep-copying
through a smart pointer).

---

## Minor Suggestions

### 5. `match` with single arm and wildcard (lines 203-208, 216-219)

**Location:** `lookup_and_print` and `get_expiration`

```rust
// In lookup_and_print:
match result {
    Some(value) => {
        println!("[{}] {} = {}", cache.config.name, key, value);
    }
    None => {}
}

// In get_expiration:
match entry {
    Some(e) => Some(e.expires_at),
    None => None,
}
```

**Fix for `lookup_and_print`:** Use `if let` when you only care about one variant:

```rust
if let Some(value) = result {
    println!("[{}] {} = {}", cache.config.name, key, value);
}
```

**Fix for `get_expiration`:** Use `map` on the Option:

```rust
pub fn get_expiration(cache: &Cache, key: &str) -> Option<u64> {
    cache.entries.get(key).map(|e| e.expires_at)
}
```

**Concept:** `if let` is idiomatic for matching a single pattern. `Option::map`
is idiomatic for transforming the inner value. Using full `match` for these cases
is verbose and obscures intent. Clippy warns about this:
`clippy::single_match` and `clippy::manual_map`.

---

### 6. `pub` on everything (throughout file)

**Location:** All struct fields, all methods, all functions

```rust
pub struct CacheEntry {
    pub value: String,
    pub expires_at: u64,
}

pub struct Cache {
    pub entries: HashMap<String, CacheEntry>,
    pub config: CacheConfig,
    pub hit_count: u64,
    pub miss_count: u64,
}
```

**Problem:** Making everything `pub` exposes implementation details. External code
can directly mutate `entries`, `hit_count`, or `miss_count`, bypassing the cache's
methods. The helper function `evict_expired` is an implementation detail that
should not be part of the public API.

**Fix:** Make fields private by default, expose only what callers need through methods:

```rust
pub struct Cache {
    entries: HashMap<String, CacheEntry>,  // private
    config: CacheConfig,                    // private
    hit_count: u64,                         // private
    miss_count: u64,                        // private
}

impl Cache {
    pub fn new(config: CacheConfig) -> Cache { ... }
    pub fn insert(&mut self, key: &str, value: &str, current_time: u64) { ... }
    pub fn get(&mut self, key: &str, current_time: u64) -> Option<&str> { ... }
    pub fn stats(&self) -> String { ... }

    // Private — internal implementation detail
    fn evict_expired(&mut self, current_time: u64) { ... }
}
```

**Concept:** Encapsulation. Rust fields are private by default for a reason --
it lets the author change internal representation without breaking callers. Only
make things `pub` when they are part of the intentional API surface. This is
especially important for library code.

---

## Positive Feedback

1. **Good use of `HashMap::retain` in `evict_expired`** -- this is the idiomatic
   way to remove entries by predicate. Clean and efficient.

2. **Reasonable struct decomposition** -- separating `CacheConfig` from `Cache`
   is good design. Configuration is a distinct concern from runtime state.

3. **Test coverage is decent** -- tests cover basic CRUD, expiration, stats, and
   snapshot cloning. The `#[should_panic]` test correctly documents the
   `unwrap()` behavior.

4. **`CacheEntry::new` correctly converts `&str` to owned `String`** -- this
   is the right pattern for struct constructors (even though `Cache::insert`
   doesn't follow it).

5. **The `get` method correctly returns `Option<&str>`** -- returning a
   borrowed slice of the stored value avoids unnecessary cloning on read.

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | `unwrap()` on user input | Error handling with `Result` |
| 2 | Critical | Returning reference to local data | Lifetimes and ownership |
| 3 | Major | `String` params where `&str` suffices | Borrowing, `Borrow` trait |
| 4 | Major | Broken manual `Clone` (entry_count = 0) | Derive vs manual trait impls |
| 5 | Minor | `match` with single arm + wildcard | `if let`, `Option::map` |
| 6 | Minor | `pub` on all fields and helpers | Encapsulation, API design |

## Related Concepts

- [[fundamentals/rust/variables-and-types]] -- String vs &str, ownership
- [[pitfalls/rust-unwrap-on-user-input]] -- When unwrap is and isn't appropriate
- [[fundamentals/variables-and-types]] -- Cross-language comparison of value vs reference
