# Debugging Exercise: Metric Aggregator

## Scenario

A monitoring service reads metric samples from several sources, aggregates them, and writes a report. The code was written by someone familiar with Python — they used `.unwrap()` liberally, wrote a `From` impl that doesn't quite work, and discarded error context in a `map_err`. There is also a subtle issue with `?` in a function that doesn't have the right return type.

## Symptoms

The code does not compile. There are **4 bugs** — one per function. Read the compiler error for each function carefully. The error messages are highly informative: they tell you what type was expected and what type was found.

## Your Task

Fix all four bugs so that `rustc --test buggy.rs` compiles and all tests pass.

**Rules:**
- Do not change function signatures unless the bug is specifically in the return type.
- Do not change the test cases.
- Each fix should be minimal — correct the root cause, not paper over it.

## Bugs (symptoms only — not the root cause)

1. **Function `lookup_sample`** — The compiler says it cannot find an implementation of a trait needed for `?`. Something about the `From` conversion is missing or wrong.

2. **Function `parse_all_samples`** — The compiler says `?` cannot be used in a function that returns `Vec<f64>`. The fix is not to remove `?`.

3. **Function `compute_average`** — The code compiles and runs, but the test `test_average_preserves_error_info` fails. The error message has been discarded. The fix does not require adding a new dependency.

4. **Function `write_report`** — The test `test_write_report_to_bad_path` panics instead of returning an error. A panic is not the same as returning `Err`.

## Run

```
rustc --test buggy.rs && ./buggy
```
