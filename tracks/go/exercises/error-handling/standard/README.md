# Exercise: Config File Validator

## Scenario

Your team maintains a CLI tool that reads YAML-like config files before deploying services. The tool must validate the config thoroughly and report **all** validation errors at once — not just the first one. Nobody wants to fix one error, re-run, discover another, repeat. The tool should also provide structured error types so the output layer can format errors helpfully (e.g., highlight the field name, link to docs).

You're implementing the validation engine that reads a config file, checks required fields, validates field values, and returns a combined error when multiple things are wrong.

## Brief

Implement a config file validator with these components:

1. A `Config` struct with required and optional fields
2. Custom error types: `FieldError` (validation failure on a specific field), `ParseError` (malformed file), `MissingFieldError` (required field absent)
3. A `Validator` that collects all errors (not fail-fast) and returns them combined
4. A `Load` function that reads a config file, parses it, and runs validation
5. Error wrapping at each layer so callers can use `errors.Is` and `errors.As`

## Acceptance Criteria

- [ ] `Config` struct has fields: `Name` (required string), `Port` (required int, 1–65535), `Host` (required string), `Timeout` (optional int, default 30, must be > 0 if set), `Tags` (optional []string)
- [ ] `FieldError` type: has `Field string`, `Value any`, `Message string`; implements `error`
- [ ] `MissingFieldError` type: has `Field string`; implements `error`
- [ ] `ParseError` type: has `Line int`, `Message string`; implements `error`
- [ ] `ValidationError` type: wraps multiple errors (use `errors.Join` or implement `Unwrap() []error`); represents "one or more fields failed validation"
- [ ] `Validate(cfg Config) error` — returns nil if valid, or a `*ValidationError` containing all `*FieldError` and `*MissingFieldError` instances found
- [ ] `Parse(data string) (*Config, error)` — parses a simple `key=value` format; returns `*ParseError` on malformed input
- [ ] `Load(path string) (*Config, error)` — reads the file, calls `Parse`, calls `Validate`; wraps errors at each step with `fmt.Errorf("load: %w", ...)`
- [ ] `errors.Is(err, ErrInvalidConfig)` returns `true` for any `*ValidationError`
- [ ] `errors.As` can extract `*FieldError`, `*MissingFieldError`, or `*ParseError` from any error returned by `Load`
- [ ] All tests pass

## Config File Format

Simple `key=value` per line. Lines starting with `#` are comments. Empty lines are ignored.

```
# service config
name=payment-service
port=8080
host=0.0.0.0
timeout=30
tags=billing,critical
```

## Constraints

- No external packages — standard library only
- `Validate` must collect **all** errors, not stop at the first
- Custom error types must use pointer receivers
- All error types must be exported
- `errors.Join` is available (Go 1.20+)

## Concepts Exercised

- Custom error types with structured data (`FieldError`, `MissingFieldError`, `ParseError`)
- Sentinel errors (`ErrInvalidConfig`)
- Error wrapping with `%w` at each layer
- Multi-error aggregation with `errors.Join`
- `errors.Is` and `errors.As` traversal
- `Unwrap() []error` for multi-error types

## Hints

<details>
<summary>Hint 1: ValidationError wrapping multiple errors</summary>

`errors.Join` returns an error whose `Unwrap() []error` exposes all component errors. You can wrap the joined error in a `ValidationError`:

```go
type ValidationError struct {
    err error  // the joined error
}

func (e *ValidationError) Error() string {
    return fmt.Sprintf("validation failed:\n%v", e.err)
}

func (e *ValidationError) Unwrap() error {
    return e.err
}

// Usage:
joined := errors.Join(fieldErrs...)
return &ValidationError{err: joined}
```

`errors.As(validationErr, &fieldErr)` will traverse through `ValidationError.Unwrap()` → `joined.Unwrap()` → `[]error{fieldErr1, fieldErr2, ...}`.
</details>

<details>
<summary>Hint 2: ErrInvalidConfig sentinel</summary>

To make `errors.Is(err, ErrInvalidConfig)` return true for any `*ValidationError`, implement the `Is` method on `ValidationError`:

```go
var ErrInvalidConfig = errors.New("invalid config")

func (e *ValidationError) Is(target error) bool {
    return target == ErrInvalidConfig
}
```

Now `errors.Is(err, ErrInvalidConfig)` returns true whenever a `*ValidationError` is anywhere in the chain.
</details>

<details>
<summary>Hint 3: Parse function structure</summary>

```go
func Parse(data string) (*Config, error) {
    cfg := &Config{Timeout: 30}  // default timeout

    for i, line := range strings.Split(data, "\n") {
        line = strings.TrimSpace(line)
        if line == "" || strings.HasPrefix(line, "#") {
            continue
        }
        parts := strings.SplitN(line, "=", 2)
        if len(parts) != 2 {
            return nil, &ParseError{Line: i + 1, Message: fmt.Sprintf("expected key=value, got %q", line)}
        }
        key, value := strings.TrimSpace(parts[0]), strings.TrimSpace(parts[1])
        // set cfg fields based on key...
    }
    return cfg, nil
}
```
</details>

<details>
<summary>Hint 4: Validate collecting all errors</summary>

```go
func Validate(cfg Config) error {
    var errs []error

    if cfg.Name == "" {
        errs = append(errs, &MissingFieldError{Field: "name"})
    }
    if cfg.Port == 0 {
        errs = append(errs, &MissingFieldError{Field: "port"})
    } else if cfg.Port < 1 || cfg.Port > 65535 {
        errs = append(errs, &FieldError{Field: "port", Value: cfg.Port, Message: "must be between 1 and 65535"})
    }
    // ... other fields ...

    if len(errs) == 0 {
        return nil
    }
    return &ValidationError{err: errors.Join(errs...)}
}
```
</details>
