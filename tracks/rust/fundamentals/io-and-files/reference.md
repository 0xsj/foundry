# Rust Reference — I/O and Files

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Programming Language](https://doc.rust-lang.org/book/),
> and [std library documentation](https://doc.rust-lang.org/std/) for the
> `io-and-files` module. Covers: `std::io` traits, `std::fs`, `std::path`,
> buffered I/O, error types, environment variables, and in-memory I/O.

---

## std::io — Core Traits

Source: [std::io](https://doc.rust-lang.org/std/io/index.html)

### Read

```rust
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    // Provided methods (have default implementations):
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize>;
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize>;
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()>;
    fn take(self, limit: u64) -> Take<Self>;
    fn chain<R: Read>(self, next: R) -> Chain<Self, R>;
    fn bytes(self) -> Bytes<Self>;
}
```

**Contract for `read`:**
- Returns `Ok(n)` where `n` is the number of bytes read into `buf[..n]`
- `Ok(0)` signals end of stream (EOF)
- Does not guarantee reading `buf.len()` bytes in a single call (partial read is valid)
- For guaranteed full-buffer reads, use `read_exact`

**Implementations in stdlib:** `File`, `TcpStream`, `Stdin`, `Cursor<T>`, `&[u8]`, `BufReader<R>`, `GzDecoder`, etc.

### Write

```rust
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    // Provided methods:
    fn write_all(&mut self, buf: &[u8]) -> Result<()>;
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result<()>;
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize>;
    fn by_ref(&mut self) -> &mut Self;
}
```

**Contract for `write`:**
- Returns `Ok(n)` where `n <= buf.len()` bytes were written
- Partial writes are valid — use `write_all` to guarantee the entire buffer is written
- `flush` pushes any buffered data to the underlying sink

**Implementations in stdlib:** `File`, `TcpStream`, `Stdout`, `Stderr`, `Cursor<Vec<u8>>`, `BufWriter<W>`, `Vec<u8>`, etc.

### BufRead

```rust
pub trait BufRead: Read {
    fn fill_buf(&mut self) -> Result<&[u8]>;
    fn consume(&mut self, amt: usize);

    // Provided methods:
    fn lines(self) -> Lines<Self>;
    fn read_line(&mut self, buf: &mut String) -> Result<usize>;
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize>;
    fn split(self, byte: u8) -> Split<Self>;
}
```

`lines()` returns an iterator of `Result<String, io::Error>`. Lines do **not** include the trailing newline character. On Windows, `\r\n` is also stripped.

**Implementations:** `BufReader<R>`, `Cursor<T>`, `StdinLock`

### Seek

```rust
pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;

    // Provided methods:
    fn rewind(&mut self) -> Result<()>;
    fn stream_len(&mut self) -> Result<u64>;
    fn stream_position(&mut self) -> Result<u64>;
}

pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}
```

Returns the resulting byte position from the start of the file.

---

## std::io — Types

### BufReader\<R\>

Source: [std::io::BufReader](https://doc.rust-lang.org/std/io/struct.BufReader.html)

```rust
pub struct BufReader<R: Read> { /* ... */ }

impl<R: Read> BufReader<R> {
    pub fn new(inner: R) -> BufReader<R>;
    pub fn with_capacity(capacity: usize, inner: R) -> BufReader<R>;
    pub fn get_ref(&self) -> &R;
    pub fn get_mut(&mut self) -> &mut R;
    pub fn buffer(&self) -> &[u8];
    pub fn capacity(&self) -> usize;
    pub fn into_inner(self) -> R;
}
```

- Default buffer capacity: 8 KB (`DEFAULT_BUF_SIZE = 8192`)
- Implements `Read`, `BufRead`, and `Seek` (if `R: Seek`)
- `into_inner()` recovers the wrapped reader (useful for reclaiming the `File` after reading)

### BufWriter\<W\>

Source: [std::io::BufWriter](https://doc.rust-lang.org/std/io/struct.BufWriter.html)

```rust
pub struct BufWriter<W: Write> { /* ... */ }

impl<W: Write> BufWriter<W> {
    pub fn new(inner: W) -> BufWriter<W>;
    pub fn with_capacity(capacity: usize, inner: W) -> BufWriter<W>;
    pub fn get_ref(&self) -> &W;
    pub fn get_mut(&mut self) -> &mut W;
    pub fn buffer(&self) -> &[u8];
    pub fn capacity(&self) -> usize;
    pub fn into_inner(self) -> Result<W, IntoInnerError<BufWriter<W>>>;
    pub fn flush(&mut self) -> Result<()>;
}
```

- Default buffer capacity: 8 KB
- **Drop behavior:** flushes the buffer, but any resulting error is **silently discarded**
- `into_inner()` flushes and returns the wrapped writer; returns `IntoInnerError` if flush fails
- Always call `flush()` explicitly before the writer goes out of scope to surface errors

### Cursor\<T\>

Source: [std::io::Cursor](https://doc.rust-lang.org/std/io/struct.Cursor.html)

```rust
pub struct Cursor<T> { /* ... */ }

impl<T: AsRef<[u8]>> Cursor<T> {
    pub fn new(inner: T) -> Cursor<T>;
    pub fn into_inner(self) -> T;
    pub fn get_ref(&self) -> &T;
    pub fn get_mut(&mut self) -> &mut T;
    pub fn position(&self) -> u64;
    pub fn set_position(&mut self, pos: u64);
}
```

Implements `Read`, `Write` (when `T: AsMut<[u8]>`), `Seek`, and `BufRead`.

Common uses:
- `Cursor<&[u8]>` — read from a byte slice
- `Cursor<Vec<u8>>` — in-memory read/write (for tests)
- `Cursor<String>` — read from a String

### io::Error

Source: [std::io::Error](https://doc.rust-lang.org/std/io/struct.Error.html)

```rust
pub struct Error { /* ... */ }

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Error
    where E: Into<Box<dyn error::Error + Send + Sync>>;

    pub fn from_raw_os_error(code: i32) -> Error;
    pub fn last_os_error() -> Error;
    pub fn kind(&self) -> ErrorKind;
    pub fn raw_os_error(&self) -> Option<i32>;
}
```

### io::ErrorKind

Source: [std::io::ErrorKind](https://doc.rust-lang.org/std/io/enum.ErrorKind.html)

```rust
#[non_exhaustive]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    AddrInUse,
    AddrNotAvailable,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    Other,
    // ... (non_exhaustive — new variants may be added)
}
```

`#[non_exhaustive]` means match arms must include a wildcard (`_ => ...`) to be future-proof.

### io::Result

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

Type alias for convenience. All `std::io` functions return `io::Result<T>`.

---

## std::io — Functions

Source: [std::io functions](https://doc.rust-lang.org/std/io/index.html#functions)

```rust
// Copy bytes from reader to writer, returns bytes copied
pub fn copy<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<u64>;

// Discard all bytes from reader (reads to /dev/null)
pub fn sink() -> Sink;

// A reader that always returns empty (EOF immediately)
pub fn empty() -> Empty;

// Write bytes to void
pub fn sink() -> Sink;

// Read bytes from /dev/zero (always returns 0 bytes)
// Not available in std — use repeat(0) instead
pub fn repeat(byte: u8) -> Repeat;
```

---

## std::fs — File System

Source: [std::fs](https://doc.rust-lang.org/std/fs/index.html)

### Convenience Functions

```rust
// Read file contents as String (fails if not valid UTF-8)
pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String>;

// Read file contents as bytes
pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>>;

// Write bytes to file (creates or overwrites)
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()>;

// Copy file from source to destination, returns bytes copied
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64>;

// Rename (move) a file or directory
pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()>;

// Remove a file
pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()>;

// Create a directory (parent must exist)
pub fn create_dir<P: AsRef<Path>>(path: P) -> io::Result<()>;

// Create a directory and all missing parents
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()>;

// Remove an empty directory
pub fn remove_dir<P: AsRef<Path>>(path: P) -> io::Result<()>;

// Remove a directory and all its contents
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()>;

// Get metadata for a path (follows symlinks)
pub fn metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata>;

// Get metadata without following symlinks
pub fn symlink_metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata>;

// List directory entries
pub fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<ReadDir>;
```

### File

Source: [std::fs::File](https://doc.rust-lang.org/std/fs/struct.File.html)

```rust
pub struct File { /* ... */ }

impl File {
    // Open file for reading (file must exist)
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<File>;

    // Create or truncate file for writing
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<File>;

    // Create new file, fail if it exists (O_CREAT | O_EXCL)
    pub fn create_new<P: AsRef<Path>>(path: P) -> io::Result<File>;

    // Get file metadata
    pub fn metadata(&self) -> io::Result<Metadata>;

    // Try to clone the file handle (new fd, same file)
    pub fn try_clone(&self) -> io::Result<File>;

    // Set permissions
    pub fn set_permissions(&self, perm: Permissions) -> io::Result<()>;
}
```

`File` implements: `Read`, `Write`, `Seek`, `Drop` (closes on drop).

### OpenOptions

Source: [std::fs::OpenOptions](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html)

```rust
pub struct OpenOptions { /* ... */ }

impl OpenOptions {
    pub fn new() -> OpenOptions;
    pub fn read(&mut self, read: bool) -> &mut OpenOptions;
    pub fn write(&mut self, write: bool) -> &mut OpenOptions;
    pub fn append(&mut self, append: bool) -> &mut OpenOptions;
    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions;
    pub fn create(&mut self, create: bool) -> &mut OpenOptions;
    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions;
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File>;
}
```

**Option interactions:**
- `write(true)` + `truncate(true)` → truncate to 0 bytes on open
- `write(true)` + `append(true)` → all writes go to end of file
- `create(true)` → create if missing, open if exists
- `create_new(true)` → fail if file already exists

### Metadata

```rust
pub struct Metadata { /* ... */ }

impl Metadata {
    pub fn len(&self) -> u64;            // file size in bytes
    pub fn is_file(&self) -> bool;
    pub fn is_dir(&self) -> bool;
    pub fn is_symlink(&self) -> bool;
    pub fn permissions(&self) -> Permissions;
    pub fn modified(&self) -> io::Result<SystemTime>;
    pub fn accessed(&self) -> io::Result<SystemTime>;
    pub fn created(&self) -> io::Result<SystemTime>;
}
```

---

## std::path — Path Types

Source: [std::path](https://doc.rust-lang.org/std/path/index.html)

### Path

```rust
pub struct Path { /* DST - dynamically sized */ }

impl Path {
    pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> &Path;
    pub fn to_str(&self) -> Option<&str>;
    pub fn to_string_lossy(&self) -> Cow<'_, str>;
    pub fn to_path_buf(&self) -> PathBuf;
    pub fn is_absolute(&self) -> bool;
    pub fn is_relative(&self) -> bool;
    pub fn has_root(&self) -> bool;
    pub fn parent(&self) -> Option<&Path>;
    pub fn file_name(&self) -> Option<&OsStr>;
    pub fn file_stem(&self) -> Option<&OsStr>;
    pub fn extension(&self) -> Option<&OsStr>;
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf;
    pub fn with_extension<S: AsRef<OsStr>>(&self, extension: S) -> PathBuf;
    pub fn with_file_name<S: AsRef<OsStr>>(&self, file_name: S) -> PathBuf;
    pub fn components(&self) -> Components<'_>;
    pub fn exists(&self) -> bool;
    pub fn is_file(&self) -> bool;
    pub fn is_dir(&self) -> bool;
    pub fn is_symlink(&self) -> bool;
    pub fn metadata(&self) -> io::Result<fs::Metadata>;
    pub fn display(&self) -> Display<'_>;
    pub fn starts_with<P: AsRef<Path>>(&self, base: P) -> bool;
    pub fn ends_with<P: AsRef<Path>>(&self, child: P) -> bool;
    pub fn strip_prefix<P: AsRef<Path>>(&self, base: P) -> Result<&Path, StripPrefixError>;
}
```

`Path` is an unsized type (like `str`) — always used behind a reference: `&Path`.

### PathBuf

```rust
pub struct PathBuf { /* ... */ }

impl PathBuf {
    pub fn new() -> PathBuf;
    pub fn with_capacity(capacity: usize) -> PathBuf;
    pub fn as_path(&self) -> &Path;
    pub fn push<P: AsRef<Path>>(&mut self, path: P);
    pub fn pop(&mut self) -> bool;
    pub fn set_file_name<S: AsRef<OsStr>>(&mut self, file_name: S);
    pub fn set_extension<S: AsRef<OsStr>>(&mut self, extension: S) -> bool;
    pub fn into_os_string(self) -> OsString;
    pub fn into_boxed_path(self) -> Box<Path>;
    pub fn capacity(&self) -> usize;
    pub fn clear(&mut self);
    pub fn reserve(&mut self, additional: usize);
    pub fn shrink_to_fit(&mut self);
}
```

`PathBuf` derefs to `Path` — all `Path` methods are available on `PathBuf`.

### AsRef\<Path\>

Most filesystem functions accept `impl AsRef<Path>` rather than `&Path` directly.
The following types implement `AsRef<Path>`:

- `Path`, `PathBuf`
- `str`, `String`
- `OsStr`, `OsString`

This means you can pass `"path/to/file"` directly to `File::open` without conversion.

---

## std::env — Environment Variables

Source: [std::env](https://doc.rust-lang.org/std/env/index.html)

```rust
// Fetch environment variable value as String
pub fn var<K: AsRef<OsStr>>(key: K) -> Result<String, VarError>;

// Fetch environment variable value as OsString (handles non-UTF-8)
pub fn var_os<K: AsRef<OsStr>>(key: K) -> Option<OsString>;

// Iterator over all (key, value) pairs as Strings (panics on non-UTF-8)
pub fn vars() -> Vars;

// Iterator over all (key, value) pairs as OsStrings
pub fn vars_os() -> VarsOs;

// Remove environment variable for the current process
pub fn remove_var<K: AsRef<OsStr>>(key: K);

// Set environment variable for the current process
pub fn set_var<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: V);

// Get current working directory
pub fn current_dir() -> io::Result<PathBuf>;

// Change current working directory
pub fn set_current_dir<P: AsRef<Path>>(path: P) -> io::Result<()>;

// Path to the current executable
pub fn current_exe() -> io::Result<PathBuf>;

// Command line arguments
pub fn args() -> Args;
pub fn args_os() -> ArgsOs;
```

### VarError

```rust
pub enum VarError {
    NotPresent,
    NotUnicode(OsString),
}
```

`var` returns `Err(VarError::NotPresent)` if the variable is unset, `Err(VarError::NotUnicode)` if the value is not valid UTF-8.

---

## std::io — Standard Streams

Source: [std::io streams](https://doc.rust-lang.org/std/io/index.html#standard-io)

```rust
// Returns a handle to stdin
pub fn stdin() -> Stdin;

// Returns a handle to stdout
pub fn stdout() -> Stdout;

// Returns a handle to stderr
pub fn stderr() -> Stderr;
```

### Locking

```rust
impl Stdin {
    pub fn lock(&self) -> StdinLock<'_>;
    pub fn lines(self) -> Lines<StdinLock<'static>>;  // (1.75+)
    pub fn read_line(&self, buf: &mut String) -> io::Result<usize>;
}

impl Stdout {
    pub fn lock(&self) -> StdoutLock<'_>;
}

impl Stderr {
    pub fn lock(&self) -> StderrLock<'_>;
}
```

`StdinLock`, `StdoutLock`, `StderrLock` hold the mutex for the duration of their lifetime.
They implement `Read`/`Write`/`BufRead` as appropriate. Using the locked variants avoids per-call mutex acquisition when writing multiple lines.

---

## Key Macros for I/O

```rust
// Write to a Write implementor (like print! but to any writer)
write!(writer, "fmt {}", value)?;
writeln!(writer, "fmt {}", value)?;

// Print to stdout (locks per call)
print!("...");
println!("...");

// Print to stderr (locks per call)
eprint!("...");
eprintln!("...");
```

---

## io::copy Signature

Source: [std::io::copy](https://doc.rust-lang.org/std/io/fn.copy.html)

```rust
pub fn copy<R: Read + ?Sized, W: Write + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u64>
```

Copies all bytes from `reader` into `writer`. Returns the total bytes transferred.
Uses an 8 KB internal buffer — efficient for large transfers without loading everything into memory.

---

## Summary: When to Use Each Type

| Task | Use |
|---|---|
| Read entire small file | `fs::read_to_string` / `fs::read` |
| Read file line by line | `BufReader::new(file).lines()` |
| Write entire small file | `fs::write` |
| Append to log file | `OpenOptions::new().append(true).open(...)` |
| High-throughput writing | `BufWriter::new(file)` + explicit `flush()` |
| Stream copy | `io::copy(&mut reader, &mut writer)` |
| In-memory I/O (testing) | `Cursor::new(vec![])` |
| Check if file exists | `Path::exists()` or `fs::metadata()` |
| Build paths safely | `PathBuf::from(base).join(name)` |
| Accept path in function | `fn f(path: impl AsRef<Path>)` |
| Environment variables | `env::var("KEY")` / `env::var_os("KEY")` |
| High-volume stdout | `stdout().lock()` — hold lock across writes |
