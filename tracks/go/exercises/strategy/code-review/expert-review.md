# Expert Review -- Payment Processing Strategy Pattern

**Verdict: Request Changes**

This PR introduces multi-provider payment processing, which is a good goal. However, there are several issues ranging from a fundamental design flaw (concrete type where interface should be) to thread safety bugs and interface bloat. The code will need significant revision before it's production-ready.

---

## Critical Issues

### 1. PaymentService uses concrete type instead of interface

**File:** `proposed.go`, `PaymentService` struct and `NewPaymentService` function

```go
type PaymentService struct {
    processor *StripeProcessor // BUG: concrete type, not interface
}

func NewPaymentService(processor *StripeProcessor) *PaymentService {
```

**Problem:** The entire point of the Strategy pattern is to program against an interface, not a concrete type. `PaymentService` accepts `*StripeProcessor`, which means:
- You cannot pass a `*PayPalProcessor` or `*BankTransferProcessor`
- The `SwitchProcessor` method also accepts `*StripeProcessor`
- All the PayPal and BankTransfer code is dead -- it can never be used through `PaymentService`

**Fix:**
```go
type PaymentService struct {
    processor PaymentProcessor // interface type
}

func NewPaymentService(processor PaymentProcessor) *PaymentService {
```

**Severity:** Critical -- this defeats the entire purpose of the PR.

### 2. Data race in StripeProcessor.Charge

**File:** `proposed.go`, `StripeProcessor.Charge` method

```go
func (s *StripeProcessor) Charge(ctx context.Context, amount float64, currency string, customerID string) (string, error) {
    // ...
    // NO MUTEX LOCK HERE
    s.transactionLog = append(s.transactionLog, entry)  // RACE
    // ...
}
```

**Problem:** The `Charge` method writes to `s.transactionLog` without holding the mutex. `Refund` correctly acquires the mutex, but `Charge` does not. If multiple goroutines call `Charge` concurrently (which is expected in a payment service), this is a data race on the slice. Under Go's memory model, this is undefined behavior -- it can corrupt the slice header, lose entries, or crash.

**Fix:**
```go
func (s *StripeProcessor) Charge(...) (string, error) {
    // ... validation ...

    s.mu.Lock()
    s.transactionLog = append(s.transactionLog, entry)
    s.mu.Unlock()

    return txID, nil
}
```

**Severity:** Critical -- data race in financial code. Would be caught by `go test -race`.

### 3. No nil check on processor

**File:** `proposed.go`, `PaymentService.ProcessPayment`

```go
func (ps *PaymentService) ProcessPayment(ctx context.Context, ...) (string, error) {
    txID, err := ps.processor.Charge(ctx, amount, currency, customerID)
```

**Problem:** If `NewPaymentService(nil)` is called (or `SwitchProcessor(nil)`), calling `ProcessPayment` panics with a nil pointer dereference. Payment services must not panic -- they should return errors.

**Fix:** Either validate at construction time:
```go
func NewPaymentService(processor PaymentProcessor) (*PaymentService, error) {
    if processor == nil {
        return nil, fmt.Errorf("processor must not be nil")
    }
    // ...
}
```

Or check before use:
```go
func (ps *PaymentService) ProcessPayment(ctx context.Context, ...) (string, error) {
    if ps.processor == nil {
        return "", fmt.Errorf("no payment processor configured")
    }
    // ...
}
```

**Severity:** Critical -- nil panic in payment path.

---

## Major Concerns

### 4. God Interface: PaymentProcessor has 9 methods

**File:** `proposed.go`, `PaymentProcessor` interface

```go
type PaymentProcessor interface {
    Charge(ctx context.Context, amount float64, currency string, customerID string) (string, error)
    Refund(ctx context.Context, transactionID string, amount float64) error
    GetBalance(ctx context.Context, customerID string) (float64, error)
    Subscribe(ctx context.Context, customerID string, planID string, interval string) (string, error)
    CancelSubscription(ctx context.Context, subscriptionID string) error
    ListTransactions(ctx context.Context, customerID string, from, to time.Time) ([]Transaction, error)
    GenerateInvoice(ctx context.Context, transactionID string) ([]byte, error)
    ValidateCard(ctx context.Context, cardToken string) (bool, error)
    SetWebhookURL(ctx context.Context, url string) error
}
```

**Problem:** This is a "God interface" -- it violates the Interface Segregation Principle. Every implementation must provide all 9 methods, even when most return `"not supported"`. PayPal doesn't support card validation. Bank transfers don't support subscriptions. Yet both must implement those methods.

Adding a new provider requires implementing 9 methods. Adding a new method to the interface breaks all existing implementations. This interface will resist change rather than enable it.

**Fix:** Break into small, focused interfaces:

```go
type Charger interface {
    Charge(ctx context.Context, amount float64, currency string, customerID string) (string, error)
}

type Refunder interface {
    Refund(ctx context.Context, transactionID string, amount float64) error
}

type Subscriber interface {
    Subscribe(ctx context.Context, customerID string, planID string, interval string) (string, error)
    CancelSubscription(ctx context.Context, subscriptionID string) error
}

type TransactionLister interface {
    ListTransactions(ctx context.Context, customerID string, from, to time.Time) ([]Transaction, error)
}
```

Then use interface upgrades to check for optional capabilities:
```go
if sub, ok := processor.(Subscriber); ok {
    sub.Subscribe(ctx, customerID, planID, interval)
}
```

**Severity:** Major -- makes the codebase rigid and discourages adding new providers.

### 5. Strategy holds mutable shared state (transactionLog)

**File:** `proposed.go`, `StripeProcessor.transactionLog`

```go
type StripeProcessor struct {
    apiKey         string
    mu             sync.Mutex
    transactionLog []string
}
```

**Problem:** A payment strategy should be a stateless adapter to an external service. Storing a transaction log inside the strategy:

1. Mixes concerns (logging is not payment processing)
2. Creates thread safety issues (as seen in Bug #2)
3. Grows unboundedly (memory leak in long-running service)
4. Is lost on restart (not persisted)

**Fix:** Move transaction logging to the `PaymentService` level (or a dedicated logging middleware). Strategies should be pure adapters to external payment APIs.

```go
type loggingProcessor struct {
    delegate PaymentProcessor
    logger   *slog.Logger
}

func (l *loggingProcessor) Charge(ctx context.Context, ...) (string, error) {
    l.logger.Info("charging", "amount", amount, "customer", customerID)
    txID, err := l.delegate.Charge(ctx, amount, currency, customerID)
    l.logger.Info("charge result", "txID", txID, "error", err)
    return txID, err
}
```

This is the decorator pattern -- wrap the strategy with logging, keeping both concerns separate.

**Severity:** Major -- architectural concern, memory leak risk, separation of concerns violation.

### 6. SwitchProcessor is not thread-safe

**File:** `proposed.go`, `PaymentService.SwitchProcessor`

```go
func (ps *PaymentService) SwitchProcessor(processor *StripeProcessor) {
    ps.processor = processor
}
```

**Problem:** If one goroutine is calling `ProcessPayment` while another calls `SwitchProcessor`, this is a data race on `ps.processor`. The processor could be swapped mid-transaction.

**Fix:** Either use `sync.RWMutex` or use `atomic.Value`:
```go
type PaymentService struct {
    mu        sync.RWMutex
    processor PaymentProcessor
}

func (ps *PaymentService) SwitchProcessor(p PaymentProcessor) {
    ps.mu.Lock()
    defer ps.mu.Unlock()
    ps.processor = p
}

func (ps *PaymentService) ProcessPayment(ctx context.Context, ...) (string, error) {
    ps.mu.RLock()
    p := ps.processor
    ps.mu.RUnlock()
    return p.Charge(ctx, amount, currency, customerID)
}
```

**Severity:** Major -- data race in concurrent service.

---

## Minor Suggestions

### 7. Use `float64` for currency amounts

```go
Charge(ctx context.Context, amount float64, ...)
```

Using `float64` for monetary values causes rounding errors. `$0.10 + $0.20 != $0.30` in floating-point. Production payment systems use integer cents (or a `Money` type):

```go
type Money struct {
    Amount   int64  // in smallest currency unit (cents, pence)
    Currency string // ISO 4217 code
}
```

This is a well-known issue but worth calling out since this is a payment system.

### 8. Transaction IDs are not globally unique

```go
txID := fmt.Sprintf("stripe_ch_%d", time.Now().UnixNano())
```

`UnixNano` is not guaranteed to be unique across goroutines or even sequential calls on fast hardware. Use UUIDs:

```go
import "crypto/rand"

func generateTxID(prefix string) string {
    b := make([]byte, 16)
    rand.Read(b)
    return fmt.Sprintf("%s_%x", prefix, b)
}
```

### 9. Missing compile-time interface guards

No `var _ PaymentProcessor = (*StripeProcessor)(nil)` guards. Adding these would catch the concrete-type bug at compile time.

### 10. Error messages inconsistent

Stripe returns `"not implemented"`, PayPal returns `"not supported by PayPal integration"`, Bank returns `"not supported"`. Standardize the error messages and consider using sentinel errors:

```go
var ErrNotSupported = fmt.Errorf("operation not supported by this provider")
```

---

## Positive Feedback

### 1. Good use of context.Context
All methods accept `context.Context` as the first parameter, which is correct for operations that may involve network calls. This enables timeout and cancellation propagation.

### 2. Input validation on Charge
All three implementations validate `amount <= 0` in `Charge`. This is good defensive programming.

### 3. Clear naming
The type names (`StripeProcessor`, `PayPalProcessor`, `BankTransferProcessor`) clearly communicate what each type does. The method names are descriptive.

### 4. Good intent
The PR correctly identifies the need for the Strategy pattern. The multi-provider requirement is real, and the overall structure is headed in the right direction. The issues are in execution, not concept.

---

## Summary

| # | Priority | Issue | Category |
|---|----------|-------|----------|
| 1 | Critical | Concrete type instead of interface in PaymentService | Design |
| 2 | Critical | Data race in StripeProcessor.Charge (missing mutex) | Thread safety |
| 3 | Critical | No nil check on processor | Correctness |
| 4 | Major | God interface (9 methods) | Design |
| 5 | Major | Strategy holds mutable logging state | Architecture |
| 6 | Major | SwitchProcessor is not thread-safe | Thread safety |
| 7 | Minor | float64 for currency amounts | Correctness |
| 8 | Minor | Non-unique transaction IDs | Correctness |
| 9 | Minor | Missing interface guards | Best practice |
| 10 | Minor | Inconsistent error messages | Consistency |

**Recommendation:** Request changes. The critical issues (concrete type, data race, nil panic) must be fixed. The God interface should be broken up. The logging concern should be extracted. After those fixes, this PR would be in good shape.
