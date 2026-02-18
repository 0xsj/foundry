# Debugging Solution: Test Code Bugs

## Bug 1: Silent Assertion (TestProcessEvent_Formats)

**Location:** `TestProcessEvent_Formats`, the condition in each subtest

**The bug:**
```go
if got != got {  // comparing got to itself — always false
    t.Errorf("FormatEventSummary() = %q, want %q", got, tt.wantFmt)
}
```

This condition is `got != got` — a value is never unequal to itself. The test always passes, regardless of what `FormatEventSummary` returns. You could change the function to return an empty string and this test would still pass.

**Why it happens:** Typo — the developer probably wrote `got` twice when they meant `got != tt.wantFmt`. Easy to miss in a code review because the test still compiles and "passes."

**The fix:**
```go
if got != tt.wantFmt {
    t.Errorf("FormatEventSummary() = %q, want %q", got, tt.wantFmt)
}
```

**How to detect:** If your tests always pass even when you introduce a known bug into the function under test, you have a silent assertion. Run `go test` on a deliberately broken version of `FormatEventSummary` to verify the test actually fails.

---

## Bug 2: Parallel Subtest Captures Loop Variable (TestProcessEvent_Parallel)

**Location:** `TestProcessEvent_Parallel`, the `t.Run` callback

**The bug:**
```go
for _, tt := range tests {
    // tt is NOT captured — all t.Run closures share the same tt variable
    t.Run(tt.name, func(t *testing.T) {
        t.Parallel()
        // by the time this runs, the loop may have advanced
        // tt.wantID and tt.wantType may be from the last iteration
        event, err := ProcessEvent(tt.raw)
        ...
    })
}
```

`t.Parallel()` defers the subtest body to run after the parent test function returns. By that time, the `for` loop has finished and `tt` holds its final value (the last test case). All three subtests process `"webhook.ping:evt-003"` instead of their intended inputs.

**Why it happens:** Classic Go closure-over-loop-variable bug. The closure captures the variable `tt`, not its value at the time of capture.

**The fix (pre-Go 1.22):**
```go
for _, tt := range tests {
    tt := tt  // shadow tt with a new variable — each iteration gets its own copy
    t.Run(tt.name, func(t *testing.T) {
        t.Parallel()
        // now tt is the per-iteration copy
        ...
    })
}
```

**The fix (Go 1.22+):**
No fix needed — Go 1.22 changed the semantics of `for` loop variables so each iteration has its own variable. But add a comment explaining this for maintainers on older Go versions.

**How to detect:** Run `go test -v -count=5 -parallel=4 ./...` — flaky failures or wrong event names indicate loop variable capture.

---

## Bug 3: Benchmark Includes Setup Time (BenchmarkProcessEvent)

**Location:** `BenchmarkProcessEvent`, missing `b.ResetTimer()`

**The bug:**
```go
func BenchmarkProcessEvent(b *testing.B) {
    events := make([]string, 1000)
    for i := 0; i < 1000; i++ {
        events[i] = fmt.Sprintf(...)  // this runs while the timer is ticking
    }
    // missing b.ResetTimer() here
    for i := 0; i < b.N; i++ {
        _, _ = ProcessEvent(events[i%len(events)])
    }
}
```

The timer starts when `BenchmarkProcessEvent` is called — before the setup loop. The 1000 `fmt.Sprintf` calls are included in the benchmark's time measurement. The resulting `ns/op` includes amortized setup cost, making `ProcessEvent` appear slower than it is.

**The fix:**
```go
func BenchmarkProcessEvent(b *testing.B) {
    events := make([]string, 1000)
    for i := 0; i < 1000; i++ {
        events[i] = fmt.Sprintf("payment.succeeded:evt-%04d:amount=%d,currency=USD", i, i*100)
    }

    b.ResetTimer()  // start timing HERE — after setup is done

    for i := 0; i < b.N; i++ {
        _, _ = ProcessEvent(events[i%len(events)])
    }
}
```

**How to detect:** If your benchmark's `ns/op` is much higher than expected, or if it changes dramatically when you change setup code size, you're likely missing `b.ResetTimer()`.

---

## Bug 4: Data Race on Shared Slice (TestProcessEvent_Concurrent)

**Location:** `TestProcessEvent_Concurrent`, `results = append(results, event)`

**The bug:**
```go
var results []Event  // plain slice — not safe for concurrent writes

var wg sync.WaitGroup
for _, raw := range raws {
    raw := raw
    wg.Add(1)
    go func() {
        defer wg.Done()
        event, _ := ProcessEvent(raw)
        results = append(results, event)  // DATA RACE: multiple goroutines write results
    }()
}
```

Multiple goroutines call `append(results, event)` concurrently. `append` is not goroutine-safe. Each goroutine reads the current length and capacity of `results` and writes to it — without synchronization, two goroutines can read the same length and both write to the same slot, overwriting each other.

**The fix (mutex):**
```go
var mu sync.Mutex
var results []Event

go func() {
    defer wg.Done()
    event, _ := ProcessEvent(raw)
    mu.Lock()
    results = append(results, event)
    mu.Unlock()
}()
```

**The fix (channel):**
```go
resultCh := make(chan Event, len(raws))

go func() {
    defer wg.Done()
    event, _ := ProcessEvent(raw)
    resultCh <- event
}()

wg.Wait()
close(resultCh)

var results []Event
for e := range resultCh {
    results = append(results, e)
}
```

**How to detect:** `go test -race ./...`. The race detector will print:
```
WARNING: DATA RACE
Write at ... goroutine N
    results = append(results, event)
```

Always run with `-race` when testing concurrent code.

---

## Summary

| # | Bug | Location | Fix |
|---|-----|----------|-----|
| 1 | Silent assertion — `got != got` always false | `TestProcessEvent_Formats` | Change to `got != tt.wantFmt` |
| 2 | Loop variable captured by parallel subtests | `TestProcessEvent_Parallel` | Add `tt := tt` before `t.Run` |
| 3 | Setup time included in benchmark | `BenchmarkProcessEvent` | Add `b.ResetTimer()` after setup |
| 4 | Unsynchronized concurrent writes to slice | `TestProcessEvent_Concurrent` | Add mutex around `append` |

## Related Pitfalls

- Silent assertions are the deadliest test bug — tests pass while giving you false confidence
- Loop variable capture is fixed in Go 1.22, but you'll encounter it in older codebases
- `b.ResetTimer()` is easy to forget — make it a habit to always write it after setup
- The race detector is free confidence for concurrent code — run it in CI

## Related Concepts

- [[fundamentals/testing-fundamentals]] — Go testing package overview
- [[pitfalls/go-closure-loop-gotcha]] — the loop variable capture problem in full
- [[pitfalls/go-test-silent-assertion]] — other forms of tests that don't actually test
