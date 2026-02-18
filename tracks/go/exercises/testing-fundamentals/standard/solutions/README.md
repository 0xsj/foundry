# Solution: Token-Bucket Rate Limiter Tests

## Approach

The test suite uses four techniques, each addressing a different testing concern:

### 1. StubClock — time control without sleeping

Defined in the test file (not the package). Implements the `Clock` interface with a controllable `now` field. The `Advance` method moves time forward explicitly. This eliminates `time.Sleep` from tests entirely.

The key insight: **any dependency that makes tests slow, flaky, or hard to assert on should be abstracted behind an interface and injected.** Time is the most common example. Others: random number generators, UUID generators, external HTTP calls.

### 2. `t.Helper()` assertion helpers

`mustAllow` and `mustDeny` reduce boilerplate. More importantly, `t.Helper()` makes failures point to the test that called the helper — not to line 3 of `mustAllow`. Without it, you'd see `ratelimiter_test.go:25: Allow() = false` instead of the test case that triggered it.

### 3. Table-driven tests

The `TestAllow_TableDriven` test covers four rate configurations in one function. Adding a new configuration is one struct literal. The test structure is visible at a glance.

### 4. Concurrency test with race detector

`TestAllow_Concurrent` fires 50 goroutines and counts allowed requests. It's not a thorough race analysis on its own — but running it with `go test -race` instruments the binary so the runtime detects any concurrent access to unsynchronized shared state. Always run with `-race` on concurrent code.

## Key Decisions

- **`StubClock` in test file:** Test doubles are test artifacts. They don't belong in the package under test. Putting them in `_test.go` means they're never shipped.

- **White-box testing (`package ratelimiter`):** The test file uses the same package name, giving access to `rl.tokens`. This lets us write more precise assertions (checking token counts directly). The tradeoff: tests are coupled to internal state. If you refactor how tokens are stored, tests break. For this module, the internal state is simple enough that the tradeoff is worth it.

- **`BenchmarkAllow` with `b.N+1` rate:** Set the rate high enough that `Allow()` never denies during the benchmark. We're measuring the cost of the allowed path. A separate `BenchmarkAllow_Denied` measures the deny path — they can have different costs (branch prediction, lock contention patterns).

- **Zero allocations target:** `Allow()` should be zero-alloc on the hot path. The benchmark confirms this. If it ever shows `1 allocs/op`, something changed that causes a heap escape — investigate immediately.

## Comparison Table: Variants

| Approach | Tradeoffs |
|----------|-----------|
| White-box tests (same package) | Access to internals; coupled to implementation; faster iteration during development |
| Black-box tests (`_test` package) | Only tests public API; survives internal refactors; less visibility into state |
| Fuzz testing | Would find edge cases in `Advance` and `window` calculations; overkill here but valuable for parsers and serializers |

## Performance Notes

Expected benchmark results (Apple M-series, Go 1.22):

```
BenchmarkAllow-8         50000000   24 ns/op   0 B/op   0 allocs/op
BenchmarkAllow_Denied-8  80000000   15 ns/op   0 B/op   0 allocs/op
```

The deny path is slightly faster — it short-circuits after the token check without doing the subtraction and store. Both should be zero-alloc.
