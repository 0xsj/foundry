# Solution — Config Loader

## Approach

The reference solution uses **explicit parsing with early returns**:
1. Start with a `Config` struct initialized to default values
2. For each environment variable key, check if it exists
3. If present, parse the string value to the target type
4. Validate the parsed value (range checks)
5. Return error early if parsing or validation fails
6. Return the populated config

This approach is clear, idiomatic Go, and handles errors explicitly.

## Key Decisions

1. **Default values in struct initialization**: Rather than checking for each missing key, we start with defaults and only override what's provided. This reduces conditional logic.

2. **Early returns for errors**: Each parsing step can fail independently. Returning immediately on error keeps the happy path linear and easy to follow.

3. **Descriptive error messages**: Errors include the field name and the invalid value. This helps debugging in production when logs show "invalid port '99999': out of range" rather than just "validation failed."

4. **Use standard library**: `strconv.Atoi` and `time.ParseDuration` are idiomatic and well-tested. No need to reinvent parsing.

## Complexity Analysis

- **Time**: O(n) where n is the number of keys in `env` map. Each lookup and parse is constant time.
- **Space**: O(1) — the Config struct size is fixed regardless of input size.

## Variants

This directory contains multiple solution approaches:

| File | Approach | Pros | Cons | When to Use |
|------|----------|------|------|-------------|
| `config.go` | Explicit parsing | Clear, idiomatic, easy to debug | Some repetition | Production code (default) |
| `variants/functional.go` | Helper closures | DRY (don't repeat yourself) | Slightly harder to follow | When you have many fields |

## Implementation Notes

- **Error wrapping**: Using `fmt.Errorf` with `%w` preserves the original error for `errors.Is` checks
- **Map lookup pattern**: `val, ok := map[key]` is the idiomatic way to check key existence
- **Validation placement**: Validation happens *after* parsing succeeds. This separates concerns (parsing vs business logic)

## Benchmarks

Not applicable for this exercise — config loading happens once at startup. Performance is not a concern.

## Further Reading

- [[error-handling]] — How Go's explicit error handling works
- [[structs-and-objects]] — Struct initialization patterns
- Go Proverb: "Clear is better than clever"
