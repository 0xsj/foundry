# Structs, Methods & Enums — Go

## Structs: Composing Data

### What a Struct Is

A struct is a composite type that groups related fields together under one name. In memory, struct fields are laid out contiguously — no vtable, no hidden pointers to an object header, no prototype chain. Just the fields, one after another.

```go
type Notification struct {
    ID        string
    Recipient string
    Subject   string
    Body      string
    Retries   int
    Sent      bool
}
```

When you declare a `Notification`, the compiler allocates enough contiguous bytes for all fields: two string headers (16 bytes each), one int (8 bytes), one bool (1 byte, padded to alignment boundary). The total depends on alignment padding — use `unsafe.Sizeof` to see the actual size.

### Struct Literals

Two ways to construct a struct:

```go
// Named fields — preferred. Order doesn't matter. Unspecified fields get zero values.
n := Notification{
    ID:        "notif-123",
    Recipient: "alice@example.com",
    Subject:   "Your order shipped",
}
// n.Body == "", n.Retries == 0, n.Sent == false

// Positional — avoid in production code.
// Order must match declaration exactly. Fragile — adding a field breaks all call sites.
n2 := Notification{"notif-124", "bob@example.com", "Payment received", "...", 0, false}
```

Named fields are mandatory in any code that could change. Positional literals are sometimes used in tests for small, stable structs, but that's the only defensible case.

### Zero Values for Structs

When you declare a struct without initializing it, all fields get their zero values:

```go
var n Notification
// n.ID == ""
// n.Recipient == ""
// n.Retries == 0
// n.Sent == false
```

This is one of Go's most useful properties. A `Notification` at zero value is ready to use — you can set fields, pass it to functions, read from it — without any explicit initialization call. Compare this to C, where uninitialized fields contain garbage, or Java, where you might get `null` field panics.

Design your structs so the zero value is meaningful. A zero-value `Notification` with empty strings and `Retries == 0` makes sense. But a zero-value database connection struct is probably not meaningful — which is a signal that it needs a constructor.

### Nested Structs

Structs compose naturally:

```go
type Address struct {
    Street string
    City   string
    State  string
    Zip    string
}

type User struct {
    ID      int64
    Email   string
    Address Address  // nested by value
}

u := User{
    ID:    1,
    Email: "alice@example.com",
    Address: Address{
        Street: "123 Main St",
        City:   "Portland",
        State:  "OR",
        Zip:    "97201",
    },
}

fmt.Println(u.Address.City) // Portland
```

The `Address` struct is embedded **by value** — it's copied with `User`. To avoid copying large nested structs, embed a pointer: `Address *Address`. The pointer overhead is 8 bytes regardless of how big `Address` is.

### Anonymous Structs

You can define a struct type inline without naming it. Useful in two places: tests and one-off JSON shapes.

```go
// In tests — fast to write, no exported type needed
testCases := []struct {
    input    string
    expected int
}{
    {"hello", 5},
    {"café", 4},
    {"", 0},
}

// For ad-hoc JSON
resp := struct {
    Status  int    `json:"status"`
    Message string `json:"message"`
}{
    Status:  200,
    Message: "ok",
}
```

Anonymous structs are value types like named structs. Two anonymous struct types are identical if they have the same fields in the same order with the same types and tags.

### Field Tags

Tags are string literals attached to struct fields, readable at runtime via reflection. They don't affect the type system — the struct compiles the same with or without tags. But the standard library and many packages use them heavily.

```go
type WebhookEvent struct {
    EventType string    `json:"event_type" validate:"required"`
    Payload   []byte    `json:"payload"`
    Timestamp int64     `json:"timestamp,omitempty"`
    internal  string    // unexported — ignored by encoding/json
}
```

The tag is a raw string literal. The convention is `key:"value"` pairs separated by spaces. Multiple keys are supported. The `encoding/json` package reads the `json` key to determine serialization names. `omitempty` skips the field when it's a zero value. `validate` tags are read by validation libraries like `go-playground/validator`.

Tags are a runtime mechanism — they're visible through `reflect.StructField.Tag`. They don't provide compile-time guarantees. If you typo `json:"event_tpye"`, the compiler won't tell you.

### Your notes

---

## Methods: Behavior Attached to Types

### Defining Methods

In Go, methods are functions with a receiver — an extra parameter before the function name that binds the function to a type. Any named type in the same package can have methods.

```go
type Notification struct {
    ID        string
    Recipient string
    Subject   string
    Retries   int
    Sent      bool
}

// Value receiver — n is a copy
func (n Notification) String() string {
    return fmt.Sprintf("[%s] to=%s subject=%q retries=%d",
        n.ID, n.Recipient, n.Subject, n.Retries)
}

// Pointer receiver — n is a reference to the original
func (n *Notification) MarkSent() {
    n.Sent = true
}

func (n *Notification) IncrementRetry() {
    n.Retries++
}
```

The receiver appears between `func` and the method name. The name is typically a one or two letter abbreviation of the type (Go convention) — not `self` or `this`.

### Value Receivers vs Pointer Receivers

This is the central question when defining any method. The rule is direct:

| Use | When |
|-----|------|
| Pointer receiver `*T` | Method needs to mutate the struct |
| Pointer receiver `*T` | Struct is large (copying is expensive) |
| Value receiver `T` | Method only reads, struct is small |
| Value receiver `T` | Method should work on a copy (intentional isolation) |

**One more rule:** if any method on a type uses a pointer receiver, all methods should use pointer receivers. Mixing them creates subtle bugs with interfaces (more on method sets below).

```go
// Value receiver: doesn't need to modify, returns computed info
func (n Notification) IsRetryable() bool {
    return n.Retries < 3 && !n.Sent
}

// Pointer receiver: modifies state
func (n *Notification) MarkSent() {
    n.Sent = true
    n.Retries = 0
}
```

### Method Sets and Addressability

This is the subtle part. Go has a rule called **method sets** that determines which methods are accessible depending on how you hold a value.

- A value of type `T` has only the methods with value receivers (`T`)
- A value of type `*T` (pointer to T) has methods from both `T` and `*T`

In practice, Go is smart about automatic address-taking. If you call a pointer-receiver method on an addressable value, Go automatically takes its address:

```go
n := Notification{ID: "123"}
n.MarkSent()   // Go automatically does (&n).MarkSent() — fine
```

But this doesn't work for non-addressable values — like values returned directly from a function or map lookups:

```go
// This fails:
notifications["alert"].MarkSent()
// ^ cannot take the address of a map element — map elements are not addressable
```

This is why you often see maps of pointers: `map[string]*Notification`.

Method sets matter most when you assign values to interface variables. A `*Notification` satisfies an interface that requires pointer-receiver methods. A `Notification` value does not — even if you could implicitly dereference.

### Constructor Functions

Go has no constructors in the class sense. The convention is a function named `New<Type>` that returns an initialized instance.

```go
func NewNotification(id, recipient, subject, body string) (*Notification, error) {
    if id == "" {
        return nil, fmt.Errorf("notification id is required")
    }
    if recipient == "" {
        return nil, fmt.Errorf("recipient is required")
    }
    return &Notification{
        ID:        id,
        Recipient: recipient,
        Subject:   subject,
        Body:      body,
    }, nil
}
```

Why return a pointer? Usually because:
1. The struct will be mutated via methods (pointer receivers)
2. You want all callers to share the same instance
3. The struct is large enough that copying is wasteful

When should you return by value? When the type is meant to be used as a value — small, immutable, or when you want each caller to get their own copy.

The constructor pattern also gives you a validation boundary. If a `Notification` can only be created through `NewNotification`, you can enforce invariants there. Users can't accidentally create a `Notification{}` with an empty ID.

### Your notes

---

## Embedding: Composition Without Inheritance

### What Embedding Is

Go doesn't have inheritance. Instead, it has **embedding** — you can embed a type inside a struct, and the embedded type's methods and fields are **promoted** to the outer struct.

```go
type RetryConfig struct {
    MaxAttempts int
    BackoffMs   int
}

func (r *RetryConfig) ShouldRetry(attempts int) bool {
    return attempts < r.MaxAttempts
}

func (r *RetryConfig) Backoff(attempts int) time.Duration {
    delay := r.BackoffMs * (1 << attempts) // exponential backoff
    return time.Duration(delay) * time.Millisecond
}

type EmailChannel struct {
    RetryConfig           // embedded — no field name, just type
    SMTPHost   string
    SMTPPort   int
    FromAddr   string
}

type WebhookChannel struct {
    RetryConfig           // same embedded type
    Endpoint   string
    Secret     string
    TimeoutMs  int
}
```

Now `EmailChannel` has `ShouldRetry` and `Backoff` methods without defining them:

```go
email := &EmailChannel{
    RetryConfig: RetryConfig{MaxAttempts: 3, BackoffMs: 100},
    SMTPHost:    "smtp.example.com",
    SMTPPort:    587,
    FromAddr:    "noreply@example.com",
}

email.ShouldRetry(1)           // calls RetryConfig.ShouldRetry — promoted
email.RetryConfig.ShouldRetry(1)  // also valid — explicit qualification
email.MaxAttempts              // promoted field access
```

The embedded type's methods and fields become part of the outer type's method set. This is not subtyping — `EmailChannel` is not a `RetryConfig`. You can't pass an `EmailChannel` where a `RetryConfig` is expected.

### Embedding Pointers

You can embed a pointer to a type instead of the type directly. The difference: the zero value of a pointer is `nil`, and calling methods on an uninitialized embedded pointer panics.

```go
type EmailChannel struct {
    *RetryConfig  // embedded pointer — must be initialized before use
    SMTPHost string
}

email := &EmailChannel{SMTPHost: "smtp.example.com"}
email.ShouldRetry(1)  // PANIC: nil pointer dereference — RetryConfig not initialized
```

Embed by value when the embedded struct should always exist. Embed by pointer when it's optional or lazily initialized.

### Method Shadowing (Override)

If the outer struct defines a method with the same name as the embedded struct, the outer method wins:

```go
type AuditLogger struct {
    prefix string
}

func (a *AuditLogger) Log(msg string) {
    fmt.Printf("[AUDIT][%s] %s\n", a.prefix, msg)
}

type AlertChannel struct {
    AuditLogger           // embedded
    RetryConfig
    Endpoint string
}

// AlertChannel defines its own Log — shadows AuditLogger.Log
func (a *AlertChannel) Log(msg string) {
    fmt.Printf("[ALERT][%s] %s\n", a.Endpoint, msg)
    a.AuditLogger.Log(msg)  // can still call the embedded method explicitly
}
```

When `a.Log("sent")` is called on an `AlertChannel`, Go calls `AlertChannel.Log`. If `AlertChannel` didn't define `Log`, Go would call `AuditLogger.Log` (promoted). Explicit qualification `a.AuditLogger.Log(msg)` always reaches the embedded method regardless of shadowing.

This is how Go approximates method overriding — but it's fundamentally different from OOP override. The embedded type doesn't know anything about the outer type. There's no virtual dispatch. `AuditLogger.Log` will never call `AlertChannel.Log` — the "override" only applies when called on an `AlertChannel` directly.

### Composition Over Inheritance: The Go Philosophy

In class-based OOP, you build hierarchies: `Animal → Mammal → Dog`. Methods at the top of the hierarchy are inherited down.

Go's take: hierarchies are rigid. Real systems need mix-and-match behavior. Embedding lets you compose arbitrary behaviors:

```go
// These share retry logic but nothing else
type EmailChannel struct {
    RetryConfig
    // email-specific fields
}

type WebhookChannel struct {
    RetryConfig
    RateLimiter  // different additional behavior
    // webhook-specific fields
}

// These share audit logging but nothing else
type AlertChannel struct {
    AuditLogger
    RetryConfig
    // alert-specific fields
}
```

No hierarchy. Each channel type picks the behaviors it needs. Adding a new behavior doesn't require touching a base class. This scales much better for systems that grow organically.

**Coming from TypeScript/JS:** This replaces both class inheritance and mixin patterns. TypeScript does this with `implements` and manual delegation, or with mixin functions. Go bakes delegation into the language as embedding.

### Your notes

---

## Enums: Typed Constants and iota

### Go Has No Enum Keyword

Go doesn't have a native `enum` type. Instead, the convention is a combination of:
1. A named integer type for type safety
2. `iota` to auto-increment constant values
3. A `String()` method for readable output

```go
type NotificationStatus int

const (
    StatusPending  NotificationStatus = iota // 0
    StatusSending                            // 1
    StatusDelivered                          // 2
    StatusFailed                             // 3
    StatusRetrying                           // 4
)
```

`NotificationStatus` is a distinct type from `int` — you can't accidentally pass a raw `int` where a `NotificationStatus` is expected, and you can't compare them without an explicit conversion. This is the type safety you want.

### iota: How It Works

`iota` is a predeclared identifier that represents the index of the constant in its `const` block, starting at 0 and incrementing for each `ConstSpec`. It resets to 0 at each new `const` keyword.

```go
const (
    _          = iota // skip 0 — useful when 0 means "unset"
    StatusPending     // 1
    StatusSending     // 2
    StatusDelivered   // 3
    StatusFailed      // 4
)

// Bit flags — iota in expressions
type Permission uint

const (
    PermRead    Permission = 1 << iota // 1 (1 << 0)
    PermWrite                          // 2 (1 << 1)
    PermDelete                         // 4 (1 << 2)
    PermAdmin                          // 8 (1 << 3)
)

p := PermRead | PermWrite  // combine with bitwise OR
if p&PermRead != 0 {       // check with bitwise AND
    fmt.Println("can read")
}
```

### Adding a String() Method

Without `String()`, printing a `NotificationStatus` shows a number:

```go
fmt.Println(StatusFailed) // "3" — not helpful
```

Add a `String() string` method and `fmt.Println` automatically calls it (via the `fmt.Stringer` interface):

```go
func (s NotificationStatus) String() string {
    switch s {
    case StatusPending:
        return "pending"
    case StatusSending:
        return "sending"
    case StatusDelivered:
        return "delivered"
    case StatusFailed:
        return "failed"
    case StatusRetrying:
        return "retrying"
    default:
        return fmt.Sprintf("NotificationStatus(%d)", int(s))
    }
}

fmt.Println(StatusFailed) // "failed"
```

The `default` case handles values that don't match any constant — important for forward compatibility (someone adds a constant later) and for values deserialized from external sources.

For large enums, code generation is common. `go:generate` with tools like `stringer` produces this boilerplate automatically. But for small enums, the switch is fine.

### Type Switches: Pattern Matching for Types

When you have an interface and need to behave differently based on the concrete type, use a type switch:

```go
type Channel interface {
    Send(n *Notification) error
    Name() string
}

type EmailChannel struct { /* ... */ }
type SMSChannel   struct { /* ... */ }
type SlackChannel struct { /* ... */ }

func logChannelInfo(ch Channel) {
    switch c := ch.(type) {
    case *EmailChannel:
        fmt.Printf("Email channel via %s\n", c.SMTPHost)
    case *SMSChannel:
        fmt.Printf("SMS channel via %s\n", c.Provider)
    case *SlackChannel:
        fmt.Printf("Slack channel to #%s\n", c.Channel)
    default:
        fmt.Printf("Unknown channel type: %T\n", c)
    }
}
```

`ch.(type)` is only valid inside a `switch`. The variable `c` in each case arm is the concrete type — `*EmailChannel`, `*SMSChannel`, etc. — so you can access type-specific fields.

This is Go's closest equivalent to pattern matching in functional languages. It's not exhaustive (no compiler error for missing cases — that's what the `default` covers), and it works on interfaces, not on typed constants. If you need exhaustive matching on enum values, use a regular switch:

```go
func statusMessage(s NotificationStatus) string {
    switch s {
    case StatusPending:
        return "waiting to be sent"
    case StatusSending:
        return "transmission in progress"
    case StatusDelivered:
        return "delivered successfully"
    case StatusFailed:
        return "delivery failed"
    case StatusRetrying:
        return "retrying after failure"
    default:
        // This runs if someone adds a new constant and forgets to update this function
        return fmt.Sprintf("unknown status: %v", s)
    }
}
```

### Your notes

---

## Comparable Structs and Map Keys

A struct is **comparable** if all its fields are comparable. Comparable types support `==` and `!=`. A comparable struct can be used as a map key.

```go
type ChannelKey struct {
    Type   string
    Region string
}

cache := map[ChannelKey]*ChannelStats{}
key := ChannelKey{"email", "us-west"}
cache[key] = &ChannelStats{Sent: 100}
```

Fields that make a struct non-comparable: slices, maps, functions. If any field is one of these, you can't use the struct as a map key, and you can't use `==` to compare instances.

```go
type NonComparableStruct struct {
    Tags []string  // slice — not comparable
}
// map[NonComparableStruct]int{}  // compile error
```

### Your notes

---

## TypeScript / JavaScript Comparison

| Concept | Go | TypeScript/JS |
|---|---|---|
| Data + behavior | Struct + methods | Class |
| "Inheritance" | Embedding (composition) | `extends` |
| Constructor | `NewXxx()` function | `constructor()` |
| Private fields | Lowercase (package-private) | `private` keyword |
| Method dispatch | Static — resolved at compile time | Virtual — resolved at runtime via prototype chain |
| Enum | Typed constants + iota | `enum` keyword (or `as const` maps) |
| Type guards | Type switch `ch.(type)` | `typeof`, `instanceof`, discriminated unions |
| Interface satisfaction | Implicit (structural) | Explicit `implements` (structural in TypeScript, but declared) |

**The biggest shift:** In TypeScript, a class bundles data, behavior, and access control into one thing, and `extends` creates a prototype chain where methods are looked up dynamically at runtime. In Go, structs are pure data, methods are associated separately, and embedding is compile-time delegation — there's no runtime lookup chain.

This means Go's dispatch is faster (no prototype traversal) but also means you can't hook into a "parent" method mid-call the way you can in OOP polymorphism. `AuditLogger.Log` will never call an overriding `AlertChannel.Log` — there's no vtable, no virtual dispatch, no way for the embedded type to know it's embedded.

**Access control:** TypeScript has `private`, `protected`, `public`. Go has: exported (uppercase) and unexported (lowercase). Unexported is package-level, not file-level. Any file in the same package can access unexported fields. This is a design choice — Go favors packages as the unit of encapsulation, not individual types.
