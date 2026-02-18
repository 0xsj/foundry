# Error Handling — Go

## Why Errors Are Values

Go's error handling is one of the most debated design decisions in the language, and it's worth understanding *why* it exists before learning *how* it works.

In most languages — JavaScript, Python, Java, C# — errors are exceptions. They flow through an invisible channel separate from the return value. A function either returns normally or throws. The caller either handles the exception or lets it propagate. Unhandled exceptions can skip multiple stack frames silently.

Go made a different choice: **errors are values**. A function that can fail returns an `error` as an explicit return value. The caller must decide what to do with it at the call site. There's no invisible channel.

```go
// Go: error is a return value — the caller must confront it
result, err := os.ReadFile("config.json")
if err != nil {
    // handle it, return it, or wrap it — but you can't ignore it accidentally
    return nil, fmt.Errorf("reading config: %w", err)
}
```

```typescript
// TypeScript: error is thrown — caller must remember to catch
try {
    const result = await fs.readFile("config.json");
} catch (err) {
    // easy to forget the try/catch entirely
}
```

The tradeoff is explicit: Go code has more `if err != nil` lines. In exchange, you always know which operations can fail, and there's no surprise propagation.

> **Key insight:** In Go, ignoring an error requires deliberate action — assigning it to `_`. In JS/TS, handling an error requires deliberate action — writing a `try/catch`. The default behavior is opposite.

### Your notes
<!-- -->

---

## The error Interface

`error` is a built-in interface with a single method:

```go
type error interface {
    Error() string
}
```

That's the entire interface. Any type that implements `Error() string` satisfies it. This is why errors are so composable — you can attach any data to an error type, as long as you also provide an `Error()` string representation.

The zero value of `error` is `nil`, which represents "no error". The convention:
- Return `nil` on success
- Return a non-nil error on failure

```go
func divide(a, b float64) (float64, error) {
    if b == 0 {
        return 0, errors.New("division by zero")
    }
    return a / b, nil
}
```

### Compared to Rust

Rust's equivalent is `Result<T, E>`:

```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Err("division by zero".to_string());
    }
    Ok(a / b)
}
```

Key differences:
- **Go:** `error` is an interface — the error type is open-ended, any type qualifies
- **Rust:** `Result<T, E>` is a generic enum — the error type `E` is fixed at compile time
- **Go:** caller uses `if err != nil` to check
- **Rust:** caller uses `match`, `?` operator, or `unwrap()`

Rust's approach gives stronger compile-time guarantees (you can't mix error types without explicit conversion). Go's approach is more flexible and dynamically typed. Neither is universally better — they reflect each language's philosophy.

### Your notes
<!-- -->

---

## Creating Errors: errors.New and fmt.Errorf

The two standard ways to create errors:

### errors.New

```go
import "errors"

var ErrNotFound = errors.New("not found")

func getUser(id int) (*User, error) {
    u := db.lookup(id)
    if u == nil {
        return nil, errors.New("user not found")
    }
    return u, nil
}
```

`errors.New` creates a simple error with a fixed message. Each call to `errors.New` creates a *new, distinct error value* — two errors created with the same string are not equal:

```go
a := errors.New("oops")
b := errors.New("oops")
fmt.Println(a == b)  // false — different allocations
```

This matters for error comparison (covered in the sentinel errors section).

### fmt.Errorf

```go
import "fmt"

func loadConfig(path string) (*Config, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        return nil, fmt.Errorf("loadConfig %q: %w", path, err)
    }
    // ...
}
```

`fmt.Errorf` creates a formatted error message. The `%w` verb (Go 1.13+) **wraps** the original error — more on this in the wrapping section.

### Convention: Error Message Style

Error messages in Go follow specific conventions:
- **Lowercase, no period at the end:** `"connection refused"`, not `"Connection refused."`
- **No "error:" prefix:** The caller adds context
- **Describe what failed, not that it failed:** `"user not found"` not `"error finding user"`

```go
// Wrong
return fmt.Errorf("Error: Failed to read file %s.", path)

// Right
return fmt.Errorf("read %s: %w", path, err)
```

The reason: errors often get wrapped multiple times. At the top level, you might see:

```
process request: load config: read /etc/app/config.json: no such file or directory
```

Each layer added `"[what I was doing]: [underlying error]"`. If each layer started with "Error:" and ended with ".", it becomes:

```
Error: Failed to process request. Error: Failed to load config. Error: Failed to read file...
```

### Your notes
<!-- -->

---

## Sentinel Errors

A **sentinel error** is a package-level error variable that represents a specific, well-known condition. Callers test for it using `errors.Is`.

```go
// From the standard library:
var EOF = errors.New("EOF")  // io.EOF — end of stream, not an actual error

// sql package:
var ErrNoRows = errors.New("sql: no rows in result set")

// Your package:
var ErrNotFound = errors.New("not found")
var ErrUnauthorized = errors.New("unauthorized")
```

**How callers use them:**

```go
row, err := db.QueryRow("SELECT * FROM users WHERE id = ?", id)
if errors.Is(err, sql.ErrNoRows) {
    // no user found — this is expected, handle gracefully
    return nil, ErrNotFound
}
if err != nil {
    // some other db error — unexpected
    return nil, fmt.Errorf("query user %d: %w", id, err)
}
```

### Why errors.Is, Not ==

You might wonder: why `errors.Is(err, sql.ErrNoRows)` instead of `err == sql.ErrNoRows`?

Two reasons:

**1. Error wrapping.** If the error was wrapped:

```go
wrappedErr := fmt.Errorf("getUserByEmail: %w", sql.ErrNoRows)

wrappedErr == sql.ErrNoRows   // false — different value
errors.Is(wrappedErr, sql.ErrNoRows)  // true — unwraps the chain
```

`errors.Is` unwraps the error chain until it finds a match. Direct equality only checks the outermost error.

**2. Custom error types.** If a custom error type implements the `Is(target error) bool` method, `errors.Is` calls it. This lets a type declare equality semantics beyond pointer identity.

```go
// errors.Is unwraps using the Unwrap() method
type AppError struct {
    Code    int
    Message string
    Cause   error  // the wrapped error
}

func (e *AppError) Error() string { return e.Message }
func (e *AppError) Unwrap() error { return e.Cause }
```

### Sentinel Errors vs Custom Types

When to use each:

| Use sentinel | Use custom type |
|---|---|
| Specific condition with no additional data | Need to attach data (status code, field name, etc.) |
| Simple "did this specific thing happen?" check | Caller needs to extract details from the error |
| Stable API (changing the message would break callers) | Caller uses `errors.As` to inspect the type |

### Your notes
<!-- -->

---

## Error Wrapping: %w, errors.Is, errors.As

Error wrapping is how Go builds **error chains** — each layer adds context while preserving the original cause.

### The %w Verb

```go
func readConfig(path string) (*Config, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        return nil, fmt.Errorf("readConfig: %w", err)  // wraps err
    }
    // ...
}

func loadApp() error {
    cfg, err := readConfig("/etc/app/config.json")
    if err != nil {
        return fmt.Errorf("loadApp: %w", err)  // wraps the already-wrapped error
    }
    // ...
    return nil
}
```

When `loadApp` fails, the error chain looks like:

```
loadApp: readConfig: open /etc/app/config.json: no such file or directory
```

Each `%w` adds a layer. The `%v` verb also formats errors but **does not wrap** — it creates a new error string that severs the chain:

```go
return fmt.Errorf("readConfig: %v", err)  // %v — no wrapping, chain severed
return fmt.Errorf("readConfig: %w", err)  // %w — wraps, chain preserved
```

**Rule:** Use `%w` when you want callers to be able to test for or extract the original error. Use `%v` when you're creating a completely new error context and the original type doesn't matter.

### errors.Is — Testing the Chain

`errors.Is(err, target)` checks whether `err` (or any error in its chain) matches `target`.

```go
var ErrPermission = errors.New("permission denied")

err := fmt.Errorf("saveFile: %w", fmt.Errorf("checkACL: %w", ErrPermission))

errors.Is(err, ErrPermission)  // true — found in chain
```

How it works:
1. Compare `err` to `target` directly (using `==` or `err.Is(target)` if the method exists)
2. If no match, call `err.Unwrap()` to get the next error in the chain
3. Repeat until match found or chain exhausted

### errors.As — Extracting the Type

`errors.As(err, &target)` finds the first error in the chain that matches the type of `target` and sets `target` to that value.

```go
type ValidationError struct {
    Field   string
    Message string
}

func (e *ValidationError) Error() string {
    return fmt.Sprintf("validation error on %s: %s", e.Field, e.Message)
}

// ---

err := fmt.Errorf("processRequest: %w", &ValidationError{
    Field:   "email",
    Message: "invalid format",
})

var valErr *ValidationError
if errors.As(err, &valErr) {
    fmt.Println(valErr.Field)    // email
    fmt.Println(valErr.Message)  // invalid format
}
```

`errors.As` is the runtime-safe alternative to type assertions. Instead of `err.(*ValidationError)` (which panics if wrong), `errors.As` searches the chain and returns `false` if the type isn't found.

### Compared to JS/TS

```typescript
// JS: instanceof check on error chain (manual, no standard)
try {
    await processRequest();
} catch (err) {
    if (err instanceof ValidationError) {
        console.log(err.field);  // type assertion built into instanceof
    } else if (err instanceof NetworkError) {
        // ...
    }
}
```

Go's `errors.As` does the same thing but searches through wrapped errors automatically. In JS, if you rethrow `new Error("context: " + err.message)`, you lose the original type entirely. With `fmt.Errorf("context: %w", err)`, the chain is preserved.

### Your notes
<!-- -->

---

## Custom Error Types

When you need to attach data to an error — status codes, field names, request IDs — create a custom error type.

```go
// A custom error type carries structured data
type NotFoundError struct {
    Resource string
    ID       string
}

func (e *NotFoundError) Error() string {
    return fmt.Sprintf("%s %q not found", e.Resource, e.ID)
}
```

```go
func getUser(id string) (*User, error) {
    u := db.find(id)
    if u == nil {
        return nil, &NotFoundError{Resource: "user", ID: id}
    }
    return u, nil
}

// Caller:
user, err := getUser("u-123")
var notFound *NotFoundError
if errors.As(err, &notFound) {
    // Can access notFound.Resource and notFound.ID
    log.Printf("no %s with id %s", notFound.Resource, notFound.ID)
    return http.StatusNotFound, nil
}
```

### Pointer vs Value Receivers on Error Types

Always use **pointer receivers** for error types:

```go
// Wrong: value receiver
func (e NotFoundError) Error() string { ... }
// The interface is satisfied by NotFoundError and *NotFoundError
// But errors.As(err, &target) where target is *NotFoundError won't find value types

// Right: pointer receiver
func (e *NotFoundError) Error() string { ... }
// errors.As(err, &target) where target is *NotFoundError works correctly
```

Convention: implement the `error` interface on pointer receivers. Always return `*YourErrorType`, not `YourErrorType`.

### Implementing Unwrap for Custom Types

If your custom error type wraps another error, implement `Unwrap()`:

```go
type ProcessingError struct {
    Step  string
    Cause error
}

func (e *ProcessingError) Error() string {
    return fmt.Sprintf("processing failed at %s: %v", e.Step, e.Cause)
}

// Implement Unwrap so errors.Is/As can traverse the chain
func (e *ProcessingError) Unwrap() error {
    return e.Cause
}
```

Without `Unwrap()`, `errors.Is(err, someTargetDeepInChain)` won't work — the chain traversal stops at your type.

### Multiple Error Types in a Package

In a real package, you'll typically have a small family of error types:

```go
// config package errors
var (
    ErrFileNotFound  = errors.New("config file not found")    // sentinel
    ErrInvalidSyntax = errors.New("config syntax invalid")    // sentinel
)

// For errors that need data:
type FieldError struct {
    Field   string
    Value   any
    Message string
}

func (e *FieldError) Error() string {
    return fmt.Sprintf("field %q: %s (got %v)", e.Field, e.Message, e.Value)
}

type ParseError struct {
    Line    int
    Column  int
    Message string
}

func (e *ParseError) Error() string {
    return fmt.Sprintf("parse error at %d:%d: %s", e.Line, e.Column, e.Message)
}
```

Keep the error types lean. If a type has more than 3-4 fields, question whether it's doing too much.

### Your notes
<!-- -->

---

## Error Handling Patterns

### Early Return (Guard Clauses)

The idiomatic Go pattern: check for error immediately, return early, keep the happy path unindented.

```go
// Wrong: nested ifs (the "pyramid of doom")
func processOrder(id string) (*Receipt, error) {
    order, err := getOrder(id)
    if err == nil {
        user, err := getUser(order.UserID)
        if err == nil {
            payment, err := chargeCard(user.CardID, order.Total)
            if err == nil {
                return &Receipt{OrderID: id, PaymentID: payment.ID}, nil
            } else {
                return nil, err
            }
        } else {
            return nil, err
        }
    }
    return nil, err
}

// Right: early return guard clauses
func processOrder(id string) (*Receipt, error) {
    order, err := getOrder(id)
    if err != nil {
        return nil, fmt.Errorf("processOrder: get order: %w", err)
    }

    user, err := getUser(order.UserID)
    if err != nil {
        return nil, fmt.Errorf("processOrder: get user: %w", err)
    }

    payment, err := chargeCard(user.CardID, order.Total)
    if err != nil {
        return nil, fmt.Errorf("processOrder: charge card: %w", err)
    }

    return &Receipt{OrderID: id, PaymentID: payment.ID}, nil
}
```

The right version reads top-to-bottom. Each `if err != nil` is a guard: if something failed, stop. The happy path proceeds without indentation.

### Wrapping with Context

Every time you propagate an error upward, add context about what *your function was doing*. The final error message is a breadcrumb trail from the top-level operation down to the root cause.

```go
// Poor: loses context
func loadUserProfile(id string) (*Profile, error) {
    data, err := db.Query("SELECT * FROM profiles WHERE user_id = ?", id)
    if err != nil {
        return nil, err  // caller has no idea what failed
    }
    // ...
}

// Good: adds context
func loadUserProfile(id string) (*Profile, error) {
    data, err := db.Query("SELECT * FROM profiles WHERE user_id = ?", id)
    if err != nil {
        return nil, fmt.Errorf("loadUserProfile %s: %w", id, err)  // breadcrumb
    }
    // ...
}
```

### Collecting Multiple Errors

Sometimes you want to collect all errors rather than stopping at the first. A common pattern is a multi-error type:

```go
// errors.Join (Go 1.20+) — combine multiple errors into one
func validateForm(form Form) error {
    var errs []error

    if form.Name == "" {
        errs = append(errs, errors.New("name is required"))
    }
    if form.Email == "" {
        errs = append(errs, errors.New("email is required"))
    }
    if !strings.Contains(form.Email, "@") {
        errs = append(errs, errors.New("email must contain @"))
    }

    return errors.Join(errs...)  // returns nil if errs is empty
}
```

`errors.Join` returns nil if all errors are nil. If any are non-nil, it returns a combined error. The combined error's `Unwrap() []error` method returns all component errors, so `errors.Is` and `errors.As` work on any of them.

```go
err := validateForm(form)
if err != nil {
    // The full message shows all errors
    fmt.Println(err)  // "name is required\nemail is required"
}
```

### Your notes
<!-- -->

---

## panic and recover

`panic` and `recover` are Go's mechanism for truly exceptional situations — not normal errors.

### When to Use panic

**Never use panic for expected errors.** Use it only for:
- Programming errors that should never happen in correct code (nil pointer where none expected, index out of bounds you've verified should be in range)
- Initialization failures that make the program impossible to run correctly
- Internal contract violations within a package (often only in unexported code)

```go
// Appropriate: a programming error that indicates a bug
func mustCompile(pattern string) *regexp.Regexp {
    re, err := regexp.Compile(pattern)
    if err != nil {
        // If a hardcoded pattern fails to compile, the code is wrong — panic is correct
        panic(fmt.Sprintf("invalid regex pattern %q: %v", pattern, err))
    }
    return re
}

// Package-level initialization — only works with hardcoded patterns
var emailPattern = mustCompile(`^[^@]+@[^@]+\.[^@]+$`)
```

The `mustXxx` convention signals: "this panics on failure, only call it if you're certain it will succeed." You'll see it in the standard library: `template.Must`, `regexp.MustCompile`, etc.

**Never panic in library code for user input errors.** Library users cannot recover from panics without disrupting the whole goroutine. Return an error instead:

```go
// Library: WRONG — panics on bad input
func ParseConfig(data []byte) *Config {
    var cfg Config
    if err := json.Unmarshal(data, &cfg); err != nil {
        panic(err)  // NO — library should not panic on user error
    }
    return &cfg
}

// Library: RIGHT — return error
func ParseConfig(data []byte) (*Config, error) {
    var cfg Config
    if err := json.Unmarshal(data, &cfg); err != nil {
        return nil, fmt.Errorf("ParseConfig: %w", err)
    }
    return &cfg, nil
}
```

### recover

`recover` stops a propagating panic and returns the panic value. It only works when called **directly from a deferred function**:

```go
func safeExecute(fn func()) (err error) {
    defer func() {
        if r := recover(); r != nil {
            err = fmt.Errorf("recovered from panic: %v", r)
        }
    }()
    fn()
    return nil
}
```

The most common use: HTTP servers catch panics from handlers so one bad request doesn't kill the whole server. This is what `net/http`'s `http.Server` does internally.

```go
// HTTP handler wrapper that catches panics
func RecoverMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        defer func() {
            if rec := recover(); rec != nil {
                log.Printf("panic in handler: %v\n%s", rec, debug.Stack())
                http.Error(w, "internal server error", http.StatusInternalServerError)
            }
        }()
        next.ServeHTTP(w, r)
    })
}
```

### panic vs error: Decision Table

| Scenario | Use |
|---|---|
| File not found | `error` — expected condition |
| Network timeout | `error` — expected condition |
| Invalid user input | `error` — expected condition |
| Hardcoded regex fails to compile | `panic` — programming error |
| Index out of bounds you calculated | `panic` or investigate the calculation |
| Required environment variable missing at startup | Either — depends on design |
| Internal invariant violated (shouldn't happen) | `panic` |
| Any public API receiving user data | Always `error` |

### Your notes
<!-- -->

---

## Compared to TypeScript: try/catch vs Explicit Returns

The fundamental difference in philosophy:

```typescript
// TypeScript: error handling is opt-in — you can skip it
async function processOrder(id: string): Promise<Receipt> {
    const order = await getOrder(id);     // throws on error — no indication in type
    const user = await getUser(order.userId);
    const payment = await chargeCard(user.cardId, order.total);
    return { orderId: id, paymentId: payment.id };
}

// Caller must know to catch:
try {
    const receipt = await processOrder("o-123");
} catch (err) {
    // What type is err? Error? NetworkError? ValidationError? Unknown.
    console.error(err);
}
```

```go
// Go: error handling is opt-out — you have to explicitly ignore it
func processOrder(id string) (*Receipt, error) {
    order, err := getOrder(id)         // error is explicit in the return type
    if err != nil {
        return nil, fmt.Errorf("processOrder: %w", err)
    }
    user, err := getUser(order.UserID)
    if err != nil {
        return nil, fmt.Errorf("processOrder: %w", err)
    }
    payment, err := chargeCard(user.CardID, order.Total)
    if err != nil {
        return nil, fmt.Errorf("processOrder: %w", err)
    }
    return &Receipt{OrderID: id, PaymentID: payment.ID}, nil
}
```

TypeScript's approach: cleaner happy path, but the error type is `unknown` — you lose type safety at the boundary. You can use discriminated unions or `Result<T,E>` types from libraries (like `neverthrow`) to get Go-like behavior in TypeScript, but it's not the default.

Go's approach: more verbose, but errors are documented in the function signature. The type system tells you "this function can fail and here's the error type."

### Your notes
<!-- -->

---

## How It All Fits Together

Here's a realistic error handling pattern you'll use constantly in production Go:

```go
// Multiple custom error types for a service
var (
    ErrNotFound   = errors.New("not found")
    ErrForbidden  = errors.New("forbidden")
    ErrBadRequest = errors.New("bad request")
)

type ValidationError struct {
    Field   string
    Message string
}

func (e *ValidationError) Error() string {
    return fmt.Sprintf("invalid %s: %s", e.Field, e.Message)
}

// Service function: returns typed errors
func updateUser(userID, requesterID string, update UserUpdate) error {
    if update.Name == "" {
        return &ValidationError{Field: "name", Message: "cannot be empty"}
    }
    if update.Email != "" && !isValidEmail(update.Email) {
        return &ValidationError{Field: "email", Message: "invalid format"}
    }

    existing, err := db.GetUser(userID)
    if err != nil {
        if errors.Is(err, ErrNotFound) {
            return ErrNotFound  // re-use sentinel
        }
        return fmt.Errorf("updateUser: get user: %w", err)
    }

    if existing.OwnerID != requesterID {
        return ErrForbidden
    }

    if err := db.UpdateUser(userID, update); err != nil {
        return fmt.Errorf("updateUser: save: %w", err)
    }

    return nil
}

// HTTP handler: maps errors to HTTP status codes
func handleUpdateUser(w http.ResponseWriter, r *http.Request) {
    // ... parse request ...
    err := updateUser(userID, requesterID, update)
    if err == nil {
        w.WriteHeader(http.StatusOK)
        return
    }

    var valErr *ValidationError
    switch {
    case errors.As(err, &valErr):
        http.Error(w, valErr.Error(), http.StatusBadRequest)
    case errors.Is(err, ErrNotFound):
        http.Error(w, "user not found", http.StatusNotFound)
    case errors.Is(err, ErrForbidden):
        http.Error(w, "forbidden", http.StatusForbidden)
    default:
        log.Printf("unexpected error: %v", err)
        http.Error(w, "internal server error", http.StatusInternalServerError)
    }
}
```

This pattern — custom error types for typed errors, sentinel errors for well-known conditions, wrapping with context at each layer, `errors.Is`/`errors.As` at the top for dispatch — is what you'll see in almost every production Go codebase.

### Your notes
<!-- -->
