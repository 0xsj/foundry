# Debugging: Text Processing Bugs

## Context

Your team inherited a text-processing utility used by a log aggregation pipeline. The utility handles user-facing display name normalization, metric label building, and log filtering. It passes code review — no obvious syntax errors — but exhibits intermittent failures and unexpectedly slow performance under load.

The test suite documents the expected behavior. Your job is to find and fix the bugs.

## Symptoms

1. **Display names are corrupted for non-ASCII users** — names like "Ångström" or "José" come back with the wrong first character. Reported by the i18n team.

2. **Label building is slow for large metric sets** — takes 12 seconds for 10,000 labels. Expected to run in under 50ms.

3. **Log filtering is slower than expected** — profiling shows most time is spent in a regexp function. The filter itself is correct, but it's doing something expensive on every call.

4. **A function claiming to work with `[]byte` is making an extra copy** — the memory profiler shows an unexpected allocation when converting between `[]byte` and `string` in a tight loop.

## How to Approach

1. Read each function and its test case
2. Form a hypothesis about the bug before looking at test output
3. Run `go test ./...` to see failing tests
4. Fix the bug — the goal is minimal, targeted changes
5. Verify with `go test ./...` — all tests should pass

The bugs are real patterns you'll encounter in production Go code, not contrived tricks.
