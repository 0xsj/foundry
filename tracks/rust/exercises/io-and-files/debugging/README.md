# Debugging Exercise: Log Processor

## Scenario

You've inherited a log processing utility. It compiles and mostly runs, but the tests are
failing or producing wrong output in subtle ways. Each function has one bug — some are silent
data loss bugs, some are correctness bugs, and one prevents compilation outright.

## Symptoms

1. **`write_summary`**: The function returns `Ok(())` and creates the file, but the file is
   always empty when you check it afterward. No errors are returned.

2. **`scan_log_slow`**: This function works correctly but takes 10–100x longer than expected
   on large files. A 50 MB log that should take milliseconds takes several seconds.

3. **`open_report_file`**: This function panics in tests and must not `unwrap` in library code.
   The error should propagate to the caller.

4. **`build_archive_path`**: This function produces incorrect paths on some systems — the
   separator is wrong, and joining paths this way is fragile.

## Your Task

Fix each bug. The tests pass once all four bugs are corrected.

Run: `rustc --test buggy.rs && ./buggy`
