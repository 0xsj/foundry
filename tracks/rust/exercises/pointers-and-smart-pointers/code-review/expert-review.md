# Expert Review: Plugin System with Shared State

## Critical Issues

### 1. Plugin holds `Rc<Registry>` — creates a reference cycle with the registry

**Location:** `Plugin::registry` field, `Registry::register`

```rust
pub struct Plugin {
    pub registry: RefCell<Option<Rc<Registry>>>,
    // ...
}

// In Registry::register:
*plugin.registry.borrow_mut() = Some(Rc::clone(registry));
registry.plugins.lock().unwrap().push(plugin);
```

**Problem:** The cycle is:

```
Registry (Rc)
  └── plugins: Vec<Rc<Plugin>>
         └── Plugin (Rc)
               └── registry: Option<Rc<Registry>>   <- points back to Registry
```

Both reference counts will always be at least 1 (each holds the other). Neither allocation
will ever be freed. The registry and every plugin registered to it will leak for the lifetime
of the program.

This is the most common memory management mistake in Rust code that uses `Rc`. The compiler
does not warn about it. It will only manifest as growing memory usage in a long-running service.

**Fix:** Change the plugin's registry reference to `Weak<Registry>`:

```rust
pub struct Plugin {
    pub registry: RefCell<Option<Weak<Registry>>>,
    // ...
}
```

Update `register` to store a `Weak`:

```rust
pub fn register(registry: &Rc<Registry>, plugin: Rc<Plugin>) {
    *plugin.registry.borrow_mut() = Some(Rc::downgrade(registry));
    registry.plugins.borrow_mut().push(plugin);
}
```

Update `get_registry_plugin_count` to upgrade the Weak:

```rust
pub fn get_registry_plugin_count(&self) -> Option<usize> {
    self.registry.borrow().as_ref()?.upgrade().map(|r| r.plugin_count())
}
```

**Concept:** In any parent-child or owner-member relationship: the owner holds `Rc`, the
member holds `Weak`. The Registry owns the plugins (it controls their lifetime). Plugins
observe the registry — they should not keep it alive.

**How to detect:** `Rc::strong_count(&registry)` in a test after dropping all local variables
should be 0 (or 1 if the test still holds it). A count above expected indicates a cycle.

---

## Major Concerns

### 2. `Arc<Mutex<T>>` used in an entirely single-threaded context

**Location:** `Registry::plugins`, `Registry::shared_config`

```rust
pub struct Registry {
    plugins: Arc<Mutex<Vec<Rc<Plugin>>>>,
    shared_config: Arc<Mutex<HashMap<String, String>>>,
}
```

**Problem:** There is no `thread::spawn` anywhere in this code. Nothing is `Send`. The
`Rc<Plugin>` values inside the `Vec` are themselves not `Send`, so the `Arc<Mutex<Vec<Rc<Plugin>>>>`
is not `Send` either (the inner type prohibits it). This means the `Arc` wrapper adds atomic
overhead and API friction with no benefit — you can't actually use it across threads anyway.

Additionally:
- `Arc::new(Mutex::new(...))` is more expensive to construct than `RefCell::new(...)`
- Every access requires `.lock().unwrap()` instead of `.borrow()` — noisier code
- The `Mutex` can deadlock if a borrow spans a function that also tries to lock
- `Arc<Mutex<Rc<...>>>` doesn't implement `Send` because `Rc` isn't `Send` — so the
  entire premise of using `Arc<Mutex<>>` for thread safety is broken here

**Fix:** Replace with `RefCell<T>` for single-threaded interior mutability:

```rust
pub struct Registry {
    plugins: RefCell<Vec<Rc<Plugin>>>,
    shared_config: RefCell<HashMap<String, String>>,
}
```

Access becomes:
```rust
pub fn plugin_count(&self) -> usize {
    self.plugins.borrow().len()
}
pub fn set_global_config(&self, key: &str, value: &str) {
    self.shared_config.borrow_mut().insert(key.to_string(), value.to_string());
}
```

If thread safety is actually needed later, the migration from `Rc` → `Arc` and
`RefCell` → `Mutex` is a deliberate, informed upgrade — not something to preemptively add.

**Concept:** Use the lightest tool that solves your current problem. `Mutex` is for thread
synchronization. `RefCell` is for interior mutability. Don't reach for the thread-safe
primitive when threads aren't involved — it misleads readers and adds unnecessary overhead.

---

### 3. `PluginConfig` wraps all fields in `RefCell` — but config is never mutated

**Location:** `PluginConfig` struct, `PluginConfig::name()`, `PluginConfig::version()`

```rust
pub struct PluginConfig {
    name: RefCell<String>,
    version: RefCell<String>,
    settings: RefCell<HashMap<String, String>>,
}
```

**Problem:** `PluginConfig` is constructed via `new()` and `with_setting()` (which takes
`self` by value — a builder pattern). After construction, nothing in the codebase mutates
a `PluginConfig`. The `RefCell` wrappers:

1. Add runtime borrow tracking overhead on every access
2. Make the API confusing: does the caller need to worry about borrow conflicts?
3. Force `name()` and `version()` to call `.borrow()` and `.clone()` — allocating on every call
4. Signal "this data may be mutated at any time" — which is misleading since it isn't

**Fix:** Use plain fields:

```rust
pub struct PluginConfig {
    name: String,
    version: String,
    settings: HashMap<String, String>,
}
```

`name()` and `version()` can return `&str`:

```rust
pub fn name(&self) -> &str { &self.name }
pub fn version(&self) -> &str { &self.version }
pub fn get_setting(&self, key: &str) -> Option<&str> {
    self.settings.get(key).map(|s| s.as_str())
}
```

If `PluginConfig` ever needs to be updated after construction (e.g., hot-reload), add `RefCell`
at that point with a clear comment explaining why.

**Concept:** `RefCell<T>` is for "I need to mutate through a shared reference." If you don't
need that, plain fields are simpler, cheaper, and more legible. Reach for `RefCell` when
you have a concrete need, not defensively.

---

## Minor Suggestions

### 4. `name()` and `version()` return `String` — `&str` avoids the allocation

**Location:** `PluginConfig::name`, `PluginConfig::version`, `PluginConfig::get_setting`

```rust
pub fn name(&self) -> String {
    self.name.borrow().clone()  // allocates a new String every call
}
```

**Problem:** The callers of `name()` in this codebase (logging, collecting names into a `Vec`)
only need to read the string — they don't need ownership. Returning `String` forces an
allocation. Every call to `plugin_names()` allocates a `String` per plugin name just to
display them.

**Fix:** After removing `RefCell` (issue #3), return `&str`:

```rust
pub fn name(&self) -> &str { &self.name }
```

If callers need an owned `String`, they can call `.to_string()` or `.to_owned()` explicitly —
making the allocation visible at the call site.

**Concept:** Return references when callers only need to read. Let callers opt into allocating
by calling `.to_owned()`. Returning `String` from an accessor is a common source of hidden
allocations in hot paths.

---

### 5. `plugin_names()` returns `Vec<String>` — an iterator would avoid intermediate allocations

**Location:** `Registry::plugin_names`

```rust
pub fn plugin_names(&self) -> Vec<String> {
    self.plugins.lock().unwrap()
        .iter()
        .map(|p| p.config.name())  // already allocates per plugin
        .collect()
}
```

**Problem:** Returns a fully materialized `Vec<String>`. Callers that only want to iterate
(print, check membership) allocate a full `Vec` unnecessarily.

**Minor fix:** After fixing issues #2 and #3, this could return an iterator or `Vec<&str>`:

```rust
// After fixes: borrow() instead of lock(), &str instead of String
pub fn plugin_names(&self) -> Vec<&str> {
    // Lifetime issue: the borrow is dropped when the guard is dropped.
    // For now, Vec<String> is acceptable if returning references is awkward.
    // The bigger wins are issues #1-#4.
}
```

This is marked minor because the return type is a reasonable API surface, and the lifetime
complications of returning references to data behind `RefCell` can make the fix non-trivial.
Fix the other issues first; revisit this if `plugin_names` appears in a hot path.

---

## Positive Feedback

1. **The builder pattern on `PluginConfig::with_setting` is well-designed.** Taking `self` by
   value and returning `Self` is the correct Rust idiom for mutable builder chains. No issues here.

2. **`Plugin::is_loaded()` returning `bool` is correct.** Reading through a `RefCell` borrow
   and returning a value (not a reference) is the right pattern — the borrow is scoped
   to the method body.

3. **`PluginState` enum with `Failed(String)` is better than a boolean.** Capturing the failure
   reason in the type rather than a separate `error` field is idiomatic. The variant self-documents
   that failure is a distinct, non-normal state.

4. **Tests are clear and cover the key behaviors.** The tests exercise register, load, config
   access, and plugin-to-registry back-reference. Good baseline coverage.

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | `Plugin` holds `Rc<Registry>` — creates a cycle, leaks memory | `Weak<T>` for non-owning back-references |
| 2 | Major | `Arc<Mutex<T>>` in single-threaded code — adds overhead with no benefit | `Rc<RefCell<T>>` vs `Arc<Mutex<T>>` |
| 3 | Major | `PluginConfig` wraps everything in `RefCell` unnecessarily | Use `RefCell` only when mutation through shared refs is needed |
| 4 | Minor | `name()` returns `String`, should return `&str` | Prefer references in accessors; let callers opt into allocation |
| 5 | Minor | `plugin_names()` materializes a `Vec<String>` unnecessarily | Iterator APIs avoid intermediate allocations |

## Related Concepts

- [[fundamentals/rust/pointers-and-smart-pointers]] — Rc, Weak, RefCell, Arc decision guide
- [[pitfalls/rust-rc-cycle-memory-leak]] — detecting and preventing reference cycles
- [[pitfalls/rust-arc-mutex-overkill]] — when to use Mutex vs RefCell
- [[pitfalls/rust-refcell-unnecessary]] — overuse of interior mutability
