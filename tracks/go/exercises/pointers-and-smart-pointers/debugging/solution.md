# Debugging Solution

## Bug 1: Value Receiver on `RecordCompletion`

**Location:** `func (s JobStats) RecordCompletion(runtimeMs int64)`

**What goes wrong:**

`RecordCompletion` has a value receiver `s JobStats`. When called, Go copies the entire `JobStats` struct into `s`. The increments `s.CompletedCount++` and `s.TotalRuntimeMs += runtimeMs` modify the local copy. When the method returns, the copy is discarded. The caller's `JobStats` is unchanged.

The symptom: `CompletedCount` and `TotalRuntimeMs` remain 0 after calling `RecordCompletion`.

```go
// Bug
func (s JobStats) RecordCompletion(runtimeMs int64) {
    s.CompletedCount++      // modifies copy — original unchanged
    s.TotalRuntimeMs += runtimeMs  // same
}

// Fix
func (s *JobStats) RecordCompletion(runtimeMs int64) {
    s.CompletedCount++      // modifies through pointer — original changed
    s.TotalRuntimeMs += runtimeMs
}
```

**Why this is subtle:** `RecordFailure` already has a pointer receiver `*JobStats` and works correctly. The inconsistency (`RecordCompletion` by value, `RecordFailure` by pointer) is the tell. Any method that modifies struct fields must use a pointer receiver. See **Pitfall:** [[go-value-receiver-mutation]].

---

## Bug 2: Embedded Pointer Not Initialized in Constructor

**Location:** `NewWorkerPool` — `*JobStats` embedded field is nil

**What goes wrong:**

`WorkerPool` embeds `*JobStats` — a pointer to `JobStats`, not the struct itself. The zero value of any pointer is `nil`. `NewWorkerPool` initializes `maxWorkers` but omits `JobStats`:

```go
return &WorkerPool{
    // JobStats left as nil — nil pointer dereference waiting to happen
    maxWorkers: maxWorkers,
}
```

The first call that uses a promoted `JobStats` method (e.g., `pool.RecordCompletion(...)`) dereferences the nil `*JobStats` field — **panic: nil pointer dereference**.

```go
// Fix option 1: embed by value (preferred — always initialized, no nil risk)
type WorkerPool struct {
    JobStats             // by value — zero value is valid (0, 0, 0)
    workers    []*Worker
    maxWorkers int
}
// No change needed in NewWorkerPool

// Fix option 2: initialize in constructor (if pointer embedding is intentional)
func NewWorkerPool(maxWorkers int) *WorkerPool {
    return &WorkerPool{
        JobStats:   &JobStats{},  // explicitly initialized
        maxWorkers: maxWorkers,
    }
}
```

**Rule:** Embed by value (`T`) when the embedded struct should always be present. Embed by pointer (`*T`) only when it's intentionally optional or lazily initialized — and then always initialize it in the constructor.

---

## Bug 3: Pointer to Loop Variable

**Location:** `RegisterAll` — `&w` inside `for i, w := range workers`

**What goes wrong:**

In `for i, w := range workers`, Go (before 1.22) creates a single `w` variable for the entire loop. Each iteration assigns the next element's value to that same variable. `&w` takes the address of that single variable — all three elements of `refs` end up with the same address.

After the loop, `w` holds the last element (`worker-c`). All three `refs[i]` point to it: `refs[0].ID == refs[1].ID == refs[2].ID == "worker-c"`.

```go
// Bug
for i, w := range workers {
    refs[i] = &w  // address of the loop variable — same address each iteration
}

// Fix 1: shadow the loop variable to create a new one per iteration
for i, w := range workers {
    w := w  // new w scoped to this iteration — different address each time
    refs[i] = &w
}

// Fix 2: take address of the slice element directly
for i := range workers {
    refs[i] = &workers[i]  // each element has its own address in the slice
}
```

**Note:** Go 1.22 changed the behavior so each range iteration creates its own loop variable. If your `go.mod` says `go 1.22` or later, Fix 1 is still good practice and harmless. Fix 2 works in all versions.

Also note: the `AddWorker` call passes `&w` (same bug) — the pool ends up with three copies of the last worker's address. Fix `AddWorker` calls consistently with the fix chosen above.

---

## Bug 4: Non-nil Interface Wrapping a nil Pointer

**Location:** `ServiceHealth.Healthy()` — nil interface check passes for non-nil interface

**What goes wrong:**

A Go interface value has two components: a type pointer and a data pointer. When you assign a typed nil pointer to an interface:

```go
var handler *ConcreteHealthHandler = nil  // typed nil
var checker HealthChecker = handler       // non-nil interface (has type info!)
```

`checker != nil` is **true** — the interface value has a type (`*ConcreteHealthHandler`) even though the data pointer is nil. The existing nil check `if s.checker == nil` doesn't catch this case.

Calling `s.checker.IsHealthy()` then dereferences the nil `*ConcreteHealthHandler` — **panic**.

The fix is to check the concrete type for nil before calling methods on the interface:

```go
// Fix: use a type assertion to extract and check the concrete pointer
func (s *ServiceHealth) Healthy() bool {
    if s.checker == nil {
        return false
    }
    // Check if the interface wraps a nil concrete pointer via reflection,
    // or — better — restructure so nil concrete pointers are never assigned to the interface.
    return s.checker.IsHealthy()
}
```

The most robust fix is **not to assign nil concrete pointers to interfaces at all**. The call site should check before creating `ServiceHealth`:

```go
// In the caller:
var handler *ConcreteHealthHandler = nil
if handler != nil {
    sh = NewServiceHealth("db", handler)
} else {
    sh = NewServiceHealth("db", nil)  // pass nil interface, not nil *ConcreteHealthHandler
}
```

For the test to pass without changing call sites, use a reflect-based check or a type assertion:

```go
import "reflect"

func (s *ServiceHealth) Healthy() bool {
    if s.checker == nil {
        return false
    }
    // Detect non-nil interface wrapping nil pointer
    v := reflect.ValueOf(s.checker)
    if v.Kind() == reflect.Ptr && v.IsNil() {
        return false
    }
    return s.checker.IsHealthy()
}
```

**The deeper lesson:** The safest approach is to never put nil concrete pointers into interfaces. If you're creating an optional behavior, accept `HealthChecker` as `nil` directly and document that nil means "no checker". See **Pitfall:** [[go-nil-interface-wrapping-nil-pointer]].

---

## Summary

| # | Bug | Root Cause | Fix |
|---|-----|-----------|-----|
| 1 | `RecordCompletion` doesn't update stats | Value receiver — mutates a copy | Change to pointer receiver `*JobStats` |
| 2 | Nil panic on `pool.RecordCompletion()` | Embedded `*JobStats` never initialized | Embed by value, or initialize in constructor |
| 3 | All `refs` point to last worker | `&w` in range loop — single loop variable | Shadow `w := w` or use `&workers[i]` |
| 4 | Nil panic in `Healthy()` | Non-nil interface wrapping nil concrete pointer | Avoid assigning nil `*T` to interface; use reflect or restructure |

## Related Pitfalls

- [[go-value-receiver-mutation]] — methods that need to mutate must use pointer receivers
- [[go-embedded-pointer-nil]] — embedded pointer fields are nil until explicitly initialized
- [[go-loop-variable-capture]] — `&loopVar` captures one address across all iterations
- [[go-nil-interface-wrapping-nil-pointer]] — `var p *T = nil; var i I = p` gives non-nil `i`
