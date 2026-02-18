# Solution: Config File Validator

## Approach

The solution uses three layers of error handling working together:

1. **Custom error types with data** (`FieldError`, `MissingFieldError`, `ParseError`) — typed errors that callers can extract via `errors.As` to get structured information
2. **Sentinel + `Is` method** (`ErrInvalidConfig`) — a simple sentinel that `*ValidationError` claims via the `Is` method, so callers can use `errors.Is` without knowing the concrete type
3. **Wrapping at every layer** — `fmt.Errorf("load %q: %w", path, err)` preserves the full chain while adding context

## Key Decisions

### ValidationError owns the ErrInvalidConfig check

```go
func (e *ValidationError) Is(target error) bool {
    return target == ErrInvalidConfig
}
```

This is the non-obvious design insight. Instead of making `ErrInvalidConfig` a type (which would require callers to use `errors.As`), we make `*ValidationError` declare "yes, I am an ErrInvalidConfig". Now callers get a simple `errors.Is` check while still being able to use `errors.As` if they need the full type.

Compare these caller experiences:

```go
// Option A: ErrInvalidConfig as type (no Is method)
var valErr *ValidationError
if errors.As(err, &valErr) {
    // only works if caller knows about *ValidationError
}

// Option B: ErrInvalidConfig as sentinel + Is method (this solution)
if errors.Is(err, ErrInvalidConfig) {
    // simple check — works from any layer
}
// AND
var valErr *ValidationError
if errors.As(err, &valErr) {
    // still works if you need the full type
}
```

Option B gives callers both choices.

### errors.Join for multi-error aggregation

```go
return &ValidationError{err: errors.Join(errs...)}
```

`errors.Join` returns an error with `Unwrap() []error`. This lets `errors.As` do **breadth-first traversal** through all collected errors. A caller can extract a specific `*FieldError` for the `port` field even if there are three other errors in the collection.

### The fileReader dependency injection

```go
type fileReader func(path string) ([]byte, error)

func Load(path string, reader fileReader) (*Config, error)
```

Rather than calling `os.ReadFile` directly inside `Load`, we accept a function. In production: `Load("config.yaml", os.ReadFile)`. In tests: inject a fake that returns controlled data without touching the filesystem. This is Go's idiomatic approach to testable I/O — no interface needed, a function type suffices.

### Permissive parsing: unknown keys are ignored

The parser silently ignores unknown keys. This is a deliberate design choice for config files — it allows adding new keys to a config without breaking older versions of the binary. If strict validation is needed, the `Validate` step is the right place to add it, not the parser.

## Comparison of Approaches

| Aspect | This solution | Alternative: error strings |
|--------|---------------|---------------------------|
| Caller access to field name | `errors.As(*FieldError)` → `.Field` | Parse the error string (fragile) |
| Sentinel check | `errors.Is(err, ErrInvalidConfig)` | `strings.Contains(err.Error(), "invalid")` (fragile) |
| Multi-error | `errors.Join` + traversal | First error wins, rest discarded |
| Test maintainability | Test behavior, not error strings | Tests break if message wording changes |

## The Full Chain

When `Load("prod.yaml", reader)` fails validation, the error chain looks like:

```
fmt.Errorf wrapper ("load \"prod.yaml\": ...")
  └── *ValidationError ("validation failed:\n...")
        └── errors.Join result (Unwrap() []error)
              ├── *MissingFieldError{Field: "host"}
              └── *FieldError{Field: "port", Value: 99999, ...}
```

Any of these is reachable by the caller:

```go
errors.Is(err, ErrInvalidConfig)  // true — via ValidationError.Is
errors.As(err, &valErr)           // true — *ValidationError
errors.As(err, &fieldErr)         // true — *FieldError (breadth-first through join)
strings.Contains(err.Error(), "prod.yaml")  // true — from the fmt.Errorf wrapper
```

## Related Concepts

- [[fundamentals/error-handling]] — Cross-language comparison
- [[fundamentals/go/error-handling]] — Go-specific patterns
- [[patterns/dependency-injection]] — fileReader pattern is a simple form of DI
- [[pitfalls/go-swallowed-errors]] — What happens when you forget to check errors
