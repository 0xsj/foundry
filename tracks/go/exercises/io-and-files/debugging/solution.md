# Debugging Solution: I/O & Files

## Bug 1 — `writeReport`: wrong defer order — file closed before buffer is flushed

**Location:** `writeReport`, the order of `defer bw.Flush()` and `defer f.Close()`

**Symptom:** File is always empty (0 bytes), function returns nil.

**Root cause:** Go defers execute in **LIFO** (last in, first out) order. In `writeReport`:

```go
defer bw.Flush() // registered first — runs LAST
defer f.Close()  // registered second — runs FIRST
```

`f.Close()` runs first — it closes the file handle. Then `bw.Flush()` attempts to write the buffered data to an already-closed file. The write fails (on a closed fd), the data is discarded, and the error is silently swallowed because `defer bw.Flush()` discards the return value. Result: empty file, no error.

The fix requires two things: correct ordering AND capturing the error.

**Fix:** Always flush explicitly and capture the error:

```go
func writeReport(path string, r Report) (err error) {
    f, err := os.Create(path)
    if err != nil {
        return fmt.Errorf("create %s: %w", path, err)
    }
    defer func() {
        if cerr := f.Close(); cerr != nil && err == nil {
            err = cerr
        }
    }()

    bw := bufio.NewWriter(f)

    fmt.Fprintf(bw, "date:     %s\n", r.Date.Format("2006-01-02"))
    // ... more writes ...

    // Explicit flush — capture and return the error
    if err := bw.Flush(); err != nil {
        return fmt.Errorf("flush: %w", err)
    }
    return nil
}
```

**Why it matters:** `defer bw.Flush()` is tempting but incorrect for write-critical paths. If Flush fails (no disk space, I/O error), you return nil and the caller thinks the write succeeded. Explicit Flush with error checking is required wherever write correctness matters.

**Rule:** `defer bw.Flush()` is acceptable only when you don't care whether the flush succeeds (e.g., writing non-critical debug output to stdout). For file writes, always flush explicitly before the function returns.

---

## Bug 2 — `processLogLines`: missing scanner.Err() check

**Location:** `processLogLines`, the return statement after the scan loop

```go
// BUG: always returns nil
for scanner.Scan() { ... }
return count, nil  // ← should be return count, scanner.Err()
```

**Root cause:** `scanner.Scan()` returns `false` for **two distinct cases**:
1. Normal EOF — no more data, everything was read successfully
2. Mid-stream error — the underlying reader returned an error

In case 2, `scanner.Scan()` returns `false` (like EOF), but `scanner.Err()` returns the error. Without checking `scanner.Err()`, you treat a failed read as a successful EOF.

**Fix:**

```go
for scanner.Scan() {
    count++
}
if err := scanner.Err(); err != nil {
    return count, fmt.Errorf("scan: %w", err)
}
return count, nil
```

**Why it matters:** In production, readers error for real reasons: disk I/O failures, network disconnections, corrupt storage. A function that returns incorrect counts (partial reads) with `nil` error misleads the entire call chain. Callers trust the error return — it must be accurate.

**Mental model:** `scanner.Scan() == false` does NOT mean success. Check `scanner.Err()`. Always.

---

## Bug 3 — `buildReportPath`: hardcoded path separator

**Location:** `buildReportPath`, the string concatenation

```go
// BUG: "/" is Unix-only
return base + "/" + service + "/" + date.Format("2006-01-02") + ".report"
```

**Root cause:** Path separators differ by OS: `/` on Unix/macOS, `\` on Windows. Hardcoding `/` produces invalid paths on Windows that the OS cannot open.

**Fix:**

```go
return filepath.Join(base, service, date.Format("2006-01-02")+".report")
```

`filepath.Join` uses `os.PathSeparator` — the correct separator for the running OS. It also cleans the path (resolves `.`, `..`, double separators).

**Why it matters:** Code that works on Mac CI and breaks in Windows production is a classic portability bug. Go programs often run cross-platform. `filepath.Join` is cheap and correct — there is no reason to use string concatenation for paths.

**Exception:** `path` (without `filepath`) uses `/` always and is appropriate for URLs, embed paths, or `io/fs` paths (which always use `/`). Use `path` for virtual/URL paths, `path/filepath` for real OS paths.

---

## Bug 4 — `processAll`: defer inside a loop

**Location:** `processAll`, `defer f.Close()` inside the `for` range loop

```go
for _, path := range paths {
    f, err := os.Open(path)
    // ...
    defer f.Close()  // BUG: runs when processAll returns, not each iteration
    // ...
}
```

**Root cause:** `defer` registers a call to execute when the **surrounding function** returns. Inside a loop, `defer f.Close()` doesn't run at the end of the loop iteration — it runs when `processAll` itself returns. If processing 1000 files, all 1000 file descriptors remain open until the function exits.

**Fix Option 1:** Extract processing to a helper function (preferred — clear scope):

```go
func processFile(path string) ProcessResult {
    f, err := os.Open(path)
    if err != nil {
        return ProcessResult{Path: path, Err: err}
    }
    defer f.Close() // correct: runs when processFile returns

    n, err := processLogLines(f)
    return ProcessResult{Path: path, Lines: n, Err: err}
}

func processAll(paths []string) []ProcessResult {
    results := make([]ProcessResult, 0, len(paths))
    for _, path := range paths {
        results = append(results, processFile(path))
    }
    return results
}
```

**Fix Option 2:** Explicit close without defer:

```go
for _, path := range paths {
    f, err := os.Open(path)
    if err != nil {
        results = append(results, ProcessResult{Path: path, Err: err})
        continue
    }
    n, err := processLogLines(f)
    f.Close() // explicit — runs each iteration, no defer
    results = append(results, ProcessResult{Path: path, Lines: n, Err: err})
}
```

**Why it matters:** Each open file holds a kernel file descriptor — a finite resource. On Linux, the default limit is 1024 per process. Processing 1025 files with this bug panics or returns an error on file 1025. In a service handling concurrent requests, leaked descriptors compound and cause hard-to-debug "too many open files" errors.

**Rule:** Never use `defer f.Close()` inside a loop. Either extract to a helper function or close explicitly.

---

## Summary

| # | Bug | Location | Root Cause | Fix |
|---|-----|----------|-----------|-----|
| 1 | Flush error silently discarded | `writeReport` | `defer bw.Flush()` discards error | Explicit `bw.Flush()` with error check |
| 2 | Scanner error swallowed | `processLogLines` | Missing `scanner.Err()` after loop | Return `scanner.Err()` |
| 3 | Hardcoded `/` separator | `buildReportPath` | Platform-specific string concat | Use `filepath.Join` |
| 4 | File descriptors leaked | `processAll` | `defer` in loop runs at function exit | Extract helper or explicit `Close()` |

## Related Pitfalls

- [[io-and-files#defer-fclose-on-writes]] — when to capture Close() errors
- [[io-and-files#scanner-err]] — scanner.Scan() is false on both EOF and error
- [[io-and-files#defer-in-loop]] — defer doesn't scope to loop iteration
