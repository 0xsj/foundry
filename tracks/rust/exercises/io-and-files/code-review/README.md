# Code Review Exercise: Log Rotation

## PR Context

**PR Title:** `feat: add log rotation to the metrics collector`

**Author:** A backend engineer new to Rust, coming from Python.

**Background:** The metrics collector writes to a single log file. The author is adding log
rotation: when the current log exceeds a size threshold, rename it to an archive and start a
new log. The PR adds a `rotate_if_needed` function and a `RotatingLogger` struct.

**What to review:**

- Correctness of I/O operations (data loss risks?)
- Error handling quality
- Path handling approach
- Performance characteristics
- RAII / resource management
- Idiomatic Rust I/O patterns

## Instructions

1. Read `proposed.rs`
2. Fill in `my-review.md` with your findings
3. Compare against `expert-review.md` when done

Run the code: `rustc --test proposed.rs && ./proposed`
