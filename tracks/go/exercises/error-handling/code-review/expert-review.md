# Expert Review: HTTP Client Error Handling

## Critical Issues

### 1. `httpError` is unexported — callers cannot use `errors.As`

**Location:** `type httpError struct`

The `httpError` type is lowercase, making it package-private. Every consumer of this package receives errors from `Do()` but has no way to type-assert or use `errors.As` on them because they can't name the type.

```go
// From the caller's package:
resp, err := httpclient.Do(cfg, "GET", "/users/123", nil)
if err != nil {
    var httpErr *httpclient.httpError  // COMPILE ERROR: cannot refer to unexported type
    if errors.As(err, &httpErr) {
        fmt.Println(httpErr.StatusCode)
    }
}
```

The only workaround is to use `IsNotFound` and `IsServerError` — which are string-matching functions (see Issue 8). This is a design trap: the type is there but inaccessible, forcing brittle string parsing.

**Fix:** Export the type:

```go
// HTTPError is the structured error for non-2xx responses.
type HTTPError struct {
    StatusCode int
    Body       string
    URL        string
}

func (e *HTTPError) Error() string {
    return fmt.Sprintf("HTTP %d from %s: %s", e.StatusCode, e.URL, e.Body)
}
```

Now callers can:

```go
var httpErr *httpclient.HTTPError
if errors.As(err, &httpErr) {
    switch httpErr.StatusCode {
    case 404:
        // handle not found
    case 429:
        // handle rate limit
    case 503:
        // handle service unavailable
    }
}
```

This is far more maintainable than string matching and handles every status code, not just the two special-cased in `IsNotFound`/`IsServerError`.

---

### 2. `panic` for nil config — library code must not panic for caller errors

**Location:** `Do`, the nil config check

```go
if cfg == nil {
    panic("config must not be nil")
}
```

The PR description justifies this as "we should never call Do() without a config" — but that's precisely the kind of input that should return an error, not panic. Nil inputs are a caller mistake, and library code must handle caller mistakes gracefully by returning errors.

In a web service, a panic in a goroutine propagates to the `http.Server`'s recover handler (if one exists) and turns the request into a 500. If there's no recover handler, the entire server process exits. The caller (the request handler) has no ability to distinguish "config was nil" from any other error.

Notice the inconsistency: the very next check does it correctly:

```go
if method == "" {
    return nil, errors.New("method is required")  // returns error — correct
}
```

**Fix:**

```go
if cfg == nil {
    return nil, errors.New("config must not be nil")
}
```

**When is panic acceptable in a library?**
- `mustXxx` constructors for hardcoded values (`regexp.MustCompile("^[a-z]+$")`)
- Internal invariant violations that represent bugs in the library itself (not caller mistakes)
- Both are typically used only in package-level `var` initialization

---

### 3. `%v` in retry loop severs the error chain

**Location:** `Do`, inside the retry loop

```go
lastErr = fmt.Errorf("attempt %d: %v", attempt+1, err)
```

`doOnce` can return a `*HTTPError` (non-2xx response) or a network error (which might be a `*url.Error` wrapping a `context.DeadlineExceeded`). Using `%v` instead of `%w` converts both into plain strings, making the final `lastErr` opaque.

Callers who write:

```go
err := Do(cfg, "GET", "/resource", nil)
var httpErr *HTTPError
if errors.As(err, &httpErr) { ... }  // always false — chain is severed at %v
```

are in for a surprise. The `*HTTPError` is rendered as text and discarded.

**Fix:**

```go
lastErr = fmt.Errorf("attempt %d: %w", attempt+1, err)
```

Note: wrapping with `%w` through multiple retries creates a chain of wrapped errors. Only the *last* retry's error is in `lastErr`. If you want all retry errors, consider a different structure:

```go
var errs []error
for attempt := 0; attempt <= cfg.MaxRetries; attempt++ {
    resp, err := doOnce(cfg, method, path, body)
    if err != nil {
        errs = append(errs, fmt.Errorf("attempt %d: %w", attempt+1, err))
        continue
    }
    return resp, nil
}
return nil, fmt.Errorf("all retries failed: %w", errors.Join(errs...))
```

This preserves all errors and makes `errors.As` find the `*HTTPError` from any attempt.

---

## Major Concerns

### 4. `IsNotFound` and `IsServerError` use string matching

**Location:** `IsNotFound`, `IsServerError`

```go
func IsNotFound(err error) bool {
    return err != nil && strings.Contains(err.Error(), "HTTP 404")
}
```

String matching on error messages is fragile:
- If the error message format ever changes, these silently break
- They only handle the cases the author anticipated (404, 5xx) — a caller who needs to handle 401, 429, or 503 specifically must copy this pattern
- They don't compose with wrapped errors correctly: `strings.Contains(err.Error(), "HTTP 404")` might match a 404 buried inside a log message that was accidentally put in an error string

Since `HTTPError` is exported (once fixed), the idiomatic approach is:

```go
func IsHTTPStatus(err error, code int) bool {
    var httpErr *HTTPError
    return errors.As(err, &httpErr) && httpErr.StatusCode == code
}

func IsNotFound(err error) bool { return IsHTTPStatus(err, 404) }
func IsServerError(err error) bool {
    var httpErr *HTTPError
    return errors.As(err, &httpErr) && httpErr.StatusCode >= 500
}
```

Or even simpler: delete these helpers and let callers use `errors.As` directly with the exported type. Helper functions only add value if they encode non-trivial logic.

---

### 5. `%v` in `doOnce` severs `*url.Error` and timeout detection

**Location:** `doOnce`, `http.NewRequest` and `client.Do` error returns

```go
return nil, fmt.Errorf("build request: %v", err)
// ...
return nil, fmt.Errorf("execute request: %v", err)
```

`client.Do` returns a `*url.Error` on failure. When the context deadline is exceeded, `*url.Error.Err` is `context.DeadlineExceeded`. If this is wrapped with `%w`, callers can detect timeouts:

```go
if errors.Is(err, context.DeadlineExceeded) {
    // retry with longer timeout or report "service is slow"
}
```

With `%v`, this information is lost. Every network failure looks the same.

---

## Minor Suggestions

### 6. Non-idiomatic error message in `Do`

**Location:** `Do`, the final error return

```go
return nil, fmt.Errorf("All %d attempts failed. Last error: %v", cfg.MaxRetries+1, lastErr)
```

Go error messages should be:
- Lowercase (not "All")
- No trailing period
- Concise — "Last error:" is redundant if the cause is wrapped

**Fix:**

```go
return nil, fmt.Errorf("all %d attempts failed: %w", cfg.MaxRetries+1, lastErr)
```

---

### 7. Inconsistency: `io.ReadAll` error uses `%w` but others use `%v`

**Location:** `doOnce`, read response body

```go
return nil, fmt.Errorf("read response body: %w", err)  // %w — correct
```

This is the only place in the file using `%w`. All other `fmt.Errorf` calls use `%v`. This inconsistency suggests the author knows about `%w` but hasn't applied it consistently. Whoever reviews this should leave a comment asking for a consistent policy.

---

## Positive Feedback

- The `*httpError` / `HTTPError` struct design is correct — a typed error with structured data is exactly the right approach for HTTP errors that callers need to inspect
- Deferring `httpResp.Body.Close()` immediately after checking `err` is correct resource management
- The `backoff` helper is appropriately extracted — retry delay logic doesn't belong inline
- Using `MaxRetries` in the loop bound (`attempt <= cfg.MaxRetries`) gives the expected behavior (MaxRetries=0 means one attempt, MaxRetries=3 means 4 attempts) — this is a reasonable convention if documented

---

## Summary

| # | Severity | Location | Issue |
|---|----------|----------|-------|
| 1 | Critical | `httpError` type | Unexported — callers cannot use `errors.As` |
| 2 | Critical | `Do`, nil config | `panic` instead of error return — library must not panic for caller mistakes |
| 3 | Critical | `Do`, retry loop | `%v` severs error chain — `errors.As(*HTTPError)` never works after retries |
| 4 | Major | `IsNotFound`, `IsServerError` | String-matching on error messages — fragile, doesn't scale to all status codes |
| 5 | Major | `doOnce` | `%v` on `net/http` errors — timeout detection (`context.DeadlineExceeded`) broken |
| 6 | Minor | `Do`, final return | Non-idiomatic error message: capitalized, trailing period, `%v` |
| 7 | Minor | `doOnce` | Inconsistent use of `%w` vs `%v` — only `readAll` error uses `%w` |
