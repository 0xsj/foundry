# Solution: Config File Parser

## Approach

The solution models config values as a recursive enum with five variants, stores them in
a `HashMap<String, ConfigValue>` inside a `Config` struct, and implements typed accessor
methods that match on specific variants.

### Key Decisions

**`ConfigValue` is recursive via `Section(Config)`**

`Config` contains a `HashMap`, which is already heap-allocated, so the enum's size is
bounded without needing `Box`. If the variant held a plain `Vec<ConfigValue>` without
indirection, the same applies — `Vec` is a fat pointer on the stack.

This is different from a naive linked list where `enum List { Cons(i32, List), Nil }` is
infinitely large. Here, `Section(Config)` is fine because `Config` has a fixed stack size
(the `HashMap` is a pointer, not inline storage).

**Typed accessors return `Option<T>` and match on variant**

```rust
fn get_str(&self, key: &str) -> Option<&str> {
    match self.values.get(key) {
        Some(ConfigValue::Str(s)) => Some(s.as_str()),
        _ => None,
    }
}
```

The `_` wildcard covers two cases simultaneously: key missing (`None`) and key present
but wrong type (`Some(ConfigValue::Int(...))` etc.). This is idiomatic — no need to
separately handle both.

**`validate` collects all errors**

Using a `Vec<String>` accumulator instead of early return gives callers the full picture.
This is the standard pattern for config validators and form validation. The check for
`port` uses a nested match to distinguish "missing" (already reported) from "wrong type
or out of range" (new error).

**`merge` mutates the base in place**

Taking both arguments by value (not reference) means we own them. Adding `mut` to `base`
lets us directly extend its HashMap with overrides without any cloning. This is more
efficient than cloning `base` first.

**`parse_value` tries Bool before Int**

"true" and "false" also parse as valid `i64` in some implementations (if you check for
"1"/"0"). By checking bool literals first, we ensure they're classified correctly.

## Variants Comparison

| Approach | Trade-off |
|---|---|
| `HashMap<String, ConfigValue>` (used) | Flexible, unordered. Key lookup is O(1). |
| `Vec<(String, ConfigValue)>` | Preserves insertion order. Key lookup is O(n). |
| Separate typed fields on the struct | Fast, type-safe, but inflexible — can't add new keys at runtime. Good for strongly-typed configs. |

## Performance Notes

- `HashMap` lookup is O(1) average, O(n) worst case (hash collision)
- `validate` iterates required keys once — O(k) where k is the number of required keys
- `merge` is O(n) where n is the number of keys in `overrides`
- `ConfigValue::Section(Config)` nests a full HashMap — deep configs use proportional
  heap allocations

## What to Try Next

- Add a `get_section(&self, key: &str) -> Option<&Config>` typed accessor
- Add `Config::keys(&self) -> impl Iterator<Item = &str>` to enumerate keys
- Connect to the Builder pattern module: make a `ConfigBuilder` that validates on `build()`
- Connect to error handling: replace `Vec<String>` errors with a custom error enum
