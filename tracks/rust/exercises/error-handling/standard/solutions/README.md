# Solution Notes: Config File Parser

## Approach

The solution is structured in three layers:

1. **`parse_config`** — syntax only. Reads the file, splits into sections and key-value pairs, returns `RawConfig` (all strings). Any I/O or syntax error short-circuits immediately with `?`.

2. **`validate`** — semantics only. Receives `RawConfig`, applies type coercions, range checks, and enum parsing. Deliberately avoids `?` to collect all validation errors.

3. **`load_config`** — composes the two with `?` propagation.

This separation keeps concerns clean and makes testing straightforward: you can unit test the parser with raw strings and the validator with in-memory maps.

## Key Decisions

### Why `#[source]` not `#[from]` on `ConfigError::Io`

`#[from]` generates a `From<io::Error>` impl that constructs the variant from just the error. But `ConfigError::Io` has a `path` field — `From` cannot inject that. Using `#[source]` instead marks the field as the error chain source without generating `From`. We use `map_err` manually to provide the path.

This is a common pattern: when your error variants carry context beyond the wrapped error, use `#[source]`, not `#[from]`.

### Why `get_field` closure for the repeated missing-field pattern

The `validate` function checks five required fields, each with the same "missing → push error, present → maybe parse" structure. A helper closure eliminates the repetition while keeping errors: it pushes the error message directly into `errors` and returns `Option<String>`.

An alternative is a standalone function, but a closure here conveniently captures `&mut errors`.

### Why `unwrap()` is safe at the end of `validate`

After the `if !errors.is_empty() { return Err(...) }` gate, we know every `Option<T>` that was set to `Some(...)` actually has a value, because `None` always caused an error to be pushed. The `unwrap()` calls are logically unreachable — they exist because the type system cannot express "these are definitely Some after the error check." This is a known gap in type expressiveness; the alternative is `expect()` with a "BUG:" message.

### Why collect errors instead of short-circuiting

If `validate` used `?` for each field, a config file with five missing fields would report only the first one. The user fixes it, runs again, sees the second. This is a terrible developer experience. Collecting all errors at once lets the user fix the entire config in one pass. This pattern is idiomatic for form/config validation and is the right choice any time "all errors at once" is more useful than "fail fast."

## Variants and Alternatives

### Variant: Using `anyhow` for the application layer

If `load_config` were part of an application binary rather than a library, you might use `anyhow::Result` in the application code while keeping `ConfigError` for the library internals:

```rust
// In the application (not the library):
use anyhow::Context;

fn start_server() -> anyhow::Result<()> {
    let config = load_config("config.toml")
        .context("failed to load server configuration")?;
    // ...
    Ok(())
}
```

This keeps `ConfigError` structured (library callers can match on it) while letting the application use anyhow's ergonomic context wrapping.

### Variant: Typed RawConfig with serde_deserialize

A more advanced approach replaces the manual `validate` function with `serde::Deserialize` derived on `Config`. This is what the `toml` crate provides. The manual approach here is pedagogically valuable: you see exactly what type coercion and validation involves before letting a macro handle it.

## Trade-offs

| Approach | Pro | Con |
|---|---|---|
| `#[source]` + `map_err` for `Io` | Full path context in error message | Cannot use bare `?` for io::Error |
| `#[from]` for `Io` | Bare `?` works | No path context in error message |
| Collect all validation errors | Better UX for config authors | More code, can't use `?` inside validation |
| Short-circuit on first error | Simpler code | Poor UX — fix one error at a time |
| `thiserror` derive | Removes boilerplate | Requires external crate |
| Manual `Display` | No crate needed | ~30 lines of boilerplate per type |
