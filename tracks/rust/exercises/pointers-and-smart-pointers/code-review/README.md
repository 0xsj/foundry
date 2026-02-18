# Code Review Exercise: Plugin System with Shared State

## Context

A teammate opened a PR adding a plugin registry to your platform's service layer. Plugins
are loaded at startup, share read-only configuration, and can hold references to each other
for dependency injection. The registry manages their lifecycles.

The code compiles and tests pass, but there are several smart pointer choices that are
either unnecessarily heavy, incorrect, or missing. The issues range from over-engineering
(using `Arc<Mutex<T>>` in a single-threaded context) to a missing `Weak` that would
cause a memory leak in production.

## Your Task

Review `proposed.rs` as if it were a real PR. Look for:

1. **Unnecessary `Arc<Mutex<T>>`** — using thread-safe primitives where single-threaded ones suffice
2. **Missing `Weak` for back-references** — plugin holds `Rc<Registry>` creating a cycle
3. **Unnecessary `RefCell`** — interior mutability where a simple `&mut` would work
4. **`clone()` where borrowing would suffice** — allocating when a reference is enough

Record your findings in `my-review.md` using the Critical / Major / Minor structure.

## Review Checklist

Work through the code in order:

- [ ] `Registry` struct — is `Arc<Mutex<T>>` justified in this single-threaded context?
- [ ] `Plugin` struct — does the back-reference to `Registry` form a cycle?
- [ ] `PluginConfig` — does it need `RefCell` if it's never mutated after construction?
- [ ] Method bodies — any unnecessary `.clone()` calls that could be borrows?
- [ ] Overall ownership structure — is there a simpler design?

## How to Run the Code

```
rustc proposed.rs && ./proposed
```
