# Config Loader Exercise

## Scenario

You're building a microservice that needs to load configuration from environment variables with fallback to defaults. The config includes server settings (host, port) and operational parameters (timeouts, max connections). Invalid values should be detected early with clear error messages.

## Brief

Implement a type-safe configuration loader that:
- Parses environment variables into a strongly-typed `Config` struct
- Provides sensible defaults for missing values
- Validates ranges (e.g., port must be 1-65535)
- Returns helpful errors for invalid values

## Acceptance Criteria

- [ ] `Config` struct with fields: `Host`, `Port`, `Timeout`, `MaxConnections`
- [ ] `LoadConfig()` function that reads from a `map[string]string` (simulating env vars)
- [ ] Default values: host="localhost", port=8080, timeout=30s, maxConnections=100
- [ ] Port validation: must be 1-65535
- [ ] Timeout validation: must be positive
- [ ] Error messages include which field failed and why
- [ ] All tests pass

## Constraints

- Use Go's type system — no `interface{}` or reflection
- Parse string values to appropriate types (`int`, `time.Duration`)
- Handle missing keys gracefully (use defaults)
- Handle invalid values explicitly (return error)

## Files

- `starter/` — Scaffold with TODOs
- `starter/config_test.go` — Test suite

## Getting Started

```bash
cd starter
go test  # Should fail initially
```

## Hints

<details>
<summary>Hint 1: Parsing values</summary>

Use `strconv.Atoi()` for integers and `time.ParseDuration()` for durations. Both return `(value, error)` — check the error.

</details>

<details>
<summary>Hint 2: Default values</summary>

Check if the key exists in the map before parsing. If missing, use the default. Pattern:

```go
if raw, ok := env["PORT"]; ok {
    // parse raw
} else {
    config.Port = 8080  // default
}
```

</details>

<details>
<summary>Hint 3: Validation</summary>

After parsing, validate ranges. Return an error with context:

```go
if port < 1 || port > 65535 {
    return Config{}, fmt.Errorf("port %d out of range (1-65535)", port)
}
```

</details>

## Solution

After completing your implementation, compare with `solutions/` directory for reference implementations and variants.
