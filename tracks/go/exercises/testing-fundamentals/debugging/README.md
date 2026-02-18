# Debugging Exercise: Test Code Bugs

## Scenario

A colleague wrote a test suite for a webhook event processor. The code compiles and `go test` reports some failures, some passes, and some flakiness. Your job is to find what's wrong with the **test code** (not the implementation — the production code is correct).

There are **4 bugs**. Each one represents a classic testing antipattern in Go:

1. A test that passes even when the function is broken
2. A parallel subtest that reads a stale loop variable
3. A benchmark whose setup time pollutes the result
4. A test that would fail under the race detector due to unsynchronized shared state

## What You're Given

- `buggy.go` — The webhook processor implementation (correct, do not modify)
- `buggy_test.go` — Test code with 4 bugs (find and fix these)
- `buggy_test.go` compiles and runs without errors, but something is wrong

## Symptoms

- `TestProcessEvent_Formats` always passes, even when the format function returns garbage
- `TestProcessEvent_Parallel` sometimes panics with "index out of range" or produces wrong event names
- `BenchmarkProcessEvent` shows inflated ns/op because it includes setup time
- `TestProcessEvent_Concurrent` fails under `go test -race` with a data race warning

## Your Task

1. Read `buggy_test.go` carefully
2. Identify all 4 bugs — what each one does wrong and why
3. Fix them
4. Verify: `go test -race -v ./...` passes cleanly
5. Verify: `go test -bench=BenchmarkProcessEvent -benchmem ./...` shows reasonable ns/op

Do not look at `solution.md` until you've found at least 3 of the 4 bugs on your own.
