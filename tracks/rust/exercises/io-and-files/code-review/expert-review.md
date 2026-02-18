# Expert Review: Log Rotation PR

## Critical Issues

### 1. `unwrap()` throughout — panics in production on any I/O error

**Locations:** `write_line`, `rotate`, `rotate_if_needed` (multiple occurrences)

```rust
// Panics if the file can't be opened (permissions, disk full, etc.)
let mut file = OpenOptions::new()
    .append(true)
    .open(&self.log_path)
    .unwrap();

// Panics if archive_dir doesn't exist
fs::rename(&self.log_path, &archive_path).unwrap();
```

**Problem:** Every `unwrap()` is a potential production panic. A disk full condition,
permission error, or missing directory will crash the process. This is especially bad in
a logger — the exact moment the disk fills is when you need error reporting the most.

**Fix:** Return `io::Result<()>` from all functions and propagate with `?`:

```rust
pub fn write_line(&mut self, line: &str) -> io::Result<()> {
    if self.needs_rotation()? {
        self.rotate()?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&self.log_path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}
```

The error type should be `io::Result`, not `Result<(), String>` — `String` error types lose
type information and make it impossible to inspect the `ErrorKind`.

---

### 2. File opened and closed on every `write_line` call — catastrophic performance

**Location:** `RotatingLogger::write_line`

```rust
pub fn write_line(&mut self, line: &str) -> Result<(), String> {
    // ...
    let mut file = OpenOptions::new()     // opens a new fd on every call
        .create(true)
        .append(true)
        .open(&self.log_path)
        .unwrap();
    writeln!(file, "{}", line).unwrap();  // one syscall per write
    // file dropped — fd closed
}
```

**Problem:** Two performance bugs in one:
1. Opening a file is a syscall (`open(2)`) that goes through the OS kernel. Doing it on
   every log write adds thousands of unnecessary syscalls per second for a busy service.
2. No `BufWriter` — every `writeln!` is a direct write syscall instead of buffered.

**Fix:** Hold the file open in the struct, wrap it in `BufWriter`:

```rust
pub struct RotatingLogger {
    writer: BufWriter<File>,
    log_path: PathBuf,
    max_bytes: u64,
    archive_dir: PathBuf,
    bytes_written: u64,  // track size without repeated metadata() calls
}
```

Opening once and reusing the handle reduces open/close syscalls from N (one per write) to 1.
`BufWriter` reduces write syscalls from N to N/8192 (one per 8 KB of output).

---

## Major Concerns

### 3. `String` used for paths — `PathBuf` is the correct type

**Locations:** `RotatingLogger::log_path`, `RotatingLogger::archive_dir`, `rotate_if_needed` signature

```rust
pub struct RotatingLogger {
    log_path: String,      // should be PathBuf
    archive_dir: String,   // should be PathBuf
    // ...
}
```

**Problem:** Using `String` for paths has three issues:
- String concatenation for path joining (`archive_dir + "/" + timestamp`) breaks on Windows
  (which uses `\` as separator) and produces double-slashes if `archive_dir` has a trailing `/`
- Functions should accept `impl AsRef<Path>` (not `&str` or `String`) to be interoperable
  with Path, PathBuf, String, and &str callers
- Storing `String` forces callers to convert `PathBuf` to `String`, which can fail if the
  path contains non-UTF-8 bytes (valid on Linux, common for some filenames)

**Fix:**

```rust
pub struct RotatingLogger {
    log_path: PathBuf,
    archive_dir: PathBuf,
    // ...
}

impl RotatingLogger {
    pub fn new(
        log_path: impl AsRef<Path>,
        max_bytes: u64,
        archive_dir: impl AsRef<Path>,
    ) -> io::Result<RotatingLogger> {
        // ...
    }
}
```

And in `rotate`:

```rust
fn rotate(&mut self) -> io::Result<()> {
    let timestamp = current_timestamp()?;
    let archive_path = self.archive_dir
        .join(format!("{}.log", timestamp));  // PathBuf::join — OS-safe
    // ...
}
```

---

### 4. `rotate_if_needed` duplicates `RotatingLogger` logic

**Location:** `rotate_if_needed` standalone function

**Problem:** `rotate_if_needed` is a free function that reimplements the size-check and
rename logic that exists in `RotatingLogger`. Any bug fix or behavior change must be applied
in two places. The function also panics (`unwrap`) where the struct method doesn't propagate.

**Fix:** Remove `rotate_if_needed` or have it delegate to `RotatingLogger`:

```rust
// Option A: remove entirely — callers use RotatingLogger directly
// Option B: thin wrapper that creates a RotatingLogger and calls write_line
// Option C: extract the rotation logic into a free function that BOTH use
```

The correct choice depends on the caller's needs. If the standalone function is used by
callers that don't want to maintain state, Option C is cleanest.

---

### 5. Hardcoded timestamp in `rotate`

**Location:** `RotatingLogger::rotate`

```rust
let timestamp = "2026-02-18T10-30-00";  // hardcoded — all rotations produce the same name
```

**Problem:** Every rotation overwrites the previous archive. The first rotation creates
`2026-02-18T10-30-00.log`, and the second rotation overwrites it.

**Fix:** Use a real timestamp from `std::time::SystemTime` or a monotonic counter:

```rust
fn rotation_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", ts)
}
```

Or use a counter (simpler, no time dependency):

```rust
pub struct RotatingLogger {
    // ...
    rotation_count: u32,
}

fn rotate(&mut self) -> io::Result<()> {
    self.rotation_count += 1;
    let archive_path = self.archive_dir
        .join(format!("app.{:04}.log", self.rotation_count));
    fs::rename(&self.log_path, &archive_path)?;
    Ok(())
}
```

---

### 6. No `create_dir_all` before rotation — panics if archive dir missing

**Location:** `RotatingLogger::rotate`

```rust
// archive_dir assumed to exist — rename fails if it doesn't
fs::rename(&self.log_path, &archive_path).unwrap();
```

**Problem:** `fs::rename` fails with `NotFound` if `archive_dir` doesn't exist. The
`RotatingLogger::new` constructor could create it eagerly:

```rust
pub fn new(log_path: impl AsRef<Path>, max_bytes: u64, archive_dir: impl AsRef<Path>) -> io::Result<Self> {
    fs::create_dir_all(&archive_dir)?;  // create once at construction time
    // ...
}
```

Or `rotate` creates it lazily:

```rust
fn rotate(&mut self) -> io::Result<()> {
    fs::create_dir_all(&self.archive_dir)?;  // idempotent — safe to call if it exists
    // ...
}
```

---

## Minor Suggestions

### 7. `needs_rotation` calls `fs::metadata` on every write — unnecessary syscall

**Location:** `RotatingLogger::write_line` → `needs_rotation`

Every `write_line` call checks the file size via `fs::metadata` (a syscall). Since the
`RotatingLogger` owns the file handle and does all the writing, it can track bytes written
in memory:

```rust
pub struct RotatingLogger {
    // ...
    bytes_written: u64,
}

fn needs_rotation(&self) -> bool {
    self.bytes_written >= self.max_bytes
}
```

Update `bytes_written` by adding `line.len() + 1` (the `\n`) on each write.

---

### 8. `rotate_if_needed` silently ignores `metadata` errors

**Location:** `rotate_if_needed`

```rust
let size = fs::metadata(log_path)
    .map(|m| m.len())
    .unwrap_or(0);  // treats PermissionDenied the same as NotFound
```

If the file can't be stat'd due to a permissions error, this silently treats it as "no file"
and skips rotation. The `unwrap_or(0)` pattern conflates "file doesn't exist" (expected, size
= 0) with "can't read metadata" (unexpected error). The fix: check the `ErrorKind`:

```rust
let size = match fs::metadata(log_path) {
    Ok(meta) => meta.len(),
    Err(e) if e.kind() == io::ErrorKind::NotFound => 0,
    Err(e) => return Err(e),  // propagate unexpected errors
};
```

---

## Positive Feedback

1. **Good use of `OpenOptions` with `append(true)`.** Append mode is the correct choice for
   log files — it atomically writes to end-of-file without needing to seek, and handles
   concurrent writers correctly (on Linux, appends to O_APPEND files are atomic up to
   `PIPE_BUF` bytes).

2. **`needs_rotation` returns `false` for missing files.** Treating a missing file as
   "no rotation needed" is the right default — the file will be created on the first write.
   This is the correct graceful behavior for a new logger startup.

3. **`rotate` uses `fs::rename` instead of copy+delete.** Rename is atomic on the same
   filesystem — the log entry is never in two places simultaneously, and there's no window
   where both the old and new path are valid. Copy+delete would create a race window.

---

## Summary

| # | Severity | Location | Issue | Concept |
|---|----------|----------|-------|---------|
| 1 | Critical | All functions | `unwrap()` panics on any I/O error | Return `io::Result`, propagate with `?` |
| 2 | Critical | `write_line` | File opened/closed per write, no BufWriter | Hold file open in struct, buffer writes |
| 3 | Major | `RotatingLogger` fields | `String` for paths — wrong type, platform-specific joining | `PathBuf` and `impl AsRef<Path>` |
| 4 | Major | `rotate_if_needed` | Duplicates struct logic, also panics | DRY, single source of truth |
| 5 | Major | `rotate` | Hardcoded timestamp overwrites archives | Use real timestamp or counter |
| 6 | Major | `rotate` | No `create_dir_all` — rename fails on missing archive dir | Create archive dir at construction |
| 7 | Minor | `needs_rotation` | `fs::metadata` syscall on every write | Track bytes written in struct |
| 8 | Minor | `rotate_if_needed` | `unwrap_or(0)` hides permission errors | Check `ErrorKind::NotFound` specifically |

## Related Concepts

- [[fundamentals/io-and-files]] — BufWriter, PathBuf, io::Result, ErrorKind
- [[fundamentals/rust/io-and-files]] — Rust-specific deep dive
- [[pitfalls/rust-bufwriter-silent-flush-failure]] — flush on drop discards errors
- [[pitfalls/rust-unwrap-in-library-code]] — never unwrap in library functions
- [[pitfalls/rust-path-string-concatenation]] — use PathBuf::join
