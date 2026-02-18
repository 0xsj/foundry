# Exercise: Testing a Token-Bucket Rate Limiter

## Scenario

Your team just shipped a token-bucket rate limiter to protect a payment processing API from bursts. The implementation is complete and checked in. What's missing is the test suite — a new engineer is joining next week and the code has zero tests. Your job is to write a comprehensive test suite before code review.

The implementation is in `starter/ratelimiter.go`. You write all the tests in `starter/ratelimiter_test.go`. The file is empty — the exercise is writing the tests, not the implementation.

## Brief

Write a test suite for `RateLimiter` that covers:

1. **Basic behavior** — allow/deny semantics, refill logic
2. **Table-driven tests** — cover multiple rate configurations with a single test function
3. **Test doubles** — use a `StubClock` to control time instead of sleeping
4. **Benchmarks** — measure `Allow()` throughput and allocation count
5. **Edge cases** — zero tokens, exactly at limit, burst behavior

## Acceptance Criteria

- [ ] At least one table-driven test using an anonymous struct slice and `t.Run`
- [ ] A `StubClock` test double that satisfies the `Clock` interface — no `time.Sleep` anywhere in tests
- [ ] Test for cooldown/refill: advance stub clock, verify tokens are replenished
- [ ] Test for concurrent safety: run multiple goroutines calling `Allow()` simultaneously, use `-race` flag to verify
- [ ] At least one `t.Helper()` assertion helper used in multiple test cases
- [ ] A `BenchmarkAllow` that uses `b.ResetTimer()` and reports allocations
- [ ] Tests pass with `go test -race ./starter/`
- [ ] All test names are descriptive (no `#00`, `#01` subtests)

## The Rate Limiter Interface

```go
// Clock is the time dependency — swap with StubClock in tests.
type Clock interface {
    Now() time.Time
}

// RateLimiter implements a token-bucket algorithm.
type RateLimiter struct { ... }  // internal fields — white-box tests can access them

// New creates a rate limiter that allows `rate` requests per `window`.
// It uses `clock` as its time source (inject RealClock{} in production,
// StubClock in tests).
func New(rate int, window time.Duration, clock Clock) *RateLimiter

// Allow reports whether the current request is within the rate limit.
// Returns true and consumes one token if allowed; returns false if the
// bucket is empty.
func (rl *RateLimiter) Allow() bool

// Tokens returns the current number of available tokens.
func (rl *RateLimiter) Tokens() int
```

## Constraints

- Standard library only — no third-party packages
- No `time.Sleep` in tests — use the `StubClock`
- Define `StubClock` in the test file (it's a test double — shouldn't ship in the binary)
- The test file package must be `package ratelimiter` (white-box — you'll want to inspect internal state)

## Hints

<details>
<summary>Hint 1: StubClock structure</summary>

A stub clock needs to satisfy `Clock` and let you control time:

```go
type StubClock struct {
    t time.Time
}

func (s *StubClock) Now() time.Time { return s.t }
func (s *StubClock) Advance(d time.Duration) { s.t = s.t.Add(d) }
```

Then inject it: `rl := New(5, time.Minute, &StubClock{t: time.Unix(0, 0)})`
</details>

<details>
<summary>Hint 2: Table test for rate configurations</summary>

Think about varying the `rate` and `window` parameters:

```go
tests := []struct {
    name     string
    rate     int
    window   time.Duration
    requests int       // how many to make
    wantAllowed int    // how many should be allowed
}{...}
```
</details>

<details>
<summary>Hint 3: Concurrency test</summary>

```go
var wg sync.WaitGroup
allowed := atomic.Int32{}
for i := 0; i < 100; i++ {
    wg.Add(1)
    go func() {
        defer wg.Done()
        if rl.Allow() {
            allowed.Add(1)
        }
    }()
}
wg.Wait()
// check: allowed.Load() == int32(rate)
```

Run with: `go test -race ./starter/`
</details>

<details>
<summary>Hint 4: What to benchmark</summary>

```go
func BenchmarkAllow(b *testing.B) {
    // Setup (not timed)
    rl := New(b.N+1, time.Hour, RealClock{})  // plenty of tokens so Allow() never denies
    b.ResetTimer()
    b.ReportAllocs()

    for i := 0; i < b.N; i++ {
        rl.Allow()
    }
}
```

A zero-allocation result (`0 allocs/op`) is the target — `Allow()` should not allocate on the happy path.
</details>
