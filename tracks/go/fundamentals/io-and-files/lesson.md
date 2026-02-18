# I/O & Files — Go

## The Central Insight: Everything is a Reader or Writer

Before diving into specific packages, internalize this: Go's entire I/O system is built on two tiny interfaces.

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}

type Writer interface {
    Write(p []byte) (n int, err error)
}
```

That's it. Files, network connections, HTTP request bodies, in-memory buffers, compressed streams, encrypted pipes — they all implement one or both of these interfaces. The power is that any code written against `io.Reader` works with all of them. A function that reads a CSV doesn't care if the data comes from disk, a test fixture in memory, or a live HTTP response.

This is Go's interface philosophy at full expression: small interfaces compose into everything.

### Compared to JS/TS

In Node.js, the stream abstraction is similarly powerful but more complex:

```typescript
// Node.js streams — event-based, more ceremony
const readable = fs.createReadStream('file.txt');
readable.on('data', (chunk) => { /* ... */ });
readable.on('end', () => { /* done */ });
readable.on('error', (err) => { /* ... */ });

// Or with async iteration (newer style)
for await (const chunk of readable) { /* ... */ }
```

Go's approach is synchronous by default and simpler to reason about:

```go
// Go — synchronous, explicit, zero ceremony
f, err := os.Open("file.txt")
if err != nil {
    return err
}
defer f.Close()

buf := make([]byte, 4096)
for {
    n, err := f.Read(buf)
    if n > 0 {
        process(buf[:n])
    }
    if err == io.EOF {
        break
    }
    if err != nil {
        return err
    }
}
```

The Node model is async-first because JavaScript is single-threaded. Go's goroutine model means blocking I/O is fine — the goroutine sleeps while waiting, and other goroutines run. Simpler mental model, same performance characteristics for I/O-bound work.

### Your notes
<!-- -->

---

## io.Reader in Depth

The `Read` method has a subtle but important contract:

```go
Read(p []byte) (n int, err error)
```

- Reads up to `len(p)` bytes into `p`
- Returns the number of bytes actually read (`n`)
- Returns `io.EOF` when there's nothing left to read
- `n > 0` and `err == io.EOF` can happen simultaneously — the last read

The critical gotcha: **`n` can be less than `len(p)` even without an error.** This is called a *short read*, and it's completely normal. A well-behaved reader might return fewer bytes than the buffer can hold — network packets, OS scheduling, whatever. You cannot assume that a single `Read` call fills the buffer.

```go
// WRONG: assumes Read fills the buffer
buf := make([]byte, 1024)
n, err := r.Read(buf)
data := buf[:n]  // might only have partial data, but err is nil

// CORRECT: use io.ReadFull when you need exactly n bytes
n, err := io.ReadFull(r, buf)
// io.ReadFull returns io.ErrUnexpectedEOF if the reader runs out before filling buf

// ALSO CORRECT: use io.ReadAll when you want everything
data, err := io.ReadAll(r)
// Returns all bytes until EOF, handles partial reads internally
```

### io.ReadAll vs reading manually

```go
// io.ReadAll — simple, loads entire content into memory
data, err := io.ReadAll(r)

// Manual loop — necessary for streaming (don't load everything at once)
buf := make([]byte, 32*1024)  // 32KB chunks
for {
    n, err := r.Read(buf)
    if n > 0 {
        _, werr := w.Write(buf[:n])
        if werr != nil {
            return werr
        }
    }
    if err == io.EOF {
        return nil
    }
    if err != nil {
        return err
    }
}
```

Use `io.ReadAll` for small files or when you need the full content for processing. Use manual streaming for large files, network data, or when memory is constrained.

### io.LimitReader

Cap how much you read from an untrusted source:

```go
// Only read up to 10MB from an HTTP request body
limited := io.LimitReader(r.Body, 10*1024*1024)
data, err := io.ReadAll(limited)
```

Without this, a malicious client can send an infinite body and exhaust your memory. Production HTTP servers always limit body size.

### Your notes
<!-- -->

---

## io.Writer in Depth

```go
Write(p []byte) (n int, err error)
```

- Must write all of `p` or return an error
- If `n < len(p)` without an error, that's a bug in the writer implementation
- Contrast with Reader: writers guarantee full writes; readers allow partial reads

This asymmetry matters when you compose them.

### io.Copy

The idiom for efficiently copying from a reader to a writer:

```go
// io.Copy handles the read loop for you
n, err := io.Copy(dst, src)
// n = total bytes copied
// Uses an internal 32KB buffer by default
```

This is what you'd use to stream a file to an HTTP response, pipe stdin to a file, or copy between any two I/O streams. Don't write the loop manually when `io.Copy` exists.

```go
// Streaming a file to HTTP response
func serveFile(w http.ResponseWriter, path string) error {
    f, err := os.Open(path)
    if err != nil {
        return err
    }
    defer f.Close()

    _, err = io.Copy(w, f)  // w implements io.Writer, f implements io.Reader
    return err
}
```

### Your notes
<!-- -->

---

## Composing Readers and Writers

This is where Go's I/O model becomes genuinely elegant. Because everything speaks the same interface, you can compose behavior by wrapping.

### io.TeeReader

Read from a reader and simultaneously write to a writer. Like a Y-splitter for data:

```go
// Reads from r, writes everything read to w, returns a reader that produces the same data
tee := io.TeeReader(r, w)

// Common use: log what you read (for debugging or audit)
var buf bytes.Buffer
tee := io.TeeReader(httpBody, &buf)

data, err := io.ReadAll(tee)
// data has the response, buf has a copy for logging
fmt.Println("Request body was:", buf.String())
```

Real-world use: reading an API response while simultaneously writing it to a cache or audit log — single pass, no buffering of the full content.

### io.MultiWriter

Fan-out: write to multiple destinations at once.

```go
// Write to two places simultaneously
w := io.MultiWriter(file, os.Stdout)
fmt.Fprintf(w, "This goes to both the file and stdout\n")

// Common use: write log to file AND stdout in dev
logFile, _ := os.Create("app.log")
logger := log.New(io.MultiWriter(logFile, os.Stdout), "", log.LstdFlags)
```

### io.MultiReader

Concatenate multiple readers into one — they're read in sequence:

```go
// Prepend a header to a stream without loading everything into memory
header := strings.NewReader("HEADER\n")
body := strings.NewReader("body content\n")
combined := io.MultiReader(header, body)

data, _ := io.ReadAll(combined)
// "HEADER\nbody content\n"
```

### io.Pipe

A synchronous, in-memory pipe connecting a writer to a reader. What one goroutine writes, another reads:

```go
pr, pw := io.Pipe()

// Writer goroutine
go func() {
    defer pw.Close()
    fmt.Fprintln(pw, "hello through the pipe")
}()

// Reader (current goroutine)
data, err := io.ReadAll(pr)
fmt.Println(string(data))  // "hello through the pipe\n"
```

`io.Pipe` is useful when an API expects a `Reader` but you have a `Writer` — or when you're building a streaming pipeline between goroutines. Note: it's synchronous — the write blocks until the read happens and vice versa.

### Your notes
<!-- -->

---

## The os Package: File Operations

### Opening Files

```go
// os.Open — read-only, most common
f, err := os.Open("config.yaml")

// os.Create — write-only, creates or truncates
f, err := os.Create("output.txt")

// os.OpenFile — full control over flags and permissions
f, err := os.OpenFile("app.log", os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
```

**Common flags for `OpenFile`:**
- `os.O_RDONLY` — read only
- `os.O_WRONLY` — write only
- `os.O_RDWR` — read/write
- `os.O_APPEND` — append to end of file
- `os.O_CREATE` — create if not exists
- `os.O_EXCL` — fail if file exists (used with O_CREATE for atomic creation)
- `os.O_TRUNC` — truncate file on open

Permission `0644` means: owner can read/write, group and others can read. Standard for config files. `0600` for secrets (owner only). `0755` for executables.

### Always defer f.Close()

```go
f, err := os.Open(path)
if err != nil {
    return err
}
defer f.Close()  // ← immediately after successful open
```

The defer goes right after the error check, not at the top of the function. If `Open` fails, there's no file to close.

### The defer f.Close() gotcha for writes

When writing, `f.Close()` can return an error — it flushes OS buffers to disk. But `defer f.Close()` discards the return value:

```go
// WRONG: silently swallows write errors
f, err := os.Create("output.txt")
if err != nil {
    return err
}
defer f.Close()  // if this errors, we never know

_, err = f.Write(data)
return err  // we might think we succeeded
```

```go
// CORRECT: capture Close error explicitly
f, err := os.Create("output.txt")
if err != nil {
    return err
}
defer func() {
    if cerr := f.Close(); cerr != nil && err == nil {
        err = cerr  // only override if no earlier error
    }
}()

_, err = f.Write(data)
return err  // now reflects Close errors too
```

This pattern is verbose but correct for write-critical paths. For reading, discarding the Close error is usually fine.

### Quick file operations

```go
// Read entire file into memory — os.ReadFile (Go 1.16+)
data, err := os.ReadFile("config.yaml")

// Write entire file atomically (well, as atomic as os allows)
err := os.WriteFile("output.txt", data, 0644)

// File info
info, err := os.Stat("file.txt")
fmt.Println(info.Size(), info.ModTime(), info.IsDir())

// Check if file exists
if _, err := os.Stat("file.txt"); os.IsNotExist(err) {
    // file doesn't exist
}
```

### Seeking

`*os.File` implements `io.Seeker`:

```go
// Seek to position
offset, err := f.Seek(100, io.SeekStart)   // 100 bytes from start
offset, err := f.Seek(-10, io.SeekEnd)     // 10 bytes from end
offset, err := f.Seek(50, io.SeekCurrent)  // 50 bytes from current position

// Get current position
pos, err := f.Seek(0, io.SeekCurrent)
```

Seeking works on files, not on all readers (network connections and pipes don't support it).

### Directory Operations

```go
// Create directory
err := os.Mkdir("logs", 0755)
// MkdirAll creates all intermediate dirs (like mkdir -p)
err := os.MkdirAll("logs/2026/02", 0755)

// Remove file or empty directory
err := os.Remove("temp.txt")
// RemoveAll removes directory tree (like rm -rf)
err := os.RemoveAll("tmp/")

// Rename/move
err := os.Rename("old.txt", "new.txt")

// List directory
entries, err := os.ReadDir("./logs")
for _, entry := range entries {
    fmt.Println(entry.Name(), entry.IsDir())
}

// Temporary files
f, err := os.CreateTemp("", "prefix-*.txt")  // "" means os.TempDir()
defer os.Remove(f.Name())                     // cleanup
```

### Environment Variables

```go
// os.Getenv — returns "" if not set (can't distinguish "unset" from "set to empty")
val := os.Getenv("DATABASE_URL")

// os.LookupEnv — returns (value, ok) — ok is false if not set
val, ok := os.LookupEnv("DATABASE_URL")
if !ok {
    // env var is not set (different from being set to "")
}

// Set (useful in tests, not typically in production code)
os.Setenv("KEY", "value")
os.Unsetenv("KEY")

// Get all environment variables
for _, env := range os.Environ() {
    parts := strings.SplitN(env, "=", 2)
    key, value := parts[0], parts[1]
}
```

Use `LookupEnv` when the distinction between "not set" and "set to empty string" matters — for required config vars, you want to fail if they're absent, not silently use empty.

### os.Stdin, os.Stdout, os.Stderr

These are `*os.File` values — they implement both `io.Reader` (Stdin) and `io.Writer` (Stdout, Stderr):

```go
// Reading from stdin
scanner := bufio.NewScanner(os.Stdin)
for scanner.Scan() {
    line := scanner.Text()
}

// Writing to stderr (for errors, logs)
fmt.Fprintln(os.Stderr, "error: something went wrong")

// io.Copy works too
io.Copy(os.Stdout, os.Stdin)  // cat, basically
```

The `fmt.Print*` functions write to Stdout by default. `fmt.Fprint*` functions take an `io.Writer` as the first argument — use them to write to any destination.

### Your notes
<!-- -->

---

## filepath: Path Manipulation

The `path/filepath` package handles path separators correctly across operating systems. On Windows, paths use `\`; on Unix, `/`. Never concatenate paths with string operations.

```go
import "path/filepath"

// Join — safe concatenation with proper separators
p := filepath.Join("logs", "2026", "02", "app.log")
// → "logs/2026/02/app.log" on Unix
// → "logs\2026\02\app.log" on Windows

// vs string concatenation — WRONG on Windows:
// "logs" + "/" + "2026"  ← hardcoded separator, breaks on Windows

// Components
dir  := filepath.Dir("/home/user/app.log")   // "/home/user"
base := filepath.Base("/home/user/app.log")  // "app.log"
ext  := filepath.Ext("/home/user/app.log")   // ".log"

// Stem (filename without extension)
name := strings.TrimSuffix(filepath.Base(path), filepath.Ext(path))

// Absolute path
abs, err := filepath.Abs("./relative/path")

// Clean — resolve . and .. without hitting disk
clean := filepath.Clean("logs/../logs/./app.log")
// → "logs/app.log"

// Check if a path is under another path (important for security!)
rel, err := filepath.Rel("/safe/base", "/safe/base/subdir/file")
// → "subdir/file", nil
rel, err := filepath.Rel("/safe/base", "/other/path")
// → "../other/path", nil — doesn't start with "..", so check manually
```

### filepath.Walk — Tree Traversal

```go
// Walk visits every file and directory under root
err := filepath.Walk("/var/log", func(path string, info os.FileInfo, err error) error {
    if err != nil {
        return err  // couldn't access path
    }

    if info.IsDir() {
        if info.Name() == ".git" {
            return filepath.SkipDir  // skip entire directory
        }
        return nil
    }

    // Process file
    fmt.Println(path, info.Size())
    return nil
})
```

`filepath.WalkDir` (Go 1.16+) is preferred — it avoids calling `os.Lstat` for every entry, making it faster:

```go
err := filepath.WalkDir(".", func(path string, d fs.DirEntry, err error) error {
    if err != nil {
        return err
    }
    if !d.IsDir() && filepath.Ext(path) == ".log" {
        // process log file
    }
    return nil
})
```

### filepath.Glob — Pattern Matching

```go
// Find all .go files in current directory
matches, err := filepath.Glob("*.go")

// filepath.Glob doesn't recurse — use WalkDir for that
```

### Your notes
<!-- -->

---

## bufio: Buffered I/O

Raw `Read` and `Write` calls on files go through the kernel. Each call is a syscall. Reading a 1MB file one byte at a time would make 1,000,000 syscalls — catastrophically slow. `bufio` solves this by batching reads/writes in memory.

### bufio.Reader

```go
// Wrap any io.Reader with buffering
br := bufio.NewReader(f)                    // default 4096-byte buffer
br := bufio.NewReaderSize(f, 64*1024)       // custom 64KB buffer

// ReadString reads until delimiter
line, err := br.ReadString('\n')            // includes the \n
// io.EOF means no more data

// ReadBytes — same but returns []byte
lineBytes, err := br.ReadBytes('\n')

// ReadLine — low-level, use Scanner instead
```

### bufio.Scanner

The idiomatic way to read line by line:

```go
scanner := bufio.NewScanner(f)

// Default: splits on lines
for scanner.Scan() {
    line := scanner.Text()   // string (without newline)
    // or scanner.Bytes()    // []byte (shares internal buffer — copy if you store it)
}
if err := scanner.Err(); err != nil {
    return fmt.Errorf("scanning: %w", err)
}
// Note: scanner.Scan() returns false on both EOF and error
// Always check scanner.Err() after the loop
```

**Common mistake:** not checking `scanner.Err()` after the loop. If the reader errors mid-stream, `scanner.Scan()` returns false — the same as EOF. You can't tell the difference without checking `scanner.Err()`.

### Scanner with custom split functions

```go
// Word-by-word
scanner.Split(bufio.ScanWords)

// Byte-by-byte
scanner.Split(bufio.ScanBytes)

// Rune-by-rune (UTF-8 aware)
scanner.Split(bufio.ScanRunes)

// Custom: split on commas
scanner.Split(func(data []byte, atEOF bool) (advance int, token []byte, err error) {
    for i, b := range data {
        if b == ',' {
            return i + 1, data[:i], nil  // token before comma, advance past comma
        }
    }
    if atEOF && len(data) > 0 {
        return len(data), data, nil  // last token
    }
    return 0, nil, nil  // need more data
})
```

### Scanner token size limit

By default, `bufio.Scanner` has a max token size of `bufio.MaxScanTokenSize` (64KB). If you're scanning lines and some lines might be longer:

```go
scanner := bufio.NewScanner(f)
buf := make([]byte, 512*1024)             // 512KB buffer
scanner.Buffer(buf, 1024*1024)            // up to 1MB tokens
```

### bufio.Writer

```go
bw := bufio.NewWriter(f)
defer bw.Flush()  // ← CRITICAL: don't forget

fmt.Fprintln(bw, "line 1")
fmt.Fprintln(bw, "line 2")
// Nothing written to disk yet — it's in the buffer

bw.Flush()  // Now it writes. Also called by defer above.
```

**The critical rule:** Always flush a `bufio.Writer`. If you forget, your data sits in memory and never reaches the underlying writer. The `defer bw.Flush()` pattern is idiomatic.

Flush can return an error. If you need to capture it:

```go
defer func() {
    if ferr := bw.Flush(); ferr != nil && err == nil {
        err = ferr
    }
}()
```

### bufio.ReadWriter

Both directions buffered:

```go
brw := bufio.NewReadWriter(
    bufio.NewReader(conn),
    bufio.NewWriter(conn),
)
// Used for TCP connections, Unix sockets, etc.
```

### Your notes
<!-- -->

---

## The embed Directive

Go 1.16 introduced `//go:embed` — embed files into the binary at compile time. No more worrying about relative paths, missing files, or deployment artifacts.

```go
import _ "embed"

//go:embed config/defaults.yaml
var defaultConfig []byte

//go:embed templates/email.html
var emailTemplate string

// Use like any other variable
fmt.Println(string(defaultConfig))
```

### Embedding multiple files with embed.FS

```go
import "embed"

//go:embed templates
var templateFS embed.FS

// templateFS contains the entire templates/ directory
data, err := templateFS.ReadFile("templates/email.html")

// Works with fs.WalkDir
fs.WalkDir(templateFS, ".", func(path string, d fs.DirEntry, err error) error {
    // ...
    return nil
})
```

`embed.FS` implements `fs.FS` — the standard file system interface. Anything that accepts `fs.FS` (like `html/template.ParseFS`, `http.FS`) works with embedded files.

### Compared to JS/TS

In TypeScript/Node.js, you'd typically use:
```typescript
// Runtime path resolution — can fail if cwd changes
const config = readFileSync(path.join(__dirname, 'config/defaults.yaml'));

// Or import JSON directly (with special handling for other types)
import config from './config/defaults.json' assert { type: 'json' };
```

Go's embed approach is compile-time: if the file doesn't exist, the build fails. The deployed binary is self-contained — no external files needed.

**When to use embed:**
- Default configuration bundled with the binary
- Static assets (HTML templates, SQL migrations, certificates)
- Data files that change only with new releases
- CLI tools that need bundled resources

**When not to use embed:**
- Configuration that changes per-deployment (use env vars or config files)
- Large binary assets where binary size matters
- Data that needs to be updated without recompiling

### Your notes
<!-- -->

---

## Error Handling in I/O

I/O is where errors are most common. The Go idiom is explicit, but the patterns are consistent.

### io.EOF

`io.EOF` is a sentinel error signaling normal end-of-stream. It's not really an error — it's a signal. Treat it specially:

```go
n, err := r.Read(buf)
if err == io.EOF {
    // Normal termination — done reading
    break
}
if err != nil {
    return fmt.Errorf("reading: %w", err)
}
```

**Important:** `io.EOF` should never be wrapped with `%w`. It's a sentinel value that callers compare with `==`. Wrapping it changes the comparison semantics. Return it unwrapped, or handle it at the point of occurrence.

### Partial reads and io.ErrUnexpectedEOF

```go
// io.ReadFull — read exactly len(buf) bytes or error
n, err := io.ReadFull(r, buf)
// err == nil: exactly len(buf) bytes read
// err == io.EOF: reader empty, zero bytes read
// err == io.ErrUnexpectedEOF: reader empty, but fewer than len(buf) bytes read
```

`io.ErrUnexpectedEOF` is the I/O error for "the stream ended before I expected." It distinguishes "empty reader" (EOF) from "truncated reader" (ErrUnexpectedEOF).

### Cleanup with defer

The canonical cleanup pattern:

```go
func process(path string) (err error) {
    f, err := os.Open(path)
    if err != nil {
        return fmt.Errorf("open %s: %w", path, err)
    }
    defer func() {
        if cerr := f.Close(); cerr != nil && err == nil {
            err = fmt.Errorf("close %s: %w", path, cerr)
        }
    }()

    // ... process ...
    return nil
}
```

Error messages should include the path/resource so callers know where the failure occurred. Wrap with `%w` to preserve the chain for `errors.Is` / `errors.As`.

### Temp files for atomic writes

When updating a file, don't write directly — use a temp file and rename:

```go
func writeAtomic(path string, data []byte) error {
    // Create temp file in the same directory (same filesystem as target)
    dir := filepath.Dir(path)
    tmp, err := os.CreateTemp(dir, ".tmp-*")
    if err != nil {
        return fmt.Errorf("create temp: %w", err)
    }
    tmpName := tmp.Name()
    defer os.Remove(tmpName)  // cleanup if we fail

    if _, err := tmp.Write(data); err != nil {
        tmp.Close()
        return fmt.Errorf("write temp: %w", err)
    }
    if err := tmp.Close(); err != nil {
        return fmt.Errorf("close temp: %w", err)
    }

    // Atomic rename — on most systems, rename is atomic
    if err := os.Rename(tmpName, path); err != nil {
        return fmt.Errorf("rename: %w", err)
    }
    return nil
}
```

Why this matters: if you write directly and crash halfway, you have a corrupted file. With the temp-then-rename approach, the old file is intact until the rename succeeds. `os.Rename` is atomic on Linux/macOS when source and destination are on the same filesystem (which the same-directory temp file guarantees).

### Your notes
<!-- -->

---

## Putting It Together: I/O Pipelines

The real payoff of the `io.Reader`/`io.Writer` abstraction is building pipelines that compose cleanly:

```go
// Count lines in a gzipped log file without decompressing to disk
func countLines(path string) (int, error) {
    f, err := os.Open(path)
    if err != nil {
        return 0, err
    }
    defer f.Close()

    // io.Reader chain: file → gzip decoder → line scanner
    gz, err := gzip.NewReader(f)
    if err != nil {
        return 0, err
    }
    defer gz.Close()

    count := 0
    scanner := bufio.NewScanner(gz)
    for scanner.Scan() {
        count++
    }
    return count, scanner.Err()
}
```

Each layer just wraps the one below with an `io.Reader`. The gzip layer doesn't know it's reading from a file. The scanner doesn't know it's reading decompressed bytes. They all speak the same interface.

This composition pattern is why I/O in Go scales from "read a single file" to "stream 100GB of compressed data through a network connection with encryption" — the building blocks are identical.

> **Related:** [[patterns/decorator-middleware]] — the same composition concept applied to HTTP handlers. I/O pipelines and middleware chains are structurally identical.

### Your notes
<!-- -->
