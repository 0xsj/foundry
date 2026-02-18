# Variants: Rate Limiter Test Approaches

## Variant A: Black-Box Tests (`package ratelimiter_test`)

The reference solution uses `package ratelimiter` (white-box) to access internal state like `rl.tokens`. An alternative is black-box testing with `package ratelimiter_test`:

```go
package ratelimiter_test

import (
    "testing"
    "time"
    "foundry/exercises/testing-fundamentals"  // import as external package
)
```

**Trade-offs:**

| Aspect | White-box (`package ratelimiter`) | Black-box (`package ratelimiter_test`) |
|--------|-----------------------------------|-----------------------------------------|
| Access | Can inspect internal fields | Only public API |
| Coupling | Tests break if internals change | Survives internal refactors |
| Precision | Can assert exact token counts | Must infer state from behavior |
| Use case | Unit tests during active development | API contract tests |

Most real Go projects use white-box for detailed unit tests and black-box for integration/API contract tests. Both can coexist in the same package using two test files with different package declarations.

## Variant B: Golden File Testing for Complex Outputs

If `FormatEventSummary` produced complex multi-line output, you'd reach for golden files instead of hardcoded expected values:

```go
func TestFormatSummary(t *testing.T) {
    event := buildTestEvent()
    got := FormatSummary(event)

    golden := filepath.Join("testdata", t.Name()+".txt")
    if *update {
        os.WriteFile(golden, []byte(got), 0644)
        return
    }

    want, err := os.ReadFile(golden)
    if err != nil {
        t.Fatalf("read golden: %v", err)
    }
    if string(want) != got {
        t.Errorf("output mismatch\ngot:\n%s\nwant:\n%s", got, want)
    }
}
```

Run `go test -update ./...` to regenerate golden files when output intentionally changes.

**When to use:** Large outputs (HTML reports, JSON schemas, SQL queries), outputs where a diff is more readable than an `Errorf` message.

## Variant C: Property-Based / Fuzz Testing

Instead of specific test cases, define invariants that must hold for *all* inputs:

```go
func FuzzAllow(f *testing.F) {
    f.Add(1, int64(time.Second), 1)
    f.Add(100, int64(time.Minute), 50)

    f.Fuzz(func(t *testing.T, rate int, windowNs int64, requests int) {
        if rate <= 0 || windowNs <= 0 || requests < 0 || requests > 1000 {
            return // skip degenerate inputs
        }

        window := time.Duration(windowNs)
        rl := New(rate, window, newStubClock())

        var allowed int
        for i := 0; i < requests; i++ {
            if rl.Allow() {
                allowed++
            }
        }

        // Invariant: allowed must never exceed rate
        if allowed > rate {
            t.Errorf("allowed %d > rate %d (requests=%d)", allowed, rate, requests)
        }
    })
}
```

Fuzzing finds the invariant violations you didn't think to write as test cases.

**When to use:** Parsers, serializers, algorithms with mathematical properties. Less useful for simple CRUD logic.
