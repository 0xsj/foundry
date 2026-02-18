# Debugging Exercise: Telemetry Aggregator

## Scenario

A telemetry service ingests metric samples from distributed agents — CPU percentages,
memory readings, request counts — and aggregates them into rolling statistics. The code
compiles, but the tests reveal incorrect behavior, panics at runtime, or won't compile
at all.

## Symptoms

1. **The `aggregate` function panics at runtime** on certain inputs even though the
   data looks valid. The test `test_aggregate_safe_access` fails with a panic.

2. **The `apply_multiplier` function doesn't compile.** The error message mentions
   something about borrowing and mutation happening simultaneously.

3. **The `build_index` function silently produces wrong counts.** The test
   `test_build_index_counts` fails with values that are off by one or wildly wrong.

4. **The `top_metrics` function returns results in the wrong type** — the test
   `test_top_metrics_type` doesn't compile at all.

## What to Do

1. Run the failing tests: `rustc --test buggy.rs && ./buggy`
2. Read the error messages carefully
3. Identify the root cause of each bug
4. Fix each one without changing the test assertions

## Hints

None — try to diagnose from the error messages and test failures first.
