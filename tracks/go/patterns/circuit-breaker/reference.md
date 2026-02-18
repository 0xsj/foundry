# Circuit Breaker Pattern -- Go Reference

## State Machine Specification

### States

| State | Description | Allows Requests | Transitions To |
|-------|-------------|-----------------|----------------|
| **Closed** | Normal operation. Requests pass through. Failures are counted. | Yes (all) | Open (when failure threshold reached) |
| **Open** | Fast-fail mode. Requests are immediately rejected. | No | Half-Open (after timeout expires) |
| **Half-Open** | Recovery probe. Limited requests allowed to test if dependency recovered. | Yes (limited) | Closed (on success threshold) or Open (on any failure) |

### Transition Rules

```
Closed -> Open:
  Trigger: failureCount >= failureThreshold
  Action:  Record timestamp, reset counters, start timeout timer

Open -> Half-Open:
  Trigger: time.Since(lastFailure) > timeout
  Action:  Reset success counter, allow limited probes

Half-Open -> Closed:
  Trigger: successCount >= successThreshold
  Action:  Reset all counters

Half-Open -> Open:
  Trigger: Any failure during probing
  Action:  Record timestamp, reset counters, restart timeout
```

### Counter Behavior

| Event | Closed State | Open State | Half-Open State |
|-------|-------------|------------|-----------------|
| Success | Reset failure count | N/A (no execution) | Increment success count |
| Failure | Increment failure count | N/A (no execution) | Transition to Open |
| State entry | Reset all counters | Record open timestamp | Reset success count |

---

## gobreaker v2 API Reference

**Import:** `github.com/sony/gobreaker/v2`

**Source:** https://github.com/sony/gobreaker

### Types

```go
// State represents the state of a CircuitBreaker.
type State int

const (
    StateClosed   State = iota
    StateHalfOpen
    StateOpen
)

// Counts holds the numbers of requests and their successes/failures.
type Counts struct {
    Requests             uint32
    TotalSuccesses       uint32
    TotalFailures        uint32
    ConsecutiveSuccesses uint32
    ConsecutiveFailures  uint32
}

// Settings configures a CircuitBreaker.
type Settings struct {
    // Name is the name of the CircuitBreaker.
    Name string

    // MaxRequests is the maximum number of requests allowed to pass
    // through when the CircuitBreaker is half-open.
    // If MaxRequests is 0, the CircuitBreaker allows only 1 request.
    MaxRequests uint32

    // Interval is the cyclic period of the closed state for the
    // CircuitBreaker to clear the internal Counts.
    // If Interval is less than or equal to 0, the CircuitBreaker
    // doesn't clear internal Counts during the closed state.
    Interval time.Duration

    // Timeout is the period of the open state, after which the
    // state of the CircuitBreaker becomes half-open.
    // If Timeout is less than or equal to 0, the timeout value
    // of the CircuitBreaker is set to 60 seconds.
    Timeout time.Duration

    // ReadyToTrip is called with a copy of Counts whenever a
    // request fails in the closed state.
    // If ReadyToTrip returns true, the CircuitBreaker will be
    // placed into the open state.
    // If ReadyToTrip is nil, default ReadyToTrip is used.
    // Default ReadyToTrip returns true when the number of
    // consecutive failures is more than 5.
    ReadyToTrip func(counts Counts) bool

    // OnStateChange is called whenever the state of the
    // CircuitBreaker changes.
    OnStateChange func(name string, from State, to State)

    // IsSuccessful is called with the error returned from a
    // request. If IsSuccessful returns true, the error is counted
    // as a success. Otherwise the error is counted as a failure.
    // If IsSuccessful is nil, default IsSuccessful is used,
    // which returns false for all non-nil errors.
    IsSuccessful func(err error) bool
}

// CircuitBreaker is a state machine to prevent sending requests
// that are likely to fail.
type CircuitBreaker[T any] struct {
    // contains filtered or unexported fields
}
```

### Constructor

```go
func NewCircuitBreaker[T any](st Settings) *CircuitBreaker[T]
```

Creates a new CircuitBreaker with the given Settings. The type parameter `T` is the return type of the wrapped function.

### Methods

```go
// Execute runs the given request if the CircuitBreaker accepts it.
// Execute returns an error instantly if the CircuitBreaker rejects the request.
// Otherwise, Execute returns the result of the request.
// If a panic occurs in the request, the CircuitBreaker handles it as an error
// and causes the same panic again.
func (cb *CircuitBreaker[T]) Execute(req func() (T, error)) (T, error)

// Name returns the name of the CircuitBreaker.
func (cb *CircuitBreaker[T]) Name() string

// State returns the current state of the CircuitBreaker.
func (cb *CircuitBreaker[T]) State() State

// Counts returns internal counters.
func (cb *CircuitBreaker[T]) Counts() Counts
```

### Error Types

```go
// ErrTooManyRequests is returned when the CB state is half open and
// the requests count is over the cb maxRequests.
var ErrTooManyRequests = errors.New("too many requests")

// ErrOpenState is returned when the CB state is open.
var ErrOpenState = errors.New("circuit breaker is open")
```

### Usage Example

```go
cb := gobreaker.NewCircuitBreaker[*http.Response](gobreaker.Settings{
    Name:        "upstream-api",
    MaxRequests: 3,
    Interval:    60 * time.Second,
    Timeout:     30 * time.Second,
    ReadyToTrip: func(counts gobreaker.Counts) bool {
        failureRatio := float64(counts.TotalFailures) / float64(counts.Requests)
        return counts.Requests >= 10 && failureRatio >= 0.6
    },
    OnStateChange: func(name string, from, to gobreaker.State) {
        log.Printf("[circuit-breaker] %s: %s -> %s", name, from, to)
    },
    IsSuccessful: func(err error) bool {
        if err == nil {
            return true
        }
        // Don't count client errors as failures
        var httpErr *HTTPError
        if errors.As(err, &httpErr) && httpErr.Code < 500 {
            return true
        }
        return false
    },
})

resp, err := cb.Execute(func() (*http.Response, error) {
    return http.Get("https://api.example.com/data")
})
```

---

## Configuration Best Practices

### Per-Service Tuning

| Service Type | Failure Threshold | Timeout | Success Threshold | Rationale |
|-------------|-------------------|---------|-------------------|-----------|
| Payment provider | 3-5 consecutive | 30-60s | 2-3 probes | Low tolerance for failure; providers recover slowly |
| Email/SMS | 5-10 consecutive | 15-30s | 1-2 probes | More tolerant; usually recovers quickly |
| Cache (Redis) | 3-5 consecutive | 5-10s | 1 probe | Should recover fast; impact of open breaker is high |
| Database | 2-3 consecutive | 10-30s | 2 probes | Critical dependency; fast detection needed |
| Analytics/logging | 10-20 or 50% rate | 60s | 1 probe | Non-critical; tolerate more errors before tripping |

### Environment-Specific Overrides

```go
type BreakerConfig struct {
    FailureThreshold int           `yaml:"failure_threshold"`
    Timeout          time.Duration `yaml:"timeout"`
    SuccessThreshold int           `yaml:"success_threshold"`
    MaxHalfOpen      int           `yaml:"max_half_open_requests"`
    Window           time.Duration `yaml:"window"`
}

// Defaults per environment
var defaults = map[string]BreakerConfig{
    "production": {
        FailureThreshold: 5,
        Timeout:          30 * time.Second,
        SuccessThreshold: 3,
        MaxHalfOpen:      3,
        Window:           60 * time.Second,
    },
    "staging": {
        FailureThreshold: 3,
        Timeout:          15 * time.Second,
        SuccessThreshold: 1,
        MaxHalfOpen:      1,
        Window:           30 * time.Second,
    },
    "development": {
        FailureThreshold: 1,
        Timeout:          5 * time.Second,
        SuccessThreshold: 1,
        MaxHalfOpen:      1,
        Window:           10 * time.Second,
    },
}
```

### What NOT to do

| Anti-Pattern | Problem | Fix |
|--------------|---------|-----|
| Same config for all services | Payment and logging have very different failure profiles | Tune per dependency |
| Hardcoded thresholds | Can't adjust without redeployment | Use config files or feature flags |
| No monitoring of breaker state | Breaker trips silently; no one investigates | Emit metrics on every state change |
| Counting 4xx as failures | Client bugs trip the breaker | Use `IsSuccessful` to classify errors |
| Breaker on read-through cache | Cache miss triggers breaker, preventing fallback to origin | Only breaker on the origin call |

---

## Metrics and Observability

### Prometheus Metrics

```go
import "github.com/prometheus/client_golang/prometheus"

var (
    breakerState = prometheus.NewGaugeVec(
        prometheus.GaugeOpts{
            Name: "circuit_breaker_state",
            Help: "Current state of circuit breaker (0=closed, 1=open, 2=half-open)",
        },
        []string{"service"},
    )

    breakerTransitions = prometheus.NewCounterVec(
        prometheus.CounterOpts{
            Name: "circuit_breaker_transitions_total",
            Help: "Total number of circuit breaker state transitions",
        },
        []string{"service", "from", "to"},
    )

    breakerRejected = prometheus.NewCounterVec(
        prometheus.CounterOpts{
            Name: "circuit_breaker_rejected_total",
            Help: "Total number of requests rejected by open circuit breaker",
        },
        []string{"service"},
    )

    breakerDuration = prometheus.NewHistogramVec(
        prometheus.HistogramOpts{
            Name:    "circuit_breaker_request_duration_seconds",
            Help:    "Duration of requests passing through circuit breaker",
            Buckets: prometheus.DefBuckets,
        },
        []string{"service", "result"},
    )
)
```

### Structured Logging

```go
// Log state transitions with context
func logStateChange(name string, from, to State) {
    slog.Warn("circuit breaker state change",
        "breaker", name,
        "from", from.String(),
        "to", to.String(),
        "timestamp", time.Now().UTC(),
    )
}

// Log rejected requests
func logRejection(name string) {
    slog.Info("circuit breaker rejected request",
        "breaker", name,
        "state", "open",
    )
}
```

### Alerting Rules (Prometheus/Alertmanager)

```yaml
groups:
  - name: circuit-breaker
    rules:
      - alert: CircuitBreakerOpen
        expr: circuit_breaker_state == 1
        for: 30s
        labels:
          severity: warning
        annotations:
          summary: "Circuit breaker {{ $labels.service }} is open"
          description: "The circuit breaker for {{ $labels.service }} has been open for 30 seconds."

      - alert: CircuitBreakerFlapping
        expr: increase(circuit_breaker_transitions_total[5m]) > 10
        labels:
          severity: critical
        annotations:
          summary: "Circuit breaker {{ $labels.service }} is flapping"
          description: "More than 10 state transitions in 5 minutes indicates instability."
```

---

## Related Patterns

| Pattern | Relationship |
|---------|-------------|
| **Retry** | Compose inside the breaker. Retry handles transient errors; breaker handles sustained failure. |
| **Timeout** | Compose outside the breaker. Prevents the entire resilience stack from blocking indefinitely. |
| **Bulkhead** | Limits concurrent requests to a dependency. Complementary to circuit breaking. |
| **Fallback** | Provides alternative behavior when the breaker is open. Strategy pattern for degraded responses. |
| **Health Check** | Active probing (vs. circuit breaker's passive monitoring). Can inform breaker state. |
| **Rate Limiter** | Protects the callee from overload. Circuit breaker protects the caller from a failing callee. |

---

## References

- [Sony gobreaker](https://github.com/sony/gobreaker) -- Most popular Go implementation
- [Martin Fowler: Circuit Breaker](https://martinfowler.com/bliki/CircuitBreaker.html) -- Original pattern description
- [Microsoft: Circuit Breaker Pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker) -- Azure architecture guide
- [Netflix Hystrix](https://github.com/Netflix/Hystrix/wiki) -- Historical reference (maintenance mode)
- [Release It!](https://pragprog.com/titles/mnee2/release-it-second-edition/) by Michael Nygard -- The book that popularized stability patterns
