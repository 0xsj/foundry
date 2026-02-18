# Solution: Package Structure Debugging

## Bug 1: Import Cycle (the cascade error)

**Location:** `pkg/a/a.go` (imports `pkg/b`) and `pkg/b/b.go` (imports `pkg/a`)

**Error:**
```
package github.com/foundry/notify/pkg/b: import cycle not allowed
    github.com/foundry/notify/pkg/a imports github.com/foundry/notify/pkg/b
    github.com/foundry/notify/pkg/b imports github.com/foundry/notify/pkg/a
```

**What's happening:** Go's package dependency graph must be a directed acyclic graph (DAG). Package `a` imports `b` to use `b.NewSender`. Package `b` imports `a` to call `a.RouteEvent`. This creates a cycle: `a → b → a`. The compiler refuses to compile cyclic imports.

**Why it happens:** The developer mixed two concerns that need each other:
- `a` (routing) needs to know about `b` (senders) to check capabilities
- `b` (senders) needs to know about `a` (routing) to find routes

This is a design smell — it usually means the code that bridges both concerns belongs in a third package, or one of the dependencies needs to be inverted.

**Three strategies to break cycles:**

**Strategy 1: Extract the shared interface to a third package**
```
pkg/
├── sender/      # defines Sender interface only — no implementation
├── router/      # was "a" — imports sender interface, not implementation
└── smtp/        # was "b" — implements sender interface
```
`router` and `smtp` both import `sender` (the interface). Neither imports the other.

**Strategy 2: Dependency inversion — pass the dependency as a parameter**
```go
// In pkg/a, don't import pkg/b at all.
// Instead, accept a capability-checking function:

// RouteEvent takes a function that reports whether a protocol is available.
func RouteEvent(eventType string, hasProtocol func(string) bool) []ChannelConfig {
    // use hasProtocol("smtp") instead of calling b.NewSender
}
```

**Strategy 3: Merge packages (if the cycle is unavoidable)**
If two packages always need each other, they should probably be one package. Artificial splits create cycles.

**Fix applied in this solution:** Move `RouteForCapability` out of package `b` entirely (it belongs in a coordinator layer, not in the sender package) and remove `b`'s import of `a`.

---

## Bug 2: Accessing a Nonexistent Field

**Location:** `main.go` line 22

**Error:**
```
./main.go:18:13: cfg.retryCount undefined (cannot refer to unexported field or method retryCount)
```

**What's happening:** `cfg` is of type `a.ChannelConfig`, which has fields `Channel` and `Priority`. There is no `retryCount` field on `ChannelConfig`.

The developer likely confused two things:
- `sender.maxRetry` — an unexported field on the unexported `b.sender` struct
- `ChannelConfig.retryCount` — which doesn't exist at all

Even if `retryCount` existed on `ChannelConfig`, it would need to be exported (start with uppercase) to be accessible from outside package `a`.

**Two lessons:**
1. Unexported fields (`lowercase`) are only accessible within the package that defines the struct. External packages cannot read or write them.
2. Don't confuse types — `ChannelConfig` and `sender` are completely different types in different packages.

**Fix:** Either add an exported `MaxRetry int` field to `ChannelConfig` if retry configuration belongs there, or remove the line if it was a mistake.

```go
// Fixed ChannelConfig with an optional retry override:
type ChannelConfig struct {
    Channel  string
    Priority int
    MaxRetry int  // exported — accessible from main.go
}

// In main.go:
fmt.Printf("max retry: %d\n", cfg.MaxRetry)  // now valid
```

---

## Bug 3: Returning Unexported Type from Exported Function

**Location:** `pkg/b/b.go` — `NewSender` returns `*sender`

**Error:**
```
./main.go:26:30: cannot use b.NewSender("smtp") (type *b.sender) as type b.Sender
```

**What's happening:** `sender` (lowercase) is an unexported type. `NewSender` returns `*sender`. External packages (like `main`) cannot name or use unexported types — they literally can't write `*b.sender` in their code, and the compiler won't let them assign a value of that type to an interface variable either.

The `Sender` interface is exported (uppercase). The concrete implementation `sender` is not. This is a common Go pattern — **return the interface, hide the implementation** — but the function signature must return the interface, not the unexported struct.

**Fix: Change `NewSender` to return `Sender`**

```go
// Before (bug): returns unexported concrete type
func NewSender(protocol string) *sender {
    ...
}

// After (fixed): returns exported interface
func NewSender(protocol string) Sender {
    if protocol != "smtp" {
        return nil
    }
    return &sender{protocol: protocol, maxRetry: 3}
}
```

Now in `main.go`:
```go
s := b.NewSender("smtp")  // s is of type b.Sender
if s != nil {
    fmt.Println(s.Protocol())  // works
}
```

**Why this is idiomatic Go:**

The caller doesn't need to know that `NewSender("smtp")` returns an `*smtpSender` internally. They just need the `Sender` interface — `Send` and `Protocol`. Hiding the concrete type:
- Allows you to change the implementation without breaking callers
- Prevents callers from bypassing the interface and calling internal methods
- Is the standard pattern in the Go standard library (e.g., `os.Open` returns `*os.File`, but `io.Reader` is what callers should use when possible)

---

## Summary

| # | Bug | Root Cause | Fix |
|---|-----|------------|-----|
| 1 | Import cycle `a ↔ b` | Circular dependencies between packages | Remove one direction; extract interface to third package or use dependency inversion |
| 2 | `cfg.retryCount` doesn't exist | Confused struct types; field not exported | Add exported `MaxRetry` field or remove the access |
| 3 | `NewSender` returns `*sender` (unexported) | Unexported return type from exported function | Change return type to `Sender` interface |

**Related:**
- [[modules-and-packages#exported-vs-unexported]] — uppercase = exported
- [[modules-and-packages#import-cycles]] — the DAG requirement
- Return interfaces, accept interfaces — a core Go idiom
