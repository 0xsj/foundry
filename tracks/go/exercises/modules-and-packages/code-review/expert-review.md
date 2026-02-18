# Expert Review: Notification Service Package Reorganization

## Critical Issues

### 1. `init()` creates invisible, order-dependent dependencies

**Location:** `email/email.go` (line 14), `sms/sms.go` (line 12)

The driver registration model requires callers to blank-import the driver packages:

```go
import _ "github.com/foundry/notify/email"
import _ "github.com/foundry/notify/sms"
```

If a caller forgets the blank import, `driver.Get("email")` returns `"no driver registered"` at runtime — silently. No compile-time error, no startup panic (unless they call `Get` at init, which is not guaranteed). This bug can make it to production.

The database/sql pattern this borrows from (`import _ "github.com/lib/pq"`) exists because the set of SQL drivers is open-ended and unknown to the library. Here, the notification service has exactly two known drivers. Explicit registration is unambiguously better:

```go
// In main.go or a setup function — explicit, visible, testable
func setupDrivers(registry *driver.Registry) {
    registry.Register("email", email.New("smtp.example.com", 587))
    registry.Register("sms", sms.New(os.Getenv("SMS_API_KEY")))
}
```

Now the registration is visible at the call site, testable (pass a mock registry), and fails at startup if credentials are missing — not at first send.

**Severity: Critical** — silent misconfiguration reachable via a forgotten import.

---

### 2. `driver.Register` panics in `init()` — unrecoverable error with no context

**Location:** `driver/driver.go` line 31

```go
panic(fmt.Sprintf("driver: protocol %q already registered", protocol))
```

Panicking in `init()` is dangerous: the `init()` call stack is controlled by the Go runtime, not by your code. If a package is accidentally imported twice (through separate import paths — rare but possible with module replace directives), the panic fires before `main()` runs, with a stack trace pointing into the runtime scheduler.

Worse: you can't recover from a panic in `init()` using `defer recover()` — there's nowhere sensible to put the deferred call.

**Fix:** Return an error from `Register`:

```go
func (r *Registry) Register(protocol string, s Sender) error {
    r.mu.Lock()
    defer r.mu.Unlock()
    if _, exists := r.drivers[protocol]; exists {
        return fmt.Errorf("driver: protocol %q already registered", protocol)
    }
    r.drivers[protocol] = s
    return nil
}
```

Then the caller (setup function, not `init`) handles the error explicitly.

---

## Major Concerns

### 3. Stuttered package names throughout

**Location:** `config/config.go`, `notifyconfig/notifyconfig.go`

Go convention: the package name is part of the identifier. When callers use your package, they qualify every name with the package:

```go
// The PR's current API:
loader := config.NewConfigLoader("APP")   // "Config" repeated twice
data, err := loader.Load()               // data is *config.ConfigData
nc := notifyconfig.DefaultNotifyConfig() // "Notify" + "Config" + "NotifyConfig"
```

The package name is already providing the namespace. You don't need to repeat it in the type name:

```go
// Fixed:
loader := config.NewLoader("APP")   // config.Loader
data, err := loader.Load()          // data is *config.Data  (or just Config)
nc := notification.Default()        // notification.Config
```

Rule: if the type name starts with the package name, remove the prefix.

**Impact:** Not a correctness bug, but API ergonomics matter. The stutter is especially visible in function calls and will accumulate across a large codebase.

---

### 4. `notifyconfig` is too granular — a package for four fields

**Location:** `notifyconfig/notifyconfig.go`

A package with one exported type, four fields, and a constructor is usually a sign that the type belongs somewhere else. The overhead of a Go package (its own directory, import path, `package` declaration) is only justified when the code it encapsulates is cohesive enough to stand alone.

`NotifyConfig` is configuration for the notification system. It naturally belongs in a `notification` package alongside the types it configures, or in the existing `config` package as a sub-struct.

```go
// Option A: fold into the main config
type Config struct {
    ServiceName  string
    Port         int
    Notification NotificationConfig
}

type NotificationConfig struct {
    MaxRetries   int
    RetryDelayMs int
    BatchSize    int
    DefaultFrom  string
}

// Option B: put it in a notification package with the senders
package notification

type Config struct {
    MaxRetries   int
    RetryDelayMs int
    // ...
}
```

**Impact:** Package proliferation makes the codebase harder to navigate. When everything is a package, nothing is a meaningful boundary.

---

### 5. `EmailSender` is exported but shouldn't be

**Location:** `email/email.go` line 23

```go
type EmailSender struct {  // exported
    host string            // unexported fields
    port int
}
```

`EmailSender` is an implementation detail. Callers interact with it through `driver.Sender`. Exporting the struct:
- Freezes the struct's name and exported methods as public API
- Tempts callers to construct it directly (bypassing the registry and init configuration)
- The unexported fields (`host`, `port`) make direct construction useless anyway — a caller who constructs `email.EmailSender{}` gets an SMTP sender with empty host and port 0

Since callers can't construct it usefully, exporting it provides no benefit but creates API commitment. Should be `emailSender` (unexported). The `driver.Sender` interface is the public API.

```go
type emailSender struct {  // unexported
    host string
    port int
}
// exported function returns the interface — hides the implementation
func New(host string, port int) driver.Sender {
    return &emailSender{host: host, port: port}
}
```

---

## Minor Suggestions

### 6. Validate credentials at construction time, not at `Send` time

**Location:** `sms/sms.go` line 24-27

```go
func (s *smsSender) Send(...) error {
    if s.apiKey == "" {
        return fmt.Errorf("sms: no API key configured")
    }
```

This check fires on the first send attempt, which could be long after startup. Fail fast — validate at construction:

```go
func New(apiKey, gateway string) (driver.Sender, error) {
    if apiKey == "" {
        return nil, fmt.Errorf("sms: API key is required")
    }
    return &smsSender{apiKey: apiKey, gateway: gateway}, nil
}
```

Now misconfiguration is caught at startup, not buried in a log entry from the first production send.

### 7. Global registry is not testable

**Location:** `driver/driver.go`

The global `registry` variable means tests run against the same shared state. If `TestEmailDriver` registers "email" and `TestSMSDriver` registers "sms", any test that calls `driver.Register("email", ...)` a second time panics. Test order matters.

**Fix:** Make the registry a struct that callers create explicitly:

```go
type Registry struct {
    mu      sync.Mutex
    drivers map[string]Sender
}

func New() *Registry {
    return &Registry{drivers: make(map[string]Sender)}
}
```

Tests create their own `Registry` instance — no shared state, no ordering issues.

---

## Positive Feedback

- The `driver.Sender` interface is clean and minimal — two methods, reasonable signatures. Well done.
- The `sms` package correctly uses an unexported type (`smsSender`) — the right default for implementation structs.
- Separating email and SMS into their own packages is the right instinct — it allows them to evolve independently (different retry strategies, different configuration, different test mocks).
- `driver.List()` is a useful debugging primitive — easy to extend for health-check endpoints.
- The `sync.Mutex` in the registry is correct — concurrent `init()` functions from different goroutines would be a race without it.

---

## Summary

| # | Severity | Location | Issue |
|---|----------|----------|-------|
| 1 | Critical | email/, sms/ | `init()` registration is invisible; missing blank imports fail silently at runtime |
| 2 | Critical | driver/ | `panic` in `init()` is unrecoverable and hard to debug |
| 3 | Major | config/, notifyconfig/ | Stuttered names (`config.ConfigLoader`, `notifyconfig.NotifyConfig`) |
| 4 | Major | notifyconfig/ | Package too granular — one type doesn't justify a package |
| 5 | Major | email/ | `EmailSender` exported unnecessarily — leaks implementation |
| 6 | Minor | sms/ | Credential validation at send-time, not construction-time |
| 7 | Minor | driver/ | Global registry makes tests order-dependent |
