# Expert Review: Rate Limiter Configuration

## Critical Issues

### 1. Nil map panic in `Register()` (line 49)

`New()` initializes `rl.defaults` but never initializes `rl.tenants`. First call to `Register` writes to a nil map → panic.

```go
// Bug
rl := &RateLimiter{}  // tenants is nil

// Fix
rl := &RateLimiter{
    tenants: make(map[string]TenantConfig),
}
```

**Concept:** Map zero value is `nil`. Nil maps are readable (return zero values) but panic on write. Always `make()` maps you intend to write to.

### 2. `DisableAll()` modifies copies, not originals (line 87-89)

The `range` over a map copies each value into the loop variable. Setting `cfg.Enabled = false` modifies the copy, not the entry in the map. After the loop, nothing has changed.

```go
// Bug: cfg is a copy
for _, cfg := range rl.tenants {
    cfg.Enabled = false  // modifies local copy, map is unchanged
}

// Fix: write back to the map
for id, cfg := range rl.tenants {
    cfg.Enabled = false
    rl.tenants[id] = cfg
}
```

**Concept:** Go structs have value semantics. `range` gives you a copy. You must write the modified value back to the map (or use pointers to structs as map values).

---

## Major Concerns

### 3. `GetConfig()` returns pointer to local copy (line 82-84)

`cfg := rl.tenants[tenantID]` copies the struct into a local variable. `return &cfg` returns a pointer to that local copy, not to the map entry. The caller gets a detached snapshot — mutations via the pointer don't affect the stored config, and subsequent changes to the map don't update the pointer. Worse, if the tenant doesn't exist, this returns a pointer to a zero-value `TenantConfig` with no indication of "not found".

```go
// Bug: pointer to a local copy, no existence check
func (rl *RateLimiter) GetConfig(tenantID string) *TenantConfig {
    cfg := rl.tenants[tenantID]
    return &cfg
}

// Fix: return value + bool, or return pointer to map value via indirection
func (rl *RateLimiter) GetConfig(tenantID string) (TenantConfig, bool) {
    cfg, ok := rl.tenants[tenantID]
    return cfg, ok
}
```

**Concept:** Pointer to a local variable is valid in Go (escape analysis moves it to the heap), but it's semantically misleading here. The caller thinks they're getting a reference to the stored config; they're getting a disconnected copy.

### 4. Integer overflow in `ValidateLimit()` (line 99)

`perSecond * 60` can overflow `int` for large inputs. On 64-bit systems this requires a very large value, but the function is a validation boundary — it should be robust. Also, negative values of `perSecond` can produce a negative `perMinute` that passes the `> 0` check after overflow.

```go
// Bug: perSecond = math.MaxInt/30 would overflow
perMinute := perSecond * 60

// Fix: check before multiplying
func ValidateLimit(perSecond int) bool {
    if perSecond <= 0 {
        return false
    }
    if perSecond > 1_000_000/60 {
        return false
    }
    return true
}
```

**Concept:** Go integers wrap silently on overflow — no panic, no error. Always validate at boundaries before arithmetic.

---

## Minor Suggestions

### 5. `TierFromString` returns `math.MinInt` for unknown (line 115)

Using `math.MinInt` as a sentinel value is fragile — it's a valid `int` that could accidentally be used as a real `Tier`. Better to return an error or use the `(Tier, error)` pattern, or define an explicit `Unknown` tier.

```go
// Better: explicit unknown + error return
const Unknown Tier = -1

func TierFromString(s string) (Tier, error) {
    switch s {
    case "free":
        return Free, nil
    case "pro":
        return Pro, nil
    case "enterprise":
        return Enterprise, nil
    }
    return Unknown, fmt.Errorf("unknown tier: %q", s)
}
```

**Concept:** Sentinel values are a code smell when Go's multiple return values give you a clean way to signal failure. `iota` enums should have an explicit invalid/unknown value if needed.

### 6. `ScaleLimit` silently truncates float → int (line 74)

`int(scaled)` truncates toward zero. `ScaleLimit("t1", 1.5)` on a 100 req/min limit gives 150 (fine), but `ScaleLimit("t1", 0.7)` on 100 gives 70.0 → 70 (fine), while edge cases like `ScaleLimit("t1", 1.1)` on 61 gives 67.1 → 67, losing precision silently. Consider using `math.Round` for clearer intent.

```go
cfg.RequestsPerMin = int(math.Round(scaled))
```

**Concept:** `int()` on a float truncates, it doesn't round. This is an explicit conversion in Go, but the behavior might surprise the caller.

---

## Positive Feedback

- Clean use of `iota` for the `Tier` enum — idiomatic Go
- Good separation between defaults and per-tenant overrides
- The read-modify-writeback pattern in `UpdateLimit` is correct (unlike `DisableAll`)
- Error returns on not-found cases in `UpdateLimit` and `ScaleLimit` are good
- `TenantConfig` struct is well-organized with clear field names

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | Nil map panic in `Register` | Zero values |
| 2 | Critical | `DisableAll` modifies copies | Value semantics |
| 3 | Major | `GetConfig` returns detached pointer | Pointer to local copy |
| 4 | Major | Integer overflow in `ValidateLimit` | Silent overflow |
| 5 | Minor | `math.MinInt` sentinel for unknown tier | Sentinel vs error return |
| 6 | Minor | Float truncation in `ScaleLimit` | Explicit type conversion |
