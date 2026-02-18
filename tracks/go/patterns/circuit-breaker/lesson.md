# Circuit Breaker Pattern -- Go

## The Problem Circuit Breaker Solves

In a distributed system, services call other services over the network. Networks fail. Services go down. Databases hit capacity. When a downstream dependency starts failing, the naive approach is to keep retrying every request. This creates a cascade: your service blocks on slow/dead connections, your thread pool fills up, your response times spike, your callers time out, and the failure propagates upstream through the entire dependency graph. One broken service takes down everything.

This is not hypothetical. Netflix famously experienced cascade failures across their microservice architecture, which led them to build Hystrix (now in maintenance mode). The core insight: **if a downstream service is failing, stop calling it**. Return a fast error (or a fallback) instead of waiting for a timeout on every request. Periodically check if the service recovered. This is the Circuit Breaker pattern.

The name comes from electrical circuit breakers. When current exceeds safe limits, the breaker trips open, cutting the circuit. You don't keep pushing electricity through a fault. You wait, then test with a small probe before restoring full flow.

### Where you encounter this in production

- **Payment processing**: Your Stripe/Adyen integration starts timing out. Without a breaker, every checkout request hangs for 30 seconds before failing. With a breaker, you fail fast and show "payment temporarily unavailable" within milliseconds.
- **API gateways**: A gateway proxying requests to 20 backend services. One backend goes down. Without breakers, the gateway's connection pool gets consumed by pending requests to the dead service, starving the healthy services.
- **Database connections**: Your primary database hits max connections. Without a breaker, every query attempt waits for the connection timeout. With a breaker, you switch to read replicas or return cached data.
- **Third-party integrations**: Email delivery, SMS providers, geocoding services -- anything you don't control can fail, and you need to degrade gracefully.
- **Service mesh**: Envoy, Istio, and Linkerd all implement circuit breakers at the infrastructure level. Understanding the pattern helps you configure them correctly.

### Your notes
<!-- Add your insights here during learning -->


---

## The State Machine

A circuit breaker is a state machine with three states. This is the core mental model -- everything else is configuration and optimization.

```
                 failure threshold reached
    ┌─────────┐ ──────────────────────────> ┌──────────┐
    │  CLOSED  │                             │   OPEN   │
    │ (normal) │ <────────────────────────── │  (fail   │
    └─────────┘     success in half-open     │   fast)  │
         ^                                   └──────────┘
         │                                        │
         │         timeout expires                │
         │      ┌──────────────┐                  │
         └───── │  HALF-OPEN   │ <────────────────┘
    success     │  (probing)   │
    threshold   └──────────────┘
    reached           │
                      │ failure in half-open
                      └──────────────> back to OPEN
```

### Closed (Normal Operation)

The breaker starts **closed**. Requests flow through normally. The breaker monitors outcomes:

- On **success**: reset failure counter (or decrement, depending on strategy)
- On **failure**: increment failure counter

When the failure count reaches the **failure threshold**, the breaker **trips open**.

```go
// Conceptual -- not production code yet
type CircuitBreaker struct {
    state        State
    failureCount int
    threshold    int
}

func (cb *CircuitBreaker) Execute(fn func() error) error {
    if cb.state == Open {
        return ErrCircuitOpen
    }

    err := fn()
    if err != nil {
        cb.failureCount++
        if cb.failureCount >= cb.threshold {
            cb.state = Open
            cb.openedAt = time.Now()
        }
        return err
    }

    cb.failureCount = 0 // reset on success
    return nil
}
```

### Open (Failing Fast)

When **open**, the breaker does not execute the wrapped function. It immediately returns an error (`ErrCircuitOpen` or a sentinel). This is the whole point -- you avoid consuming resources on a call you already know will fail.

The breaker starts a **timeout timer**. After the timeout expires, it moves to **half-open**.

Key insight: the "fast fail" is not just about protecting your service. It's about protecting the failing downstream service. If it's overloaded, hammering it with retries makes recovery harder. Backing off gives it room to heal.

### Half-Open (Probing for Recovery)

After the timeout, the breaker enters **half-open**. It allows a **limited number of probe requests** through. This is the recovery test:

- If the probe **succeeds**: the breaker moves back to **closed** (recovered)
- If the probe **fails**: the breaker moves back to **open** (still broken), and the timeout resets

The "limited number" is important. You don't want to send full traffic to a service that might still be struggling. One or a small handful of requests is the probe. If those succeed, you gradually restore traffic.

```go
func (cb *CircuitBreaker) allowRequest() bool {
    switch cb.state {
    case Closed:
        return true
    case Open:
        if time.Since(cb.openedAt) > cb.timeout {
            cb.state = HalfOpen
            return true // allow one probe
        }
        return false
    case HalfOpen:
        return false // already probing, block others
    }
    return false
}
```

### Your notes
<!-- How does this compare to retry with backoff? When would you use one vs the other? -->


---

## Configuration Parameters

Getting the configuration right is the difference between a useful breaker and one that either never trips (useless) or trips on normal variance (harmful).

### Failure Threshold

**What it is:** How many failures before the breaker trips open.

**Too low (e.g., 1-2):** Normal transient errors trip the breaker. A single timeout or network blip cuts off the service. You get false positives -- the breaker becomes a source of outages rather than preventing them.

**Too high (e.g., 100+):** The service is clearly down, but you're still sending 99 requests into the void before the breaker reacts. You absorb the failure rather than isolating it.

**Typical range:** 5-10 consecutive failures for high-traffic services, or a percentage-based threshold (e.g., >50% failure rate in a 10-second window) for lower-traffic services.

### Timeout Duration

**What it is:** How long the breaker stays open before moving to half-open.

**Too short (e.g., 1 second):** The breaker immediately probes a service that hasn't had time to recover. It finds the service still broken, re-opens, probes again 1 second later, creating a rapid cycle that adds load to the failing service.

**Too long (e.g., 5 minutes):** The downstream service recovered 30 seconds in, but you're still returning errors for another 4.5 minutes. Your users see degraded service long after the root cause resolved.

**Typical range:** 15-60 seconds. Adjust based on the downstream service's typical recovery time.

### Success Threshold (Half-Open)

**What it is:** How many successful probes before the breaker closes again.

**Why it exists:** A single success doesn't mean the service is stable. It might have handled one request but will crumble under full load. Multiple successes give more confidence.

**Typical range:** 1-5 successful probes. More cautious systems require more successes. Some implementations gradually increase traffic instead of using a hard threshold.

### Sliding Window vs. Consecutive Failures

Two approaches to counting failures:

**Consecutive failures:** Simple. Reset counter on any success. Trips after N failures in a row. Works well for services that fail completely (down) but poorly for services that are degraded (intermittent failures -- some succeed, some fail).

**Sliding window:** Track success/failure over a time window (e.g., last 60 seconds). Trip when failure rate exceeds a threshold (e.g., >50%). Better for detecting degradation, but more complex to implement.

```go
// Consecutive: simple counter
type ConsecutiveCounter struct {
    failures int
}

func (c *ConsecutiveCounter) RecordSuccess() { c.failures = 0 }
func (c *ConsecutiveCounter) RecordFailure() { c.failures++ }
func (c *ConsecutiveCounter) ShouldTrip(threshold int) bool {
    return c.failures >= threshold
}

// Sliding window: track outcomes over time
type SlidingWindow struct {
    outcomes []outcome // ring buffer
    window   time.Duration
}

func (sw *SlidingWindow) FailureRate() float64 {
    // Count failures in the window
    // Return failures / total
}
```

### Your notes
<!-- What configuration would you use for a payment provider? For a logging service? -->


---

## Implementation from Scratch

Let's build a circuit breaker step by step. The code examples in the `basic/`, `http/`, and `advanced/` subdirectories show the progression. Here we walk through the design decisions.

### Step 1: Define the state machine

```go
type State int

const (
    StateClosed   State = iota // normal operation
    StateOpen                  // failing fast
    StateHalfOpen              // probing for recovery
)

func (s State) String() string {
    switch s {
    case StateClosed:
        return "closed"
    case StateOpen:
        return "open"
    case StateHalfOpen:
        return "half-open"
    default:
        return "unknown"
    }
}
```

Using `iota` for the states is idiomatic Go. The `String()` method makes logging and debugging clearer.

### Step 2: Define the breaker struct

```go
type CircuitBreaker struct {
    mu sync.Mutex

    state        State
    failureCount int
    successCount int // for half-open probing

    // Configuration
    failureThreshold int
    successThreshold int
    timeout          time.Duration

    // Timing
    lastFailureTime time.Time

    // Callbacks (optional but useful)
    onStateChange func(from, to State)
}
```

The `sync.Mutex` is essential. Circuit breakers are shared across goroutines (the whole point is wrapping concurrent HTTP calls). Every state read and write must be synchronized.

### Step 3: The Execute method

```go
func (cb *CircuitBreaker) Execute(fn func() error) error {
    cb.mu.Lock()

    if !cb.canExecute() {
        cb.mu.Unlock()
        return ErrCircuitOpen
    }

    // Move to executing state before unlocking
    isHalfOpen := cb.state == StateHalfOpen
    cb.mu.Unlock()

    // Execute the function WITHOUT holding the lock
    // This is critical -- the wrapped call may take seconds
    err := fn()

    cb.mu.Lock()
    defer cb.mu.Unlock()

    if err != nil {
        cb.recordFailure()
        return err
    }

    cb.recordSuccess(isHalfOpen)
    return nil
}
```

Notice the lock/unlock pattern: we check state under the lock, release the lock during execution (so we don't hold a mutex for the duration of an HTTP call), then re-acquire the lock to record the outcome. This is a common pattern in Go concurrency -- minimize the critical section.

### Step 4: State transition logic

```go
func (cb *CircuitBreaker) canExecute() bool {
    switch cb.state {
    case StateClosed:
        return true
    case StateOpen:
        if time.Since(cb.lastFailureTime) > cb.timeout {
            cb.setState(StateHalfOpen)
            return true
        }
        return false
    case StateHalfOpen:
        // Only allow limited probes
        return cb.successCount == 0 // only one probe at a time
    default:
        return false
    }
}

func (cb *CircuitBreaker) recordFailure() {
    switch cb.state {
    case StateClosed:
        cb.failureCount++
        if cb.failureCount >= cb.failureThreshold {
            cb.setState(StateOpen)
            cb.lastFailureTime = time.Now()
        }
    case StateHalfOpen:
        // Probe failed -- back to open
        cb.setState(StateOpen)
        cb.lastFailureTime = time.Now()
    }
}

func (cb *CircuitBreaker) recordSuccess(wasHalfOpen bool) {
    switch cb.state {
    case StateClosed:
        cb.failureCount = 0
    case StateHalfOpen:
        cb.successCount++
        if cb.successCount >= cb.successThreshold {
            cb.setState(StateClosed)
        }
    }
}

func (cb *CircuitBreaker) setState(newState State) {
    old := cb.state
    cb.state = newState

    // Reset counters on state change
    cb.failureCount = 0
    cb.successCount = 0

    if cb.onStateChange != nil {
        cb.onStateChange(old, newState)
    }
}
```

The key decision: **counters reset on state transition**. When the breaker closes after recovery, the failure count starts at zero. Old failures don't carry over. This prevents phantom trips where ancient failures from before the outage cause an immediate re-open.

### Your notes
<!-- What surprised you about the locking pattern? -->


---

## Integration with Retry and Timeout

Circuit breakers don't replace retries and timeouts -- they compose with them. The layering order matters.

### Correct layering

```
Request → Timeout → Circuit Breaker → Retry → Actual Call
```

- **Timeout** wraps the entire operation. If the circuit breaker + retry takes too long, abort.
- **Circuit breaker** checks first. If open, fail fast before any retry.
- **Retry** handles transient failures that slip through the breaker.

```go
func CallWithResilience(ctx context.Context, breaker *CircuitBreaker) error {
    // Outer timeout
    ctx, cancel := context.WithTimeout(ctx, 10*time.Second)
    defer cancel()

    // Circuit breaker wraps the retry
    return breaker.Execute(func() error {
        // Retry inside the breaker
        return retry.Do(func() error {
            return callUpstream(ctx)
        }, retry.Attempts(3), retry.Delay(100*time.Millisecond))
    })
}
```

### Why this order?

If you put retry outside the breaker, each retry attempt triggers a breaker check. Three retries = three failures recorded in the breaker. A single bad request could trip the breaker because the retry loop inflates the failure count.

If you put the breaker inside the retry, you retry even when the breaker is open. You'd make 3 attempts that all immediately return `ErrCircuitOpen`. Pointless.

The correct order: breaker decides whether to attempt the call. If allowed, the retry handles transient errors within that single breaker "execution." The breaker sees one success or one failure per call, regardless of how many retries happened internally.

### Your notes
<!-- Have you seen retry + circuit breaker composed differently? What happened? -->


---

## Health Checks and Monitoring

A circuit breaker without observability is a black box. When the breaker trips, you need to know.

### What to expose

```go
type BreakerStats struct {
    State           string        `json:"state"`
    Failures        int           `json:"failures"`
    Successes       int           `json:"successes"`
    ConsecutiveFail int           `json:"consecutive_failures"`
    LastFailure     time.Time     `json:"last_failure"`
    TotalRequests   int64         `json:"total_requests"`
    TotalFailures   int64         `json:"total_failures"`
    TotalSuccesses  int64         `json:"total_successes"`
    TotalRejected   int64         `json:"total_rejected"` // fast-failed by open breaker
}
```

### Key metrics to emit

- **breaker.state** (gauge): Current state as a numeric value (0=closed, 1=open, 2=half-open). Alert when any breaker goes to open.
- **breaker.rejected** (counter): Requests rejected by an open breaker. Spikes here mean real user impact.
- **breaker.state_change** (event): Log every state transition with timestamps and the triggering failure. These are your incident timeline.
- **breaker.failure_rate** (gauge): Rolling failure rate for the wrapped service. This is the leading indicator before the breaker trips.

### Integration with health endpoints

```go
// /healthz endpoint
func healthHandler(breakers map[string]*CircuitBreaker) http.HandlerFunc {
    return func(w http.ResponseWriter, r *http.Request) {
        status := make(map[string]string)
        allHealthy := true

        for name, cb := range breakers {
            state := cb.State()
            status[name] = state.String()
            if state != StateClosed {
                allHealthy = false
            }
        }

        if !allHealthy {
            w.WriteHeader(http.StatusServiceUnavailable)
        }
        json.NewEncoder(w).Encode(status)
    }
}
```

### Your notes
<!-- What alerts would you set up for circuit breakers in production? -->


---

## Cross-Language Comparison

### TypeScript: opossum library

TypeScript's most popular circuit breaker library is `opossum`. It wraps async functions (Promises) and provides an event-based API:

```typescript
import CircuitBreaker from 'opossum';

const options = {
  timeout: 3000,           // If function takes longer, trigger failure
  errorThresholdPercentage: 50,  // Trip at 50% failure rate
  resetTimeout: 30000,     // Try again after 30 seconds
};

const breaker = new CircuitBreaker(callPaymentService, options);

breaker.on('open', () => console.log('Circuit opened'));
breaker.on('halfOpen', () => console.log('Circuit half-open'));
breaker.on('close', () => console.log('Circuit closed'));

// Fallback when open
breaker.fallback(() => ({ status: 'cached', amount: 0 }));

const result = await breaker.fire(paymentRequest);
```

**Key differences from Go:**
- Event-driven API (`.on('open', ...)`) vs callbacks/interfaces
- Built-in timeout handling (Go usually composes `context.WithTimeout` separately)
- Percentage-based thresholds by default (Go libraries tend to use consecutive counts)
- Promise-based naturally (async/await), while Go uses goroutines and channels

### Rust: conceptual approach

Rust doesn't have a dominant circuit breaker crate, but the pattern maps naturally to Rust's type system:

```rust
// Conceptual Rust approach
enum State {
    Closed { failures: u32 },
    Open { since: Instant },
    HalfOpen { successes: u32 },
}

impl<F, T, E> CircuitBreaker<F, T, E>
where
    F: Fn() -> Result<T, E>,
{
    fn call(&mut self) -> Result<T, CircuitError<E>> {
        match &self.state {
            State::Open { since } if since.elapsed() < self.timeout => {
                Err(CircuitError::Open)
            }
            _ => {
                // Execute and record outcome
            }
        }
    }
}
```

**Key differences from Go:**
- Enum with associated data replaces struct + state field -- each state carries only the data it needs
- `Result<T, E>` forces callers to handle the circuit-open case at compile time
- No need for a mutex in single-threaded async contexts (use `Arc<Mutex<>>` when shared across tasks)
- Pattern matching on state is more ergonomic than switch statements

### Your notes
<!-- Which approach do you find most natural? -->


---

## Real-World Libraries

### gobreaker (Sony)

The most widely-used circuit breaker library in Go. Clean API, well-tested, production-proven at Sony.

```go
import "github.com/sony/gobreaker/v2"

cb := gobreaker.NewCircuitBreaker[[]byte](gobreaker.Settings{
    Name:        "payment-service",
    MaxRequests: 3,              // half-open: allow 3 concurrent probes
    Interval:    60 * time.Second, // sliding window for failure counting
    Timeout:     30 * time.Second, // how long to stay open
    ReadyToTrip: func(counts gobreaker.Counts) bool {
        return counts.ConsecutiveFailures > 5
    },
    OnStateChange: func(name string, from, to gobreaker.State) {
        log.Printf("breaker %s: %s -> %s", name, from, to)
    },
})

result, err := cb.Execute(func() ([]byte, error) {
    return callPaymentService(ctx, req)
})
```

**Why gobreaker is worth studying:**
- Generic type parameter (`CircuitBreaker[T]`) for the return value
- `ReadyToTrip` callback lets you define custom trip logic (percentage, consecutive, weighted, etc.)
- `Counts` struct gives you access to total requests, successes, failures, consecutive successes/failures
- Thread-safe with internal mutex
- Battle-tested at scale

### Netflix Hystrix (historical context)

Hystrix was the library that popularized circuit breakers in microservices. It's now in maintenance mode (Netflix moved to adaptive concurrency limits), but its concepts shaped every circuit breaker library that followed:

- **Thread pool isolation**: Each dependency gets its own thread pool. A slow dependency can't starve others.
- **Bulkhead pattern**: Limit concurrent requests to a dependency (related to but distinct from circuit breaking).
- **Request collapsing**: Batch multiple requests to the same dependency.
- **Dashboard**: Real-time visualization of breaker states across the fleet.

The Go equivalent of Hystrix's ideas: gobreaker for circuit breaking, `semaphore.Weighted` for bulkheading, `singleflight` for request collapsing.

### Your notes
<!-- Have you seen circuit breakers in infrastructure (Envoy, Istio)? How do they compare? -->


---

## Anti-Patterns

### 1. Circuit breaker on local operations

```go
// DON'T: circuit breaker around an in-memory operation
result, err := breaker.Execute(func() error {
    return json.Unmarshal(data, &config)
})
```

Circuit breakers are for **network calls to external dependencies**. Local operations that fail will fail consistently (a parse error won't heal after 30 seconds). The breaker adds overhead and never provides benefit.

### 2. Too aggressive thresholds

```go
// DON'T: trip after a single failure
Settings{
    ReadyToTrip: func(c Counts) bool {
        return c.ConsecutiveFailures > 0 // trips on first error
    },
}
```

Transient errors are normal. A single timeout, DNS hiccup, or TCP reset doesn't mean the service is down. You need a threshold that distinguishes "the service is degraded" from "one request failed."

### 3. Shared breaker for unrelated services

```go
// DON'T: one breaker for all external calls
var globalBreaker = NewCircuitBreaker(...)

func CallPayments() { globalBreaker.Execute(callStripe) }
func CallEmail()    { globalBreaker.Execute(callSendGrid) }
func CallAuth()     { globalBreaker.Execute(callAuth0) }
```

Stripe going down should not prevent you from sending emails. Each external dependency (or even each endpoint on the same dependency) should have its own breaker.

### 4. No fallback strategy

```go
// DON'T: just return the error
result, err := breaker.Execute(callUpstream)
if err != nil {
    return nil, err // user gets a 500
}
```

When the breaker is open, you have an opportunity to degrade gracefully:
- Return cached data
- Return a default value
- Route to a backup provider
- Queue the request for later processing

### 5. Ignoring which errors should count as failures

```go
// DON'T: count 400 Bad Request as a circuit breaker failure
func callUpstream() error {
    resp, err := http.Get(url)
    if err != nil {
        return err // network error -- yes, count this
    }
    if resp.StatusCode != 200 {
        return fmt.Errorf("status: %d", resp.StatusCode) // 400? 404? Not a service failure
    }
    return nil
}
```

Client errors (4xx) are not service failures -- they're the caller's bug. Only count server errors (5xx) and network errors as failures. Some breaker implementations let you provide a classifier function:

```go
IsSuccessful: func(err error) bool {
    var httpErr *HTTPError
    if errors.As(err, &httpErr) {
        return httpErr.StatusCode < 500 // 4xx is "successful" for the breaker
    }
    return err == nil
}
```

### Your notes
<!-- Which anti-pattern have you seen (or committed) in your own code? -->


---

## Summary

| Concept | Key Point |
|---------|-----------|
| **Purpose** | Prevent cascade failures by fast-failing when a dependency is down |
| **States** | Closed (normal) -> Open (failing fast) -> Half-Open (probing) |
| **Failure threshold** | 5-10 consecutive or >50% in a window -- tune per dependency |
| **Timeout** | 15-60 seconds -- match downstream recovery time |
| **Success threshold** | 1-5 probes in half-open -- more = more cautious recovery |
| **Composition** | Timeout > Circuit Breaker > Retry (outside to inside) |
| **Per-dependency** | Each external service/endpoint gets its own breaker |
| **Fallback** | Cache, default, backup provider, or queue for later |
| **Monitoring** | State changes, rejection count, failure rate -- these are your incident signals |

### Next steps

- **Exercise**: Build a circuit breaker for a payment gateway -- see `exercises/circuit-breaker/`
- **Related patterns**: [[strategy]] (for pluggable fallback strategies), [[observer]] (for breaker event monitoring), [[decorator]] (circuit breaker wraps calls -- that's decoration)
- **Related concepts**: [[retry]], [[timeout]], [[bulkhead]], [[rate-limiting]]
