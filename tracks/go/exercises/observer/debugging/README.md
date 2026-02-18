# Debugging Exercise -- Event Notification System

## Symptoms

The team built an in-process event notification system for a payment processing service. During load testing, they observe the following issues:

1. **Race condition detected.** Running `go test -race` reports data races on the handler slice.
2. **Deadlock under load.** The service hangs when a handler tries to subscribe a new handler during event notification. The stack trace shows goroutines waiting for a lock.
3. **Goroutine leak.** After unsubscribing a channel-based subscriber, the goroutine monitoring that channel never exits. Over time, memory usage climbs.
4. **Nil callback panic.** Occasionally, the system panics with `runtime error: invalid memory address or nil pointer dereference` during notification.

Example output from `go test -race`:

```
==================
WARNING: DATA RACE
Write at 0x00c0000b4060 by goroutine 8:
  main.(*EventBus).Subscribe()
      buggy.go:45 +0x...
Previous read at 0x00c0000b4060 by goroutine 7:
  main.(*EventBus).Publish()
      buggy.go:55 +0x...
==================
panic: runtime error: invalid memory address or nil pointer dereference
```

## Context

The event bus is used internally to decouple the payment processing pipeline from side effects like audit logging, fraud detection, and notification sending. Multiple goroutines call `Publish` and `Subscribe` concurrently.

## Your Task

1. Read through `buggy.go` and identify all four root causes
2. Fix each bug
3. Verify tests pass: `go test -race`
4. Document your debugging process below

## Files

- `buggy.go` -- The code with four intentional bugs
- `buggy_test.go` -- Tests that expose the bugs (some use `-race`)
- `solution.md` -- Explanation of each bug (don't peek until you've tried!)

---

## Your Debugging Process

### Hypothesis 1
What I thought the problem was:

Result:

### Hypothesis 2
What I thought the problem was:

Result:

### Hypothesis 3
What I thought the problem was:

Result:

### Hypothesis 4
What I thought the problem was:

Result:

### Root Causes
What the actual problems were:

### The Fixes
What I changed:

### Lessons
What I learned from this:
