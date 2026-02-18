# I/O and Files — Rust

## How I/O Works Under the Hood

### Everything Is a Trait

Rust's I/O system is built on two foundational traits: `Read` and `Write`. Every I/O operation in the standard library — reading files, writing to stdout, sending data over a socket, decompressing a stream — is expressed in terms of these traits.

This is the same design philosophy as Go's `io.Reader` and `io.Writer` interfaces, but with Rust's ownership system layered on top. Where Go uses interfaces (dynamic dispatch, runtime cost), Rust uses generics (static dispatch, monomorphized, zero-cost by default).

```rust
// The core Read trait — simplified
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
}

// The core Write trait — simplified
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
    fn flush(&mut self) -> io::Result<()>;
}
```

`read` fills a buffer and returns how many bytes were written. The contract: a return of `Ok(0)` means end-of-stream. Any other return means some bytes were read — not necessarily all of them. This partial-read behavior is why almost all practical code uses `read_to_end`, `read_to_string`, or wraps in a `BufReader`.

**Comparison to Go:** Go's `io.Reader` has `Read(p []byte) (n int, err error)` — nearly identical. The partial-read contract is the same. Rust makes the mutable borrow explicit in the signature (`buf: &mut [u8]`) whereas Go just passes a slice.

### Your notes
<!-- -->


---

## BufReader and BufWriter

### The Problem with Unbuffered I/O

Every call to `Read::read` or `Write::write` is potentially a syscall. Syscalls are expensive — hundreds of nanoseconds each due to the context switch from user space to kernel space. If you read a 10 MB file one byte at a time, you make ~10 million syscalls.

The fix: buffer. `BufReader<R>` wraps any `R: Read` and maintains an internal buffer (default 8 KB). It fills that buffer in a single `read` call and serves subsequent reads from memory. `BufWriter<W>` wraps any `W: Write` and accumulates writes in memory, flushing to the underlying writer in larger chunks.

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write};

// Without buffering: one syscall per byte (10 million syscalls for 10 MB)
let file = File::open("large.log")?;

// With buffering: one syscall per 8 KB chunk (~1200 syscalls for 10 MB)
let reader = BufReader::new(file);
```

### BufRead — the `lines()` Iterator

`BufRead` is a trait that extends `Read` for buffers. Its most important method is `lines()`, which returns an iterator over lines of text. Each item is an `io::Result<String>` — reading can fail, and the iterator surfaces that.

```rust
use std::fs::File;
use std::io::{BufReader, BufRead};

fn count_errors(path: &str) -> io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let count = reader
        .lines()
        .filter_map(|line| line.ok())           // skip unreadable lines
        .filter(|line| line.contains("ERROR"))
        .count();

    Ok(count)
}
```

Note: `lines()` strips the trailing `\n` (and `\r\n` on Windows). No need to `trim()` manually.

### Why BufWriter Needs flush()

`BufWriter` writes to an internal buffer. That buffer is flushed to the underlying writer when:
1. The buffer is full
2. `flush()` is called explicitly
3. The `BufWriter` is dropped

The third point is the trap. **Flushing on drop silently discards the error.** If the write fails during drop (disk full, broken pipe), the error is swallowed. The correct pattern is explicit `flush()` before the writer goes out of scope:

```rust
use std::fs::File;
use std::io::{BufWriter, Write};

fn write_report(path: &str, content: &str) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(content.as_bytes())?;
    writer.flush()?;  // explicit flush — errors surface here, not silently in drop

    Ok(())
}
```

**Comparison to Go:** Go's `bufio.Writer` has the same requirement. Forgetting `Flush()` is one of the most common Go I/O bugs. Rust makes it slightly more visible through the type system (you see `BufWriter<File>` in the type, not just `*os.File`), but the discipline is the same.

### Your notes
<!-- -->


---

## std::fs — File System Operations

### Reading Files

```rust
use std::fs;
use std::io;

// Read entire file into a String — the "just works" function
fn load_config(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}

// Read into bytes — when UTF-8 validity isn't guaranteed
fn load_binary(path: &str) -> io::Result<Vec<u8>> {
    fs::read(path)
}
```

`fs::read_to_string` fails if the file contains invalid UTF-8. For binary files or unknown encodings, use `fs::read` which returns raw bytes.

### Writing Files

```rust
use std::fs;

// Overwrite (or create) a file atomically
fn save_config(path: &str, content: &str) -> io::Result<()> {
    fs::write(path, content)
}

// Append to an existing file
use std::fs::OpenOptions;
use std::io::Write;

fn append_log(path: &str, entry: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", entry)?;
    Ok(())
}
```

### Directory Operations

```rust
use std::fs;

// Create a directory and all parents — like `mkdir -p`
fs::create_dir_all("data/cache/responses")?;

// Remove a file
fs::remove_file("data/cache/stale.json")?;

// Remove an empty directory
fs::remove_dir("data/cache")?;

// Remove directory and all contents
fs::remove_dir_all("data/cache")?;

// List directory contents
for entry in fs::read_dir("data")? {
    let entry = entry?;
    println!("{}", entry.file_name().to_string_lossy());
}
```

### File Metadata

```rust
use std::fs;

fn log_file_info(path: &str) -> io::Result<()> {
    let meta = fs::metadata(path)?;

    println!("size:     {} bytes", meta.len());
    println!("readonly: {}", meta.permissions().readonly());
    println!("is_file:  {}", meta.is_file());
    println!("is_dir:   {}", meta.is_dir());

    Ok(())
}
```

### Your notes
<!-- -->


---

## File — Open, Create, Read, Write, Seek

`File` is Rust's direct handle to an OS file descriptor. It implements `Read`, `Write`, and `Seek`. You open or create a file using the static methods:

```rust
use std::fs::File;
use std::io::{self, Read, Write, Seek, SeekFrom};

// Open for reading (file must exist)
let file = File::open("config.json")?;

// Create (or truncate) for writing
let file = File::create("output.json")?;

// Full control: OpenOptions
use std::fs::OpenOptions;
let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(false)   // don't clear existing content
    .open("data.bin")?;
```

### Seeking Within a File

```rust
fn read_last_n_bytes(path: &str, n: u64) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;

    // Seek from end
    file.seek(SeekFrom::End(-(n as i64)))?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}
```

`SeekFrom` has three variants: `Start(offset)`, `End(offset)`, `Current(offset)`. Negative offsets from `End` or `Current` are how you seek backwards.

### Your notes
<!-- -->


---

## Path and PathBuf

In Rust, filesystem paths are represented by two types analogous to `str` / `String`:

| Type | Owned? | Analogous to | Use when |
|------|--------|--------------|----------|
| `Path` | No (borrowed) | `str` | Accepting paths as arguments |
| `PathBuf` | Yes | `String` | Storing or building paths |

```rust
use std::path::{Path, PathBuf};

// Path is a borrowed reference — like &str for paths
fn file_exists(path: &Path) -> bool {
    path.exists()
}

// PathBuf is owned — like String for paths
fn build_log_path(base_dir: &str, service: &str, date: &str) -> PathBuf {
    let mut path = PathBuf::from(base_dir);
    path.push(service);
    path.push(format!("{}.log", date));
    path
}
// Result: "logs/auth/2026-02-18.log" — correct separator for the OS
```

### Why Not Just Use &str?

String concatenation for paths breaks on every OS boundary. On Windows, paths use `\`. On Unix, `/`. Path/PathBuf handle separators automatically, and the display/debug output is human-readable.

```rust
// Wrong: string concatenation
let path = base_dir.to_string() + "/" + service + "/" + date + ".log";
// Breaks on Windows, error-prone on Unix

// Right: PathBuf
let path = PathBuf::from(base_dir).join(service).join(format!("{}.log", date));
```

### Common Path Operations

```rust
let path = Path::new("/var/log/app/service.log");

// Components
path.parent()             // Some(Path("/var/log/app"))
path.file_name()          // Some(OsStr("service.log"))
path.file_stem()          // Some(OsStr("service"))
path.extension()          // Some(OsStr("log"))

// Checks
path.exists()             // true/false
path.is_file()            // true/false
path.is_dir()             // true/false

// Display
path.display()            // implements Display — use in format strings
path.to_string_lossy()    // Cow<str> — handles non-UTF-8 names
```

### Converting Between Path and String

```rust
// &Path -> &str (may fail if path contains non-UTF-8 bytes)
let s: &str = path.to_str().unwrap();

// &Path -> String (lossy — replaces invalid bytes with replacement char)
let s: String = path.to_string_lossy().into_owned();

// &str -> PathBuf
let p = PathBuf::from("/var/log");

// String -> PathBuf
let p = PathBuf::from(my_string);
```

**Comparison to Go:** Go's `path/filepath` package has functions like `filepath.Join`, `filepath.Ext`, `filepath.Dir`. Rust bundles these as methods on `Path`/`PathBuf`, which is more ergonomic. Go's paths are just `string` under the hood; Rust's `OsStr` handles platform-specific encoding differences.

### Your notes
<!-- -->


---

## RAII for Resource Cleanup

Rust's ownership system guarantees that files are closed when they go out of scope. There is no `defer file.Close()` (Go), no `try/finally`, no `with` statement (Python). The file handle's `Drop` implementation calls `close(2)` automatically.

```rust
fn process_config(path: &str) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    Ok(content)
    // file is dropped here — close(2) called automatically
    // reader owns file, so dropping reader drops file
}  // <-- file.close() happens here
```

### Controlling Scope With Blocks

If you need to close a file before the end of a function, use an explicit scope block:

```rust
fn update_and_read(path: &str) -> io::Result<String> {
    {
        // Write phase
        let mut file = File::create(path)?;
        writeln!(file, "updated at: {}", "2026-02-18")?;
        // file dropped here — flushed and closed
    }

    // Read phase — file is closed, safe to re-open
    fs::read_to_string(path)
}
```

**Comparison to Go:** Go uses `defer file.Close()` immediately after opening. This works but can obscure error handling — `Close()` on a write might fail, and deferred closes don't surface errors easily. Rust's RAII pattern has the same "auto-close on drop" behavior. The difference is that Rust forces you to think about flush errors explicitly (the drop doesn't panic on flush failure), while Go's `defer` is more explicit in code structure.

### Your notes
<!-- -->


---

## std::io::Error and io::Result

All I/O operations return `io::Result<T>`, which is `Result<T, io::Error>`.

`io::Error` wraps an OS error code plus a kind classification:

```rust
use std::io::{self, ErrorKind};

fn open_or_create(path: &str) -> io::Result<String> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // File doesn't exist — create it with defaults
            fs::write(path, "default config")?;
            Ok(String::from("default config"))
        }
        Err(e) => Err(e),  // propagate other errors
    }
}
```

### Common ErrorKind Values

| `ErrorKind` | OS Error | Meaning |
|---|---|---|
| `NotFound` | ENOENT | File or directory does not exist |
| `PermissionDenied` | EACCES | Insufficient permissions |
| `AlreadyExists` | EEXIST | File already exists (exclusive create) |
| `WouldBlock` | EAGAIN | Operation would block (non-blocking I/O) |
| `InvalidInput` | EINVAL | Invalid argument |
| `UnexpectedEof` | — | Read returned 0 bytes before expected end |
| `BrokenPipe` | EPIPE | Write to a closed pipe/socket |
| `ConnectionRefused` | ECONNREFUSED | TCP connection refused |

### Creating Custom io::Errors

```rust
fn validate_log_path(path: &Path) -> io::Result<()> {
    if path.extension() != Some(std::ffi::OsStr::new("log")) {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("expected .log extension, got: {}", path.display()),
        ));
    }
    Ok(())
}
```

**Comparison to Go:** Go returns `error` interface values from I/O operations. To check the kind, you inspect with `errors.Is(err, fs.ErrNotFound)` or type-assert to `*os.PathError`. Rust's `ErrorKind` enum is more ergonomic — no type assertions, exhaustive matching is possible, and the kind is always available without unwrapping.

### Your notes
<!-- -->


---

## stdin, stdout, stderr

Standard streams in Rust are accessed through functions that return locked or unlocked handles.

```rust
use std::io::{self, BufRead, Write};

fn echo_lines() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    // Lock once for the duration — avoids repeated locking per line
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        writeln!(out, "{}", line)?;
    }
    Ok(())
}
```

### Lock for Performance

`io::stdout()` returns an `Stdout` handle. Every `println!` call internally locks the stdout mutex, writes, and unlocks. For high-throughput output, lock once and hold:

```rust
use std::io::{self, Write};

fn print_records(records: &[String]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();  // lock once

    for record in records {
        writeln!(handle, "{}", record)?;  // no repeated lock/unlock
    }
    // handle (and the lock) dropped here

    Ok(())
}
```

### stderr for diagnostics

```rust
use std::io::{self, Write};

fn report_error(msg: &str) {
    // eprintln! goes to stderr automatically — no lock needed for simple use
    eprintln!("error: {}", msg);

    // Or explicitly:
    let stderr = io::stderr();
    let mut handle = stderr.lock();
    writeln!(handle, "error: {}", msg).ok();  // .ok() — ignore write errors in error handlers
}
```

### Your notes
<!-- -->


---

## Cursor\<Vec\<u8\>\> — Testing I/O Code

When writing functions that accept `impl Read` or `impl Write`, you want to test them without touching the filesystem. `io::Cursor<Vec<u8>>` implements both `Read` and `Write` and operates entirely in memory.

```rust
use std::io::{self, Cursor, BufRead, Write};

// Function that works with any BufRead — testable without files
fn parse_csv_headers(reader: impl BufRead) -> io::Result<Vec<String>> {
    let first_line = reader
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "empty input"))??;

    Ok(first_line.split(',').map(|s| s.trim().to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_headers() {
        let input = b"name, age, email\nalice, 30, alice@example.com\n";
        let cursor = Cursor::new(input);
        let reader = std::io::BufReader::new(cursor);

        let headers = parse_csv_headers(reader).unwrap();
        assert_eq!(headers, vec!["name", "age", "email"]);
    }
}
```

`Cursor` wraps any `AsRef<[u8]>` — you can use `&[u8]`, `Vec<u8>`, or `String` as the underlying data. For captures (reading what was written), use `Cursor<Vec<u8>>` and call `.into_inner()` after writing to inspect the bytes.

### Your notes
<!-- -->


---

## io::copy and Read Chaining

`io::copy` transfers bytes from any `Read` to any `Write`:

```rust
use std::io;
use std::fs::File;

fn copy_file(src: &str, dst: &str) -> io::Result<u64> {
    let mut reader = File::open(src)?;
    let mut writer = File::create(dst)?;
    io::copy(&mut reader, &mut writer)
    // returns number of bytes copied
}
```

### Read Chaining with Take and Chain

`Read::take(n)` limits reading to `n` bytes. `Read::chain(other)` concatenates two readers:

```rust
use std::io::{self, Read};

fn read_bounded(reader: impl Read, max_bytes: u64) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader.take(max_bytes).read_to_end(&mut buf)?;
    Ok(buf)
}

fn read_header_then_body(header: &[u8], body: impl Read) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let header_cursor = io::Cursor::new(header);
    header_cursor.chain(body).read_to_end(&mut buf)?;
    Ok(buf)
}
```

### Your notes
<!-- -->


---

## Environment Variables

```rust
use std::env;

// Read a required environment variable — returns Result
fn get_database_url() -> Result<String, env::VarError> {
    env::var("DATABASE_URL")
}

// Read with a fallback — returns String (never fails)
fn get_port() -> u16 {
    env::var("PORT")
        .unwrap_or_else(|_| String::from("8080"))
        .parse()
        .unwrap_or(8080)
}

// var_os — returns OsString, works with non-UTF-8 values
fn get_path_raw() -> Option<std::ffi::OsString> {
    env::var_os("PATH")
}

// Iterate all environment variables
fn list_env() {
    for (key, val) in env::vars() {
        println!("{key}={val}");
    }
}
```

`env::var` returns `Err(VarError::NotPresent)` if the variable doesn't exist, and `Err(VarError::NotUnicode)` if the value contains non-UTF-8 bytes. For systems programming (or when dealing with paths that may not be valid UTF-8), use `env::var_os` which returns `Option<OsString>`.

### Your notes
<!-- -->


---

## Cross-Language Comparison

### I/O Traits vs Interfaces

| Concept | Rust | Go |
|---|---|---|
| Reader abstraction | `trait Read` | `io.Reader` interface |
| Writer abstraction | `trait Write` | `io.Writer` interface |
| Buffered reader | `BufReader<R>` | `bufio.Reader` |
| Buffered writer | `BufWriter<W>` | `bufio.Writer` |
| Line iterator | `BufRead::lines()` → `Result<String>` | `bufio.Scanner` |
| File handle | `File` | `*os.File` |
| Error handling | `io::Result<T>`, `ErrorKind` | `error` interface, `errors.Is` |
| Resource cleanup | RAII (Drop) | `defer file.Close()` |
| Dispatch | Generic (monomorphized) or `dyn` | Interface (vtable always) |
| In-memory I/O | `Cursor<Vec<u8>>` | `bytes.Buffer` |
| Copy bytes | `io::copy` | `io.Copy` |

### Error Handling Pattern

Go requires explicit error checks after every operation:
```go
file, err := os.Open(path)
if err != nil { return err }
defer file.Close()
```

Rust uses `?` to propagate errors, which desugars to the same pattern but is much less repetitive:
```rust
let file = File::open(path)?;
// file auto-closes on drop — no defer needed
```

The `?` operator works in any function returning `Result` (or `Option`). It is not magic — it calls `.into()` on the error to convert types, then either returns early with `Err` or unwraps to the `Ok` value.

### TypeScript (Node.js) Comparison

Node.js I/O is asynchronous by default (event loop, callbacks/promises). Rust `std::fs` is synchronous — blocking. For async Rust I/O, the `tokio` crate provides `tokio::fs` and `tokio::io` with `async/await` syntax that mirrors the sync API closely. This module covers sync I/O; async I/O is a separate topic.

```typescript
// TypeScript/Node.js — sync reads (for comparison)
import * as fs from 'fs';

const content = fs.readFileSync('config.json', 'utf8');  // string
const bytes = fs.readFileSync('data.bin');               // Buffer

// Error kind check
try {
    fs.readFileSync('missing.txt');
} catch (e) {
    if ((e as NodeJS.ErrnoException).code === 'ENOENT') {
        console.log('file not found');
    }
}
```

```rust
// Rust equivalent
let content = fs::read_to_string("config.json")?;       // String
let bytes = fs::read("data.bin")?;                      // Vec<u8>

// Error kind check
match fs::read_to_string("missing.txt") {
    Ok(s) => println!("{}", s),
    Err(e) if e.kind() == ErrorKind::NotFound => println!("file not found"),
    Err(e) => return Err(e),
}
```

### Your notes
<!-- -->
