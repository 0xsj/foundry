# Debugging: Pointer Bugs

## Context

You've inherited a small job scheduler codebase. The previous developer knew enough Go to be dangerous — they used pointers, but they hit four classic pointer traps. The test suite is failing. Your job: find each bug, fix it, and understand why it happened.

## Symptoms

Running `go test ./...` from this directory produces:

1. **TestWorkerPoolStats** — panic: runtime error: invalid memory address or nil pointer dereference
2. **TestJobUpdate** — `Count` field shows 0 after `RecordCompletion`, expected non-zero
3. **TestWorkerReferences** — all workers in the slice appear to be the same worker (last one registered)
4. **TestHealthCheckStatus** — health check calls `IsHealthy()` but still reports `healthy=true` despite the handler being configured as unhealthy; OR a panic on nil dereference

## What To Do

1. Run `go test -v .` from this directory to see the failures
2. Read each failing test to understand what behavior is expected
3. Find the bug in `buggy.go` — the comments label which bug is which, but not how to fix it
4. Fix `buggy.go` and verify all tests pass

Do not read `solution.md` until you've attempted to fix each bug.
