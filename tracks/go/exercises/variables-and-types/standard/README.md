# Exercise: Service Config Loader

## Scenario

You're building a microservice that needs to load its configuration from environment variables at startup. The config has required fields (the service crashes if they're missing) and optional fields (they fall back to sensible defaults). Your team needs a clean, type-safe way to distinguish "not set" from "set to the zero value" — for example, is `maxRetries=0` intentional, or did someone forget to set it?

## Brief

Implement a `Config` struct and a `LoadConfig` function that:

1. Parses raw string environment variables into typed Go values
2. Applies default constants where values are missing
3. Uses pointer fields to distinguish "not set" (`nil`) from "explicitly set to zero"
4. Validates required fields and returns clear errors

## Acceptance Criteria

- [ ] `Config` struct has fields for: `Host` (string), `Port` (int), `Debug` (bool), `MaxRetries` (*int — optional), `TimeoutSec` (float64), `Environment` (string)
- [ ] Constants defined for: `DefaultPort`, `DefaultTimeoutSec`, `DefaultEnvironment`, `DefaultMaxRetries`
- [ ] `LoadConfig(env map[string]string) (Config, error)` parses the env map into a `Config`
- [ ] `Host` is required — return an error if missing or empty
- [ ] `Port` defaults to `DefaultPort` if not in env. If present but not a valid integer, return an error
- [ ] `MaxRetries` is `nil` when not set (distinguishable from `*int` pointing to `0`)
- [ ] `Debug` parses `"true"/"false"/"1"/"0"` — defaults to `false`
- [ ] `TimeoutSec` defaults to `DefaultTimeoutSec` if not set
- [ ] `Environment` defaults to `DefaultEnvironment` if not set
- [ ] `Config.String()` returns a human-readable summary (format up to you)
- [ ] All type conversions are explicit — no implicit casts

## Constraints

- Use only the standard library (`strconv`, `fmt`)
- No third-party config libraries
- Think about what zero values mean for each field type

## Concepts Exercised

- `var` vs `:=` declarations
- Constants and `iota` (if you want to model environments as an enum)
- Struct types with mixed field types
- Pointer fields for optional values (`*int` vs `int`)
- Zero value awareness
- Explicit type conversions (`strconv.Atoi`, `strconv.ParseBool`, `strconv.ParseFloat`)
- Value semantics (Config is returned by value)

## Hints

<details>
<summary>Hint 1: Pointer fields for optionality</summary>

```go
type Config struct {
    MaxRetries *int  // nil = not set, &0 = explicitly zero
}

// To set it:
retries := 5
cfg.MaxRetries = &retries
```
</details>

<details>
<summary>Hint 2: Parsing with strconv</summary>

```go
portStr := env["PORT"]
port, err := strconv.Atoi(portStr)
if err != nil {
    return Config{}, fmt.Errorf("invalid PORT: %w", err)
}
```
</details>

<details>
<summary>Hint 3: Checking "present but empty" vs "not present"</summary>

```go
// These are different:
val, exists := env["KEY"]  // exists tells you if the key was in the map
if !exists {
    // use default
} else if val == "" {
    // key exists but empty — might be an error
}
```
</details>
