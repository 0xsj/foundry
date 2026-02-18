# Debugging: Service Config Loader

## Scenario

A colleague wrote a service configuration loader that reads YAML-like config files, connects to a database, and validates the connection. The code compiles and runs, but the tests are failing in unexpected ways — some errors that should be detectable aren't, and one function panics instead of returning an error.

There are **4 bugs**. Find them.

## Symptoms

1. `errors.Is(err, ErrTimeout)` always returns `false`, even after a known timeout
2. Checking `if err.Error() == "connection refused"` works in isolation but fails in the integration test
3. `ConnectWithRetry` panics instead of returning an error when the DB address is empty
4. The connection pool silently succeeds even when the ping fails — the ping error is checked but never returned

## What You Know

- The config loading pipeline wraps errors at each layer
- `ErrTimeout` is a sentinel that should be detectable with `errors.Is` anywhere in the error chain
- Library code should never panic for bad input — it should return an error
- If `ping()` fails, the connection should be considered failed

## What to Do

1. Read `buggy.go` and the failing tests in `buggy_test.go`
2. Identify all 4 bugs
3. Fix them — the tests should all pass after your fixes
4. Write down your reasoning in comments or notes

Run tests: `go test -v ./...`
