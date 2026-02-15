# Solution: Configuration Validator Bugs

## Bugs Found

### Bug 1: Port Range Condition (Line 32)

**Buggy Code:**
```go
if cfg.Port < 0 && cfg.Port > 65535 {
```

**Problem:** Uses AND (`&&`) instead of OR (`||`). This condition is NEVER true because a number cannot be both less than 0 AND greater than 65535 simultaneously.

**Fix:**
```go
if cfg.Port < 1 || cfg.Port > 65535 {
```

**Lesson:** When checking if a value is outside a range, use OR (`||`). When checking if a value is inside a range, use AND (`&&`).

---

### Bug 2: Early Return Prevents Collecting All Errors (Line 43)

**Buggy Code:**
```go
if cfg.Timeout <= 0 {
    return []ValidationError{{
        Field:   "Timeout",
        Message: "must be positive",
    }}
}
```

**Problem:** Returns immediately on first error, preventing other validations from running. This violates the function's contract (should return ALL errors).

**Fix:**
```go
if cfg.Timeout <= 0 {
    errors = append(errors, ValidationError{
        Field:   "Timeout",
        Message: "must be positive",
    })
}
```

**Lesson:** Guard clauses are great for invalid input and early exits, but when you need to collect multiple errors, append them instead of returning early.

---

### Bug 3: Email Validation Logic (Lines 52-62)

**Buggy Code:**
```go
for i := range cfg.AdminEmails {
    if strings.Contains(cfg.AdminEmails[i], "@") {
        break // breaks on FIRST valid email
    }
}

if len(cfg.AdminEmails) == 0 {
    errors = append(errors, ValidationError{
        Field:   "AdminEmails",
        Message: "at least one admin email required",
    })
}
```

**Problems:**
1. Loop breaks on first valid email, so it doesn't verify all emails are valid
2. Only checks `len(cfg.AdminEmails) == 0`, doesn't check if emails are valid
3. Doesn't track whether ANY valid email was found

**Fix:**
```go
hasValidEmail := false
for _, email := range cfg.AdminEmails {
    if strings.Contains(email, "@") {
        hasValidEmail = true
        break
    }
}

if !hasValidEmail {
    errors = append(errors, ValidationError{
        Field:   "AdminEmails",
        Message: "at least one valid admin email required",
    })
}
```

**Lesson:** When you need to check if "at least one" item satisfies a condition, use a boolean flag and set it when you find a match.

---

## Fixed Code

```go
func ValidateConfig(cfg Config) []ValidationError {
    var errors []ValidationError

    // Fix Bug 1: Use OR (||) for out-of-range check
    if cfg.Port < 1 || cfg.Port > 65535 {
        errors = append(errors, ValidationError{
            Field:   "Port",
            Message: "must be between 1 and 65535",
        })
    }

    // Fix Bug 2: Append error instead of returning early
    if cfg.Timeout <= 0 {
        errors = append(errors, ValidationError{
            Field:   "Timeout",
            Message: "must be positive",
        })
    }

    // Fix Bug 3: Track if ANY valid email exists
    hasValidEmail := false
    for _, email := range cfg.AdminEmails {
        if strings.Contains(email, "@") {
            hasValidEmail = true
            break
        }
    }

    if !hasValidEmail {
        errors = append(errors, ValidationError{
            Field:   "AdminEmails",
            Message: "at least one valid admin email required",
        })
    }

    return errors
}
```

---

## Root Causes

| Bug | Root Cause | Category |
|-----|------------|----------|
| Port range | Logic error (AND vs OR) | Conditional |
| Early return | Premature exit | Control flow |
| Email validation | Incorrect loop logic + missing flag | Loop + State |

---

## How to Prevent These Bugs

### 1. Range Checks
When checking ranges, remember:
- **Outside range:** `x < min || x > max`
- **Inside range:** `x >= min && x <= max`

### 2. Error Collection
When you need to collect multiple errors:
```go
var errors []Error
// Check condition 1
if invalid1 {
    errors = append(errors, ...)
}
// Check condition 2
if invalid2 {
    errors = append(errors, ...)
}
return errors
```

NOT:
```go
if invalid1 {
    return error1  // ❌ Stops checking other conditions
}
```

### 3. "At Least One" Pattern
```go
found := false
for _, item := range items {
    if condition(item) {
        found = true
        break
    }
}
if !found {
    // handle error
}
```

---

## Testing Strategy

These bugs were caught by:
1. **Edge case tests** (port 1, port 65535) → found Bug 1
2. **Multiple error test** → found Bug 2
3. **Invalid email test** → found Bug 3

**Lesson:** Always test:
- Edge cases (boundary values)
- Multiple errors simultaneously
- Both positive and negative cases

---

## Related Patterns

- **Guard Clauses** - Early returns for invalid input (but not for error collection)
- **Validation Pipeline** - Collect all errors before returning
- **Flag Pattern** - Track state during iteration (hasValidEmail)
