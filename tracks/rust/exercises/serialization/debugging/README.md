# Debugging Exercise: Config Loader

## Scenario

A colleague wrote a configuration loader that reads JSON config files and populates
typed structs. It was working in their local test, but several bugs were introduced
during a "quick cleanup" refactor. The tests are now failing in 4 different places.

## Symptoms

Running `rustc --test buggy.rs && ./buggy` produces:

1. **`test_missing_retry_count_uses_default` — test panics** instead of using the default.
   The function is supposed to handle missing optional fields gracefully.

2. **`test_deserialize_log_level` — assertion fails**: the parsed `log_level` doesn't match
   the expected value even though the JSON looks correct.

3. **`test_nested_optional_description` — wrong type behavior**: a field that should be
   `Option<Option<String>>` is not parsing correctly, causing unexpected results.

4. **`test_timestamp_parsing` — parse error**: a datetime string that should parse correctly
   is returning an error. The format in the code doesn't match the format in the test data.

## Your Task

Find and fix all 4 bugs. Run the tests after each fix to confirm it passes.

```
rustc --test buggy.rs && ./buggy
```

Do not look at `solution.md` until you've found all 4 bugs on your own.
