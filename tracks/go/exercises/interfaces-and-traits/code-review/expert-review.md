# Expert Review: Logging Middleware PR

## Summary

The PR correctly implements request/response logging and compiles without errors. No bugs, panics, or behavioral problems. However, all three interface-related issues are design problems: the `Logger` interface is larger than what the middleware uses, `RegisterHandler` accepts a concrete type instead of an interface, and `MetricsCollector` is a single-method interface where a function type would be simpler and more composable.

---

## Critical Issues

None. The code is correct and safe.

---

## Major Concerns

### 1. `LoggingMiddleware` accepts the full `Logger` interface but only uses `Log()`

**Location:** `NewLoggingMiddleware` parameter, `LoggingMiddleware.logger` field

`LoggingMiddleware` only ever calls `lm.logger.Log(...)`. It never calls `Flush()` or `Close()`. But it accepts `Logger` (3 methods), forcing anyone who wants to use `LoggingMiddleware` to provide all three methods.

**Impact on testability:**

```go
// To test LoggingMiddleware, you need a test Logger:
type testLogger struct {
    entries []string
}
func (t *testLogger) Log(level, msg string, fields map[string]any) {
    t.entries = append(t.entries, msg)
}
func (t *testLogger) Flush() error { return nil }   // NEVER CALLED — just required by interface
func (t *testLogger) Close() error { return nil }   // NEVER CALLED — just required by interface
```

You're implementing two methods that are never used, just to satisfy the interface. That's interface pollution.

**The simple function approach is often better for single-operation loggers:**

```go
type LogFunc func(level, message string, fields map[string]any)
```

But if you prefer an interface, define the minimal one you actually use:

```go
// LogWriter is the minimal logging capability used by LoggingMiddleware.
type LogWriter interface {
    Log(level, message string, fields map[string]any)
}

func NewLoggingMiddleware(next Handler, logger LogWriter) *LoggingMiddleware
```

Now `*StdoutLogger` still satisfies `LogWriter` (it has `Log`). And a simple test double is:

```go
type testLogger struct{ entries []string }
func (t *testLogger) Log(level, msg string, _ map[string]any) {
    t.entries = append(t.entries, msg)
}
// No Flush(), no Close() needed.
```

**The principle:** Define interfaces at the point of use, sized to what you actually use. "Accept interfaces, return structs" — but the interface you accept should be as small as possible.

---

### 2. `RegisterHandler` accepts `*Router` instead of a minimal interface

**Location:** `RegisterHandler(r *Router, path string, h Handler, logger Logger)`

`RegisterHandler` calls exactly one method on `r`: `r.Register(path, wrapped)`. But it accepts `*Router` — the concrete type. This creates hard coupling.

**What it means in practice:**

```go
// You can only call RegisterHandler with *Router:
RegisterHandler(router, "/api/users", usersHandler, log)

// You cannot call it with a mock, a test router, or a future RouterV2:
RegisterHandler(mockRouter, "/api/users", usersHandler, log)  // compile error
```

Writing a test for any code that calls `RegisterHandler` now requires constructing a real `*Router`, even if you just want to verify that the handler was registered at the right path.

**Fix:** Define a minimal `Registrar` interface:

```go
// Registrar can register a handler for a path.
// *Router satisfies this — no changes needed to Router itself.
type Registrar interface {
    Register(path string, h Handler)
}

func RegisterHandler(r Registrar, path string, h Handler, logger LogWriter) {
    wrapped := NewLoggingMiddleware(h, logger)
    r.Register(path, wrapped)
}
```

Now tests can pass a `*testRegistrar` that just records what was registered. The real `*Router` works unchanged.

**The principle:** Functions should accept the smallest interface that covers their actual usage. If you only call one method, accept an interface with one method.

---

## Minor Suggestions

### 3. `MetricsCollector` is a single-method interface — consider a function type

**Location:** `MetricsCollector` interface, `NewMetricsMiddleware`

```go
type MetricsCollector interface {
    RecordLatency(route string, d time.Duration)
}
```

One method, no state. Callers have to define a type with `RecordLatency` to use `MetricsMiddleware` — they can't pass an inline function.

**A function type is more composable:**

```go
type RecordLatencyFunc func(route string, d time.Duration)

type MetricsMiddleware struct {
    next   Handler
    record RecordLatencyFunc
}

func NewMetricsMiddleware(next Handler, record RecordLatencyFunc) *MetricsMiddleware {
    return &MetricsMiddleware{next: next, record: record}
}

func (mm *MetricsMiddleware) Handle(req Request) (Response, error) {
    start := time.Now()
    resp, err := mm.next.Handle(req)
    mm.record(req.Path, time.Since(start))
    return resp, err
}
```

Callers can now pass any function:

```go
// With Prometheus:
NewMetricsMiddleware(handler, prometheus.RecordLatency)

// With a closure:
NewMetricsMiddleware(handler, func(route string, d time.Duration) {
    histogram.Observe(route, d.Seconds())
})

// In tests:
var recorded []string
NewMetricsMiddleware(handler, func(route string, _ time.Duration) {
    recorded = append(recorded, route)
})
```

No interface definition, no boilerplate type, no `RecordLatency` method stub.

**When to use function types vs interfaces:**
- Function type: single operation, no state, implementations are often short closures
- Interface: multiple related methods, implementations have meaningful state, multiple users of the same contract

`RecordLatencyFunc` fits the function type criteria perfectly.

---

## Positive Feedback

- `LoggingMiddleware` itself satisfies `Handler` — it's composable with the existing `Handler` chain. You can stack `LoggingMiddleware(MetricsMiddleware(actualHandler))`.
- `Handle` logs both the request *and* the response, with latency. Useful for debugging.
- Error path is handled explicitly — error responses are logged at `"error"` level rather than silently passing through.
- The struct fields are unexported — callers can't bypass the middleware by reaching into `lm.next` directly.
- The `String()` method is a nice touch for debug output.

---

## Summary

| # | Severity | Issue | Fix |
|---|----------|-------|-----|
| 1 | Major | `LoggingMiddleware` accepts full `Logger` (3 methods), only uses `Log()` | Define minimal `LogWriter` interface with just `Log()` |
| 2 | Major | `RegisterHandler` accepts `*Router` (concrete), only calls `Register()` | Define `Registrar` interface with just `Register()` |
| 3 | Minor | `MetricsCollector` is a single-method interface | Use `type RecordLatencyFunc func(route string, d time.Duration)` instead |
