# Standard Exercise: Config File Parser

## Scenario

You're building the configuration loading layer for a deployment toolchain. The tool reads a simple TOML-like config file (key = value pairs, sections in `[brackets]`, `#` comments), validates required fields, applies type coercions, and returns a structured `Config`. Errors must be structured (callers match on variants to decide whether to abort or apply defaults), carry context (which file, which line, which key), and chain correctly so the full error path is printable.

## Brief

Implement a config file parser with structured error handling. You will define a custom error enum using `thiserror` (or manually if preferred), parse and validate a config file format, propagate errors with `?` and `From` conversions, and write a validation layer that collects multiple errors rather than stopping at the first.

This is a Cargo project exercise. Create a new project with `cargo new config-parser --name config_parser`.

## Acceptance Criteria

### 1. Error type — `ConfigError`

Define an error enum using `thiserror` with these variants:

- `Io { path: String, source: std::io::Error }` — file read failure, with path context
- `ParseError { path: String, line: usize, message: String }` — malformed syntax
- `MissingField { section: String, key: String }` — required key absent
- `InvalidValue { section: String, key: String, value: String, expected: String }` — wrong type or out of range
- `ValidationErrors(Vec<String>)` — multiple validation failures collected at once

`Display` messages must include enough context to pinpoint the problem without reading source code. Example: `"missing required field 'port' in section [server]"`.

### 2. Parser — `parse_config(path: &str) -> Result<RawConfig, ConfigError>`

`RawConfig` is `HashMap<String, HashMap<String, String>>` — a map of section name to key-value pairs.

- Lines starting with `#` (after trimming) are comments — skip them
- Blank lines are skipped
- `[section_name]` sets the current section (default section is `"default"`)
- `key = value` adds to the current section (value is trimmed; key is trimmed)
- Any other non-empty line is a `ParseError` with the file path and line number
- Use `?` to propagate `std::io::Error` from `fs::read_to_string` (the `#[from]` on `Io` handles the conversion)

### 3. Validator — `validate(raw: &RawConfig, path: &str) -> Result<Config, ConfigError>`

`Config` is a struct with at minimum:
```rust
pub struct Config {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub log_level: LogLevel,
    pub timeout_ms: u64,
}

pub enum LogLevel { Debug, Info, Warn, Error }
```

Required fields and their sections:
- `[server]` section: `host` (String), `port` (u16, range 1–65535), `max_connections` (u32)
- `[logging]` section: `log_level` (one of: debug, info, warn, error)
- `[timeouts]` section: `timeout_ms` (u64)

The validator must **collect all validation errors** before returning. A single missing field should not prevent checking for other missing fields. Return `ConfigError::ValidationErrors(errors)` if any validation fails, where `errors` is a `Vec<String>` of human-readable messages.

### 4. Public API

```rust
pub fn load_config(path: &str) -> Result<Config, ConfigError>
```

Calls `parse_config` then `validate`. Errors from either step propagate with `?`.

### 5. Tests

All tests must pass with `cargo test`:

- `test_valid_config` — parse and validate a valid config string written to a temp file, check all fields
- `test_missing_required_field` — config missing `port`, expect `ValidationErrors` containing a message about `port`
- `test_invalid_port_type` — `port = notanumber`, expect `ValidationErrors` with message about `port`
- `test_invalid_port_range` — `port = 70000`, expect `ValidationErrors` with message about `port`
- `test_unknown_log_level` — `log_level = verbose`, expect `ValidationErrors` with message about `log_level`
- `test_multiple_errors_collected` — config with both missing `port` and invalid `log_level`, expect `ValidationErrors` with two entries
- `test_parse_error_bad_line` — file with a line that is neither comment, section, nor key=value, expect `ParseError`
- `test_file_not_found` — path that does not exist, expect `Io` variant

## Constraints

- Use `thiserror` for the `ConfigError` enum
- `parse_config` must use `?` for I/O propagation (not explicit `match` on the io::Error)
- `validate` must collect ALL errors before returning — do not `?` on each field check
- No `unwrap()` in non-test code
- The `#[from]` attribute on `Io.source` must enable `?` to work on `io::Error` directly

## Hints

<details>
<summary>Hint 1: thiserror Io variant with context</summary>

When you add a `path` field alongside the `#[from]` source, you cannot use `#[from]` alone — `From` only converts the error without the extra context. Use `#[source]` instead:

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error reading '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    // ...
}
```

Then in `parse_config`:
```rust
std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
    path: path.to_string(),
    source: e,
})?;
```

</details>

<details>
<summary>Hint 2: Collecting validation errors</summary>

Don't use `?` inside the validation loop — that would return early on the first error.
Instead, accumulate into a `Vec<String>`:

```rust
let mut errors = Vec::new();

match raw.get("server").and_then(|s| s.get("port")) {
    None => errors.push("missing required field 'port' in section [server]".to_string()),
    Some(v) => match v.parse::<u16>() {
        Err(_) => errors.push(format!("invalid value '{}' for 'port': expected u16", v)),
        Ok(0) => errors.push("'port' must be > 0".to_string()),
        Ok(p) => port = Some(p),
    }
}

// ... check other fields ...

if !errors.is_empty() {
    return Err(ConfigError::ValidationErrors(errors));
}
```

</details>

<details>
<summary>Hint 3: Parsing sections and key-value pairs</summary>

```rust
let mut current_section = "default".to_string();
let mut result: HashMap<String, HashMap<String, String>> = HashMap::new();

for (line_num, line) in contents.lines().enumerate() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') { continue; }

    if line.starts_with('[') && line.ends_with(']') {
        current_section = line[1..line.len()-1].trim().to_string();
    } else if let Some((key, value)) = line.split_once('=') {
        result.entry(current_section.clone())
              .or_default()
              .insert(key.trim().to_string(), value.trim().to_string());
    } else {
        return Err(ConfigError::ParseError {
            path: path.to_string(),
            line: line_num + 1,
            message: format!("expected 'key = value' or '[section]', got '{}'", line),
        });
    }
}
```

</details>
