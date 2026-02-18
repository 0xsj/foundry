# Debugging Solution: Log Processor

## Bug 1 — BufWriter Not Flushed

**Function:** `write_summary`

**Symptom:** File is created but always empty. `Ok(())` is returned. No errors.

**Root cause:** `BufWriter` accumulates writes in an 8 KB memory buffer and sends them
to the underlying `File` only when the buffer fills, `flush()` is called explicitly,
or the `BufWriter` is dropped. For small writes (a few lines), the buffer never fills.
When the function returns, the `BufWriter` is dropped — and **drop calls `flush()` but
discards any resulting error**. If the flush fails (disk full, broken pipe), the data is
lost silently and `Ok(())` is returned to the caller.

**The fix:**

```rust
pub fn write_summary(path: &Path, lines: &[String]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for line in lines {
        writeln!(writer, "{}", line)?;
    }

    writer.flush()?;  // explicit flush — error propagates to caller
    Ok(())
}
```

**Rule:** Always call `flush()` explicitly before a `BufWriter` goes out of scope when
the write must succeed. Never rely on drop to flush.

**Related:** [[pitfalls/rust-bufwriter-silent-flush-failure]]

---

## Bug 2 — File Read Into Memory Instead of Buffered Line Iteration

**Function:** `scan_log_slow`

**Symptom:** Correct results but extremely slow on large files. On a 50 MB log: seconds
instead of milliseconds. On a 10 GB log: process OOMs.

**Root cause:** `file.read_to_string(&mut content)` reads the entire file into a `String`
before processing begins. This is:
- **Memory-inefficient:** allocates O(file size) memory
- **Slow on large files:** waits for the entire file to load
- **Wrong pattern for streaming:** defeats the purpose of I/O traits

**The fix:**

```rust
use std::io::{BufRead, BufReader};

pub fn scan_log_slow(path: &Path, term: &str) -> io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);  // wrap in BufReader

    let count = reader
        .lines()
        .filter_map(|line| line.ok())  // skip unreadable lines
        .filter(|line| line.contains(term))
        .count();

    Ok(count)
}
```

`BufReader::new(file)` wraps the `File` in an 8 KB buffer. Each call to `lines()` reads
up to 8 KB from the OS in one syscall, then serves lines from memory. Memory usage stays
constant at ~8 KB regardless of file size.

**Rule:** Never use `read_to_string` for line-by-line processing of large files. Wrap in
`BufReader` and iterate with `lines()`.

**Related:** [[pitfalls/rust-read-to-string-oom]]

---

## Bug 3 — `unwrap()` in Library Code

**Function:** `open_report_file`

**Symptom:** Function panics when the path doesn't exist, parent directory is missing,
or permissions are denied. The entire process crashes.

**Root cause:** `.unwrap()` on a `Result` panics on `Err`. Library functions must never
panic on expected failure cases. A missing file or permission error is expected; a crash
is not an acceptable response.

**The fix:**

```rust
pub fn open_report_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    // No unwrap — the Result<File, io::Error> propagates directly
}
```

**Rule:** Library code must return `Result` for fallible operations. `unwrap()` and `expect()`
are acceptable in tests, `main()`, and well-documented "this should never fail" scenarios.
They are never acceptable in library functions that callers depend on.

**Related:** [[pitfalls/rust-unwrap-in-library-code]]

---

## Bug 4 — String Concatenation for Paths

**Function:** `build_archive_path`

**Symptom:** On Windows, paths use `\` but the function always produces `/`.
With a trailing slash in `base_dir`, produces double slashes (`//`).

**Root cause:** String concatenation for paths is fragile:
- Hardcodes `/` as separator (breaks on Windows)
- Sensitive to trailing slashes
- No handling of `..`, `.`, or absolute path components
- No type that signals "this is a path, not an arbitrary string"

**The fix:**

```rust
pub fn build_archive_path(base_dir: &str, date: &str) -> PathBuf {
    PathBuf::from(base_dir)
        .join(format!("{}.tar.gz", date))
    // PathBuf::join handles:
    // - OS-native path separators
    // - Trailing slashes in base_dir (normalizes them)
    // - Absolute paths in the joined component (replaces base)
}
```

`PathBuf::join` uses the OS separator, normalizes redundant separators, and handles
edge cases correctly. The return type `PathBuf` signals to callers that this is a
filesystem path, not an arbitrary string.

**Rule:** Never use string concatenation for path construction. Use `PathBuf::from(base).join(name)`.

**Related:** [[pitfalls/rust-path-string-concatenation]]

---

## Summary

| # | Function | Bug | Concept |
|---|----------|-----|---------|
| 1 | `write_summary` | `BufWriter` not flushed — data lost silently | `BufWriter` flush on drop discards errors |
| 2 | `scan_log_slow` | `read_to_string` for line iteration — memory + performance | `BufReader` for streaming line iteration |
| 3 | `open_report_file` | `unwrap()` in library code — panics on expected errors | Library code must return `Result` |
| 4 | `build_archive_path` | String concatenation for paths — fragile, platform-specific | `PathBuf::join` for OS-safe path composition |

## Related Concepts

- [[fundamentals/io-and-files]] — BufReader, BufWriter, flush, PathBuf
- [[pitfalls/rust-bufwriter-silent-flush-failure]]
- [[pitfalls/rust-unwrap-in-library-code]]
