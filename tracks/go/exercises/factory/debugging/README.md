# Debugging Exercise -- Service Registry Factory

## Symptoms

A service registry factory is behaving unpredictably in production:

1. **All services resolve to the same configuration.** After registering 3 services (auth, billing, notifications), calling `Resolve("auth")` returns the notifications service config. The same thing happens for `Resolve("billing")`.

2. **Nil pointer dereference panic.** Calling `Create("unknown-service")` panics instead of returning an error. The stack trace points to a method call on a nil pointer.

3. **Data race detected.** Running with `go test -race` shows a data race in the registration logic when multiple goroutines register services concurrently.

4. **Type assertion panic.** Calling `GetHTTPService("api-gateway")` panics with `interface conversion: Service is *TCPService, not *HTTPService`. But the service was registered as HTTP.

Example output:
```
--- FAIL: TestResolveReturnsCorrectService
    Expected auth service at localhost:8001, got localhost:8003
--- FAIL: TestCreateUnknownService
    panic: runtime error: invalid memory address or nil pointer dereference
--- FAIL: TestConcurrentRegistration (with -race flag)
    WARNING: DATA RACE
--- FAIL: TestGetHTTPService
    panic: interface conversion: Service is *TCPService, not *HTTPService
```

## Context

The service registry is used in a microservices platform. Services register themselves at startup, and other services look them up by name to discover endpoints. The factory creates service client instances from the registered configurations.

## Your Task

1. Read through `buggy.go` and identify all 4 bugs
2. Fix each bug
3. Verify tests pass: `go test -v`
4. Verify no race conditions: `go test -race`
5. Document your debugging process below

## Files

- `buggy.go` -- The code with 4 bugs
- `buggy_test.go` -- Tests that expose the bugs
- `solution.md` -- Explanation (don't peek until you've tried!)

---

## Your Debugging Process

### Bug 1: All services resolve to same config
**Hypothesis:**

**Root cause:**

**Fix:**

### Bug 2: Nil pointer panic
**Hypothesis:**

**Root cause:**

**Fix:**

### Bug 3: Data race
**Hypothesis:**

**Root cause:**

**Fix:**

### Bug 4: Type assertion panic
**Hypothesis:**

**Root cause:**

**Fix:**

### Lessons



