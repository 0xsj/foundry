# Expert Review: Config File Loader

## Critical Issues

### 1. `dec.Decode` error is silently ignored — bad config loads as zero-value struct

**Location:** `LoadConfig`, line `dec.Decode(&cfg)`

```go
dec := json.NewDecoder(f)
dec.Decode(&cfg) // error return value discarded
return &cfg, nil
```

The error returned by `dec.Decode` is completely ignored. If the config file contains invalid JSON (a syntax error, a type mismatch, a truncated file), `Decode` returns an error — but the code discards it and returns `&cfg` with a partially populated or zero-value struct. The caller gets `nil` error and a `*WebhookConfig` that looks valid but has empty `ServiceName`, zero `Version`, nil `Endpoints`, and zero-value retry defaults.

The service then starts successfully, connects to no endpoints, and processes no webhooks. No error surface — it just silently does nothing.

**Fix:**

```go
dec := json.NewDecoder(f)
if err := dec.Decode(&cfg); err != nil {
    return nil, fmt.Errorf("parse config %s: %w", path, err)
}
```

Always check errors from `Decode`, `Unmarshal`, `Scan`, and any other fallible I/O operation. Go's convention is explicit error handling — every error return must be either handled or deliberately discarded with `_` (with a comment explaining why).

**Concept:** Ignoring error returns is Go's most dangerous pattern. The compiler doesn't prevent it. Static analysis tools (e.g., `errcheck`) catch it. Add `errcheck` to CI.

---

## Major Concerns

### 2. No validation after unmarshal — silently accepts broken configs

**Location:** `LoadConfig` — after `dec.Decode(&cfg)`

Even with the decode error fixed, `LoadConfig` performs no validation of the parsed values. A config file that parses successfully can still be unusable:

```json
{
    "service_name": "",
    "version": 0,
    "endpoints": [
        {"name": "prod", "url": "", "enabled": true}
    ]
}
```

This config parses without error. But:
- `ServiceName` is empty — which service is this?
- `Version` is `0` — valid JSON integer, but likely a misconfiguration
- An endpoint with `url: ""` will fail at runtime when the relay tries to POST to it

The failure surfaces much later, far from the config loading code. Debug time: 20 minutes wondering why no webhooks are delivered; answer: empty URL in config.

**Fix:** Add a `Validate()` method and call it in `LoadConfig`:

```go
func (c *WebhookConfig) Validate() error {
    if c.ServiceName == "" {
        return fmt.Errorf("service_name is required")
    }
    if c.Version < 1 {
        return fmt.Errorf("version must be >= 1, got %d", c.Version)
    }
    if len(c.Endpoints) == 0 {
        return fmt.Errorf("at least one endpoint is required")
    }
    for i, ep := range c.Endpoints {
        if ep.Name == "" {
            return fmt.Errorf("endpoint[%d]: name is required", i)
        }
        if ep.URL == "" {
            return fmt.Errorf("endpoint %q: url is required", ep.Name)
        }
        if ep.Retry.MaxAttempts < 0 {
            return fmt.Errorf("endpoint %q: max_attempts cannot be negative", ep.Name)
        }
    }
    return nil
}

func LoadConfig(path string) (*WebhookConfig, error) {
    // ... open, decode ...
    if err := cfg.Validate(); err != nil {
        return nil, fmt.Errorf("invalid config %s: %w", path, err)
    }
    return &cfg, nil
}
```

**Concept:** Parsing and validation are distinct phases. `json.Unmarshal`/`Decode` validates syntax and types. Business-rule validation (required fields, value ranges, cross-field consistency) is always a separate step. The closer to the I/O boundary this validation happens, the easier the rest of the code is to reason about.

---

### 3. `interface{}` for `Headers` and `Extensions` — unsafe and unergonomic

**Location:** `EndpointConfig.Headers` and `WebhookConfig.Extensions`

```go
Headers    interface{}  `json:"headers"`    // could be anything
Extensions interface{}  `json:"extensions"` // could be anything
```

`interface{}` (or `any`) tells the compiler and the reader nothing about what these fields contain. When callers use them:

```go
// Working with Headers — every access requires type assertion chains
if headers, ok := ep.Headers.(map[string]interface{}); ok {
    for k, v := range headers {
        if s, ok := v.(string); ok {
            req.Header.Set(k, s)
        }
    }
}
```

This is fragile and verbose. If the JSON has `"headers": ["not", "a", "map"]`, the outer type assertion fails silently and no headers are set — no error, no log.

**Fix for `Headers`:** Use `map[string]string`. The headers in a webhook request are always `string → string`. The JSON schema is known. Use the right type:

```go
Headers map[string]string `json:"headers,omitempty"`
```

Now `ep.Headers` is directly usable: `req.Header.Set(k, v)` in a range loop, no type assertions needed.

**Fix for `Extensions`:** Use `json.RawMessage`. Extensions exist for forward compatibility — the config format may add new fields in future versions, and old readers should preserve and forward them without losing data:

```go
Extensions json.RawMessage `json:"extensions,omitempty"`
```

`json.RawMessage` keeps the JSON bytes verbatim. Your code doesn't need to understand the extensions — it can forward them to another service or store them for later. If you used `interface{}`, the JSON is deserialized to `map[string]interface{}` and then re-serialized — potentially losing type information (e.g., large integers become `float64`, then get precision-truncated when re-marshaled to JSON).

**Concept:**
- Use concrete types (`map[string]string`, `[]string`, named structs) when you know the shape.
- Use `json.RawMessage` when the shape is unknown or variable and you need to preserve it faithfully.
- Use `interface{}` / `any` as a last resort, never as a default for "I don't know what this will be."

---

## Minor Suggestions

### 4. `MergeRetry` uses zero-value as "not set" — ambiguous for explicitly-zero configs

**Location:** `MergeRetry` function

```go
if merged.MaxAttempts == 0 {
    merged.MaxAttempts = defaults.MaxAttempts
}
```

`MaxAttempts == 0` could mean either "this endpoint didn't specify a retry count" or "this endpoint explicitly wants zero retries (no retries)". There's no way to tell the difference.

A team member might set `"max_attempts": 0` in a config to mean "disable retries" — a reasonable interpretation. But `MergeRetry` silently replaces it with the default.

**Better:** Use `*int` for optional fields in `EndpointConfig.Retry`:

```go
type EndpointRetryConfig struct {
    MaxAttempts *int `json:"max_attempts,omitempty"` // nil = use default, 0 = no retries
    BackoffMs   *int `json:"backoff_ms,omitempty"`
    TimeoutMs   *int `json:"timeout_ms,omitempty"`
}
```

Then `MergeRetry` checks `nil` (not set) vs pointer-to-0 (explicitly set):

```go
if ep.Retry.MaxAttempts == nil {
    merged.MaxAttempts = defaults.MaxAttempts
} else {
    merged.MaxAttempts = *ep.Retry.MaxAttempts
}
```

This is more verbose but unambiguous. For config files, the added clarity is worth it.

---

### 5. `LoadConfig` should use `DisallowUnknownFields` to catch config typos

**Location:** `LoadConfig`, decoder setup

```go
dec := json.NewDecoder(f)
```

By default, `json.Decoder` ignores unknown fields. A typo like `"max_attemps"` (missing t) silently loads as `MaxAttempts: 0` — which then gets overridden by the default, making the config appear correct while the actual value is silently ignored.

**Fix:**

```go
dec := json.NewDecoder(f)
dec.DisallowUnknownFields()
```

With `DisallowUnknownFields`, `"max_attemps"` causes a decode error with the message `json: unknown field "max_attemps"`. Typos are caught at load time, not discovered in production.

**When to use this:** `DisallowUnknownFields` is appropriate when you control both the config file producer and the consumer (i.e., internal config files). Do NOT use it when consuming external API responses from services you don't control — they may add new fields in their API at any time, and you want old client code to continue working gracefully.

---

## Positive Feedback

- `MergeRetry` is a clean, readable pattern for layered configuration with defaults
- `GetEnabledEndpoints` is simple and correct — good use of a nil-safe append pattern
- Struct definitions are well-organized with clear separation of concerns
- `defer f.Close()` is in the right place
- `fmt.Errorf` with `%w` for error wrapping is correct Go style

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | `dec.Decode` error silently discarded — zero-value config returned | Always check error returns |
| 2 | Major | No validation after unmarshal — accepts empty URLs, zero versions | Parse ≠ validate |
| 3 | Major | `interface{}` for Headers/Extensions — type-unsafe, loses data on round-trip | Use `map[string]string` / `json.RawMessage` |
| 4 | Minor | `MergeRetry` zero-value ambiguity — can't distinguish unset vs explicit zero | Optional fields with `*int` |
| 5 | Minor | No `DisallowUnknownFields` — typos silently ignored | Strict config loading |
