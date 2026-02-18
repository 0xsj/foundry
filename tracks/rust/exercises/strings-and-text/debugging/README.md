# Debugging Exercise: Log Processor

## Scenario

A teammate wrote a batch log-processing utility that reads log lines from a buffer,
normalizes text fields, and produces a summary report. Some functions don't compile, one
panics at runtime on valid data, one silently produces wrong output, and one compiles but
uses patterns that are flagged in code review as bad practice.

## Symptoms

1. **`extract_level` panics at runtime** on certain log lines even though they look
   correctly formatted. The test `test_extract_level_multibyte` fails with a panic.

2. **`accepts_string` does not compile.** The error mentions a type mismatch between
   `&String` and `&str`. The function works, but the signature could be improved.

3. **`count_allocations` produces the correct output but allocates far more than it
   should.** The test passes but a code review flags `.to_string()` calls inside a loop
   where no owned string is needed.

4. **`compare_flag` always returns `false`** even when the argument clearly matches. A
   cross-type comparison is silently failing.

## What to Do

1. Run the failing tests: `rustc --test buggy.rs && ./buggy`
2. Read the error messages and test failures carefully
3. Identify the root cause of each bug — one is a runtime panic, one is a compile error,
   one is a logic bug, one compiles and "works" but has an idiomatic problem
4. Fix each one without changing the test assertions

## Hints

None — try to diagnose from the error messages and test failures first.
