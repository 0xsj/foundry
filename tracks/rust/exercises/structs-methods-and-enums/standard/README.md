# Standard Exercise: Config File Parser

## Scenario

You're building a configuration loader for a service mesh control plane. It reads
a simplified TOML-like config format where values can be strings, integers, booleans,
lists of values, or nested sections. The loader must parse raw string inputs into a
typed `ConfigValue` enum, assemble them into a `Config` tree, and provide methods
to query, validate, and merge configs from multiple sources (e.g., default config
overridden by environment-specific config).

## Brief

Implement a `ConfigValue` enum that covers all value types, a `Config` struct that
holds a named map of key-to-value pairs, and a set of methods and functions for
constructing, querying, validating, and merging configs.

## Acceptance Criteria

1. **`ConfigValue` enum** with variants:
   - `Str(String)` — a string value
   - `Int(i64)` — an integer value
   - `Bool(bool)` — a boolean value
   - `List(Vec<ConfigValue>)` — a list of values (mixed types allowed)
   - `Section(Config)` — a nested config section
   - Derives: `Debug`, `Clone`, `PartialEq`

2. **`Config` struct** with fields:
   - `name: String`
   - `values: std::collections::HashMap<String, ConfigValue>`
   - Derives: `Debug`, `Clone`, `PartialEq`

3. **`Config::new(name: &str)`** — constructor
   - Creates an empty config with the given name

4. **`Config::set(&mut self, key: &str, value: ConfigValue)`**
   - Inserts or overwrites a key

5. **`Config::get(&self, key: &str) -> Option<&ConfigValue>`**
   - Returns a reference to the value if the key exists

6. **`Config::get_str(&self, key: &str) -> Option<&str>`**
   - Returns the inner `&str` if the key exists and is a `ConfigValue::Str`
   - Returns `None` if the key is missing or is a different variant

7. **`Config::get_int(&self, key: &str) -> Option<i64>`**
   - Returns the inner `i64` if the key exists and is a `ConfigValue::Int`

8. **`Config::get_bool(&self, key: &str) -> Option<bool>`**
   - Returns the inner `bool` if the key exists and is a `ConfigValue::Bool`

9. **`Config::validate(&self) -> Result<(), Vec<String>>`**
   - Validates that the config contains the required keys: `"host"`, `"port"`, `"service_name"`
   - Validates that `"port"`, if present, is a `ConfigValue::Int` with a value in 1..=65535
   - Returns `Ok(())` if valid, or `Err(Vec<String>)` with all validation errors found
   - Collect ALL errors, not just the first one

10. **`merge(base: Config, overrides: Config) -> Config`**
    - Top-level function (not a method)
    - Returns a new `Config` with the name of `base`
    - All keys from `base` are present in the result
    - Keys in `overrides` overwrite keys in `base`
    - Keys in `overrides` not in `base` are also included

11. **`parse_value(s: &str) -> ConfigValue`**
    - Parses a raw string into the most appropriate `ConfigValue`:
      - `"true"` or `"false"` (case-insensitive) → `ConfigValue::Bool`
      - Parseable as `i64` → `ConfigValue::Int`
      - Anything else → `ConfigValue::Str`

## Constraints

- No external crates — stdlib only
- All tests in the starter file must pass against your implementation
- `validate` must return ALL errors, not just the first

## Hints

<details>
<summary>Hint 1: Matching on a reference to an enum</summary>

When your function takes `&self` and you want to match on a field:

```rust
fn get_str(&self, key: &str) -> Option<&str> {
    match self.values.get(key) {
        Some(ConfigValue::Str(s)) => Some(s.as_str()),
        _ => None,
    }
}
```

`self.values.get(key)` returns `Option<&ConfigValue>`. Matching on `Some(ConfigValue::Str(s))`
destructures through the reference. The `s` you bind is a `&String`, so `.as_str()` gets you `&str`.

</details>

<details>
<summary>Hint 2: Collecting all validation errors</summary>

Instead of returning early on the first error, push all errors into a Vec:

```rust
let mut errors: Vec<String> = Vec::new();

if self.get("host").is_none() {
    errors.push(String::from("missing required key: host"));
}
// ... more checks ...

if errors.is_empty() { Ok(()) } else { Err(errors) }
```

</details>

<details>
<summary>Hint 3: merge function ownership</summary>

`merge` takes both configs by value (not reference). It can freely modify them.
The simplest approach: start with `base.values`, then extend with `overrides.values`.

```rust
fn merge(mut base: Config, overrides: Config) -> Config {
    for (key, value) in overrides.values {
        base.values.insert(key, value);
    }
    base
}
```

This works because `HashMap::insert` overwrites existing keys.

</details>

<details>
<summary>Hint 4: parse_value ordering</summary>

Try bool first (before integer), because `"1"` and `"0"` parse as valid i64 but not
as the booleans you probably intend them to mean. Use `to_lowercase()` for case-insensitive
bool comparison.

</details>
