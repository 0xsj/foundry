# Expert Review: Typed Event Bus

## Critical Issues

### 1. All struct fields are `pub` — implementation details are exposed

**Location:** `Subscriber`, `BusStats`, `EventBus` (throughout)

```rust
pub struct EventBus {
    pub subscribers: Vec<Subscriber>,
    pub stats: BusStats,
    pub next_id: u64,
}
```

**Problem:** Making internal state `pub` breaks encapsulation in the most fundamental
way. Any caller can do this:

```rust
bus.subscribers.push(Subscriber::new(999, "injected", EventCategory::System));
bus.stats.delivered = 0;  // reset stats without publishing anything
bus.next_id = 0;          // cause ID collisions
```

The methods `subscribe`, `unsubscribe`, and `publish` exist to be the *only* way to
modify the bus's state. With `pub` fields, those methods become suggestions.

**Fix:** Make all fields private. Expose read-only access through methods where needed:

```rust
pub struct EventBus {
    subscribers: Vec<Subscriber>,     // private
    stats: BusStats,                  // private
    next_id: u64,                     // private
}

impl EventBus {
    pub fn stats(&self) -> &BusStats { &self.stats }
    pub fn subscriber_count(&self) -> usize { self.subscribers.len() }
    // publish, subscribe, unsubscribe are the only ways to change state
}
```

Same for `Subscriber` — `received_count` should increment only via `record_received`.
Same for `BusStats` — callers should read it, not write it.

**Concept:** Fields in Rust are private by default for a reason. Every `pub` on a
field is a deliberate decision to make that field part of the public API. Defaulting
to `pub` on everything is the same mistake as writing `public` on everything in Java
before you think about whether it should be.

---

### 2. Missing `#[derive(Default)]` on `BusStats` and `EventBus::new` could use it

**Location:** `BusStats`, `EventBus`

```rust
impl BusStats {
    pub fn new() -> BusStats {
        BusStats {
            published: 0,
            delivered: 0,
            dropped: 0,
        }
    }
}
```

**Problem:** `BusStats` has three `u64` fields, all initialized to `0`. This is
exactly what `Default` provides. The manual `new()` function duplicates what
`#[derive(Default)]` generates, and it's inconsistent: if someone adds a field to
`BusStats`, they must remember to also initialize it in `new()`. With `derive`,
the compiler handles it automatically.

Additionally, `EventBus::new()` is the conventional name, but Rust also looks for
`impl Default for EventBus` when code uses `EventBus::default()` or when the struct
appears as a field of another `#[derive(Default)]` struct. Not implementing `Default`
makes the type awkward to compose.

**Fix:**

```rust
#[derive(Debug, Clone, Default)]  // add Default
pub struct BusStats {
    published: u64,   // all u64 default to 0
    delivered: u64,
    dropped: u64,
}

impl EventBus {
    pub fn new() -> EventBus {
        EventBus::default()  // delegate to Default
    }
}

impl Default for EventBus {
    fn default() -> EventBus {
        EventBus {
            subscribers: Vec::new(),
            stats: BusStats::default(),
            next_id: 1,
        }
    }
}
```

**Concept:** `Default` is a standard trait that the ecosystem expects. Types used
as struct fields, function parameters with defaults, or in generic bounds (`T: Default`)
should implement it. The pattern `fn new() -> Self { Self::default() }` is idiomatic
when the type has a meaningful default state.

---

## Major Concerns

### 3. No newtype for `subscriber_id` — bare `u64` allows ID confusion

**Location:** `Subscriber::subscriber_id`, `EventBus::subscribe` return, `EventBus::unsubscribe`/`find_subscriber` parameters

```rust
pub fn subscribe(&mut self, name: &str, category: EventCategory) -> u64 { ... }
pub fn unsubscribe(&mut self, subscriber_id: u64) -> bool { ... }
pub fn find_subscriber(&self, id: u64) -> Option<&Subscriber> { ... }
```

**Problem:** `subscribe` returns a `u64` subscriber ID. Callers then pass that ID
to `unsubscribe` and `find_subscriber`. But there's no type-level distinction between
a subscriber ID and any other `u64` the caller might have lying around:

```rust
let plugin_version_hash: u64 = compute_hash("1.2.0");
bus.unsubscribe(plugin_version_hash);  // compiles fine, silently wrong
```

If a caller mixes up a plugin ID, a hash, and a subscriber ID, the compiler cannot help.

**Fix:** Introduce a `SubscriberId` newtype:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriberId(u64);

impl EventBus {
    pub fn subscribe(&mut self, name: &str, category: EventCategory) -> SubscriberId {
        let id = SubscriberId(self.next_id);
        self.next_id += 1;
        self.subscribers.push(Subscriber::new(id, name, category));
        id
    }

    pub fn unsubscribe(&mut self, id: SubscriberId) -> bool { ... }
    pub fn find_subscriber(&self, id: SubscriberId) -> Option<&Subscriber> { ... }
}
```

Now passing a `PluginId(42)` where a `SubscriberId` is expected is a compile error.

**Concept:** The newtype pattern. Wrapping primitives in tuple structs costs nothing
at runtime (the type disappears after compilation) but gives you compile-time enforcement
of which IDs are which.

---

### 4. `BusEvent::name()` clones unnecessarily — a `&str` return would suffice

**Location:** `BusEvent::name`

```rust
pub fn name(&self) -> String {
    match self {
        BusEvent::PluginLoaded { plugin_id, .. } => {
            format!("plugin_loaded({})", plugin_id.clone())  // unnecessary .clone()
        }
        // ...
    }
}
```

**Problem 1:** `.clone()` is unnecessary inside `format!`. `format!` borrows its
arguments — it does not consume them. `plugin_id.clone()` allocates a second `String`
that is immediately converted to a format argument and then dropped. Remove the `.clone()`:

```rust
format!("plugin_loaded({})", plugin_id)  // plugin_id is &String, format! borrows it
```

**Problem 2:** The method allocates a `String` on every call. If callers only need a
label for logging or debugging, returning a `&'static str` (for the type name) or a
`Cow<'static, str>` (for dynamic labels) avoids the allocation entirely.

For a pure type label, a separate method returning `&'static str` is cleaner:

```rust
pub fn kind_name(&self) -> &'static str {
    match self {
        BusEvent::PluginLoaded { .. } => "plugin_loaded",
        BusEvent::PluginUnloaded { .. } => "plugin_unloaded",
        BusEvent::ConfigChanged { .. } => "config_changed",
        BusEvent::HealthCheck { .. } => "health_check",
        BusEvent::Shutdown => "shutdown",
    }
}
```

Keep `name()` (which includes the plugin ID) as a `String` return — that's appropriate
since it constructs a new string with dynamic data. Just remove the unnecessary `.clone()`.

**Concept:** Inside `format!`, all arguments are borrowed. Cloning before passing to
`format!` is always unnecessary. Clippy catches this with `clippy::clone_on_ref_ptr` and
related lints.

---

## Minor Suggestions

### 5. `find_subscriber` uses a redundant `match` — use `if let` or just return directly

**Location:** `EventBus::find_subscriber`

```rust
pub fn find_subscriber(&self, id: u64) -> Option<&Subscriber> {
    let result = self.subscribers.iter().find(|s| s.subscriber_id == id);
    match result {
        Some(s) => Some(s),
        None => None,
    }
}
```

**Problem:** The `match` here maps `Some(s)` to `Some(s)` and `None` to `None`. It is
completely identity — it passes through the `Option` unchanged. This is equivalent to:

```rust
pub fn find_subscriber(&self, id: u64) -> Option<&Subscriber> {
    self.subscribers.iter().find(|s| s.subscriber_id == id)
}
```

The intermediate variable and the match add visual noise without adding meaning. Clippy
warns about this with `clippy::match_as_ref` or `clippy::redundant_pattern_matching`.

If there were any transformation — even wrapping in a different type — the match would
be appropriate. But when both arms are identity, just return the value directly.

---

### 6. `format_for_log` takes `EventCategory` by value — should take `&EventCategory`

**Location:** `format_for_log`

```rust
pub fn format_for_log(event: &BusEvent, category: EventCategory) -> String {
    format!("[{:?}] {}", category, event.name())
}
```

**Problem:** `EventCategory` is moved into `format_for_log`. The caller cannot use their
`EventCategory` value after calling this function. Since `EventCategory` only derives
`Clone`, the caller must explicitly clone before passing if they want to keep it. But
`format_for_log` only reads the category — it doesn't store it or consume it.

**Fix:** Take a shared reference:

```rust
pub fn format_for_log(event: &BusEvent, category: &EventCategory) -> String {
    format!("[{:?}] {}", category, event.name())
}
```

As a follow-up, if `EventCategory` is small (it has no heap-allocated fields), derive
`Copy` so it's implicitly copyable — no `&` needed and no explicit clone:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]  // add Copy
pub enum EventCategory { Lifecycle, Config, Health, System }
```

`Copy` is appropriate for enums with no heap-allocated data. It's the same as all
integer and boolean types — cheap to copy because the data is just a small tag.

---

## Positive Feedback

1. **The `BusEvent::category()` method is well designed.** Centralizing the category
   classification in one place means adding a new event type requires updating exactly
   one match, not scattered `if let` chains across the codebase.

2. **`publish` correctly tracks both `delivered` and `dropped`.** The distinction
   between "published with no subscribers" (dropped) and "published with delivery"
   is exactly right for production observability. Many bus implementations miss the
   dropped count.

3. **`unsubscribe` using `retain` is idiomatic.** `Vec::retain` is the correct, efficient
   way to remove elements by predicate. No manual index tracking needed.

4. **Tests cover the key behaviors.** subscribe/publish, unsubscribe, cross-category
   filtering, and received_count are all tested. The test coverage is actually good — the
   issues here are design issues, not correctness issues.

5. **`Subscriber::matches` is a clean delegation pattern.** Having the subscriber own
   the filter logic (`fn matches(&self, event: &BusEvent) -> bool`) rather than having
   the bus check the category directly keeps the subscriber's filtering logic cohesive.

---

## Summary

| # | Severity | Issue | Rust Concept |
|---|----------|-------|--------------|
| 1 | Critical | All fields are `pub` — state can be mutated directly | Encapsulation, private fields by default |
| 2 | Critical | Missing `Default` impl — manual `new()` duplicates what derive handles | `#[derive(Default)]`, `Default` trait |
| 3 | Major | Bare `u64` subscriber IDs — type confusion between different IDs | Newtype pattern |
| 4 | Major | Unnecessary `.clone()` in `format!`, returns `String` when `&'static str` works for some cases | Borrowing in format macros, allocation cost |
| 5 | Minor | Redundant `match` in `find_subscriber` — identity mapping | `if let`, returning `Option` directly |
| 6 | Minor | `format_for_log` takes `EventCategory` by value — should borrow | `&T` vs `T` in function parameters, `Copy` derive |

## Related Concepts

- [[fundamentals/rust/structs-methods-and-enums]] — struct encapsulation, newtype pattern, derive macros
- [[patterns/builder]] — `EventBus::new` plus `Default` is a stepping stone to the builder pattern
- [[pitfalls/rust-pub-fields]] — when pub fields break invariants
- [[pitfalls/rust-unnecessary-clone]] — clone in format!, clone before borrow
