# I/O & Files — Go Reference

Extracted from the Go standard library documentation and language specification.

> **Official sources:**
> - [io package](https://pkg.go.dev/io)
> - [os package](https://pkg.go.dev/os)
> - [bufio package](https://pkg.go.dev/bufio)
> - [path/filepath package](https://pkg.go.dev/path/filepath)
> - [embed package](https://pkg.go.dev/embed)
> - [io/fs package](https://pkg.go.dev/io/fs)

---

## io Package

### Core Interfaces

```go
// Reader is the interface that wraps the basic Read method.
// Read reads up to len(p) bytes into p. Returns the number of bytes
// read (0 <= n <= len(p)) and any error encountered. Even if Read
// returns n < len(p), it may use all of p as scratch space during the call.
// If some data is available but not len(p) bytes, Read conventionally
// returns what is available instead of waiting for more.
//
// When Read encounters an error or end-of-file condition after
// successfully reading some data, it returns the number of bytes read.
// It may return the (non-nil) error from the same call or return the
// error (and n == 0) from a subsequent call.
//
// The EOF sentinel: callers should treat a return of 0 and io.EOF
// as indicating that the input stream has ended. The caller is expected
// to call Read again if more data is needed.
type Reader interface {
    Read(p []byte) (n int, err error)
}

// Writer is the interface that wraps the basic Write method.
// Write must return a non-nil error if it returns n < len(p).
// Write must not modify the slice data, even temporarily.
type Writer interface {
    Write(p []byte) (n int, err error)
}

// Closer is the interface that wraps the basic Close method.
type Closer interface {
    Close() error
}

// Seeker is the interface that wraps the basic Seek method.
// Seek sets the offset for the next Read or Write to offset,
// interpreted according to whence:
//   SeekStart   = 0  (relative to the start of the file)
//   SeekCurrent = 1  (relative to the current offset)
//   SeekEnd     = 2  (relative to the end of the file)
type Seeker interface {
    Seek(offset int64, whence int) (int64, error)
}

// Seek whence constants
const (
    SeekStart   = 0 // seek relative to the origin of the file
    SeekCurrent = 1 // seek relative to the current offset
    SeekEnd     = 2 // seek relative to the end
)

// Composed interfaces
type ReadWriter interface {
    Reader
    Writer
}

type ReadCloser interface {
    Reader
    Closer
}

type WriteCloser interface {
    Writer
    Closer
}

type ReadWriteCloser interface {
    Reader
    Writer
    Closer
}

type ReadWriteSeeker interface {
    Reader
    Writer
    Seeker
}
```

### Sentinel Errors

```go
// EOF is the error returned by Read when no more input is available.
// Functions should return EOF only to signal a graceful end of input.
// If the EOF occurs unexpectedly in a structured data stream,
// the appropriate error is either ErrUnexpectedEOF or some other error
// giving more detail.
var EOF = errors.New("EOF")

// ErrUnexpectedEOF means that EOF was encountered in the middle of
// reading a fixed-size block or data structure.
var ErrUnexpectedEOF = errors.New("unexpected EOF")

// ErrClosedPipe is the error used for read or write operations on a closed pipe.
var ErrClosedPipe = errors.New("io: read/write on closed pipe")

// ErrNoProgress is returned by some clients of a Reader when
// many calls to Read have failed to return any data or error,
// usually the sign of a broken Reader implementation.
var ErrNoProgress = errors.New("multiple Read calls return no data or error")
```

### Functions

```go
// Copy copies from src to dst until either EOF is reached on src or
// an error occurs. It returns the number of bytes copied and the first
// error encountered while copying, if any.
// A successful Copy returns err == nil, not err == EOF.
// Copy uses a fixed-size 32KB internal buffer.
func Copy(dst Writer, src Reader) (written int64, err error)

// CopyBuffer is identical to Copy except that it stages through the
// provided buffer (if one is required) rather than allocating a temporary one.
// If buf is nil, one is allocated; if len(buf) is zero, CopyBuffer panics.
func CopyBuffer(dst Writer, src Reader, buf []byte) (written int64, err error)

// CopyN copies n bytes (or until an error) from src to dst.
func CopyN(dst Writer, src Reader, n int64) (written int64, err error)

// ReadAll reads from r until an error or EOF and returns the data it read.
// A successful call returns err == nil, not err == EOF. Because ReadAll
// is defined to read from src until EOF, it does not treat an EOF from
// Read as an error to be reported.
func ReadAll(r Reader) ([]byte, error)

// ReadFull reads exactly len(buf) bytes from r into buf. It returns
// the number of bytes copied and an error if fewer bytes were read.
// The error is EOF only if no bytes were read. If an EOF happens after
// reading some but not all the bytes, ReadFull returns ErrUnexpectedEOF.
func ReadFull(r Reader, buf []byte) (n int, err error)

// ReadAtLeast reads from r into buf until it has read at least min bytes.
// It returns the number of bytes copied and an error if fewer bytes were read.
// The error is EOF only if no bytes were read.
func ReadAtLeast(r Reader, buf []byte, min int) (n int, err error)

// WriteString writes the contents of the string s to w, which accepts a slice
// of bytes. If w implements StringWriter, its WriteString method is invoked directly.
func WriteString(w Writer, s string) (n int, err error)

// LimitReader returns a Reader that reads from r but stops with EOF
// after n bytes. The underlying implementation is a *LimitedReader.
func LimitReader(r Reader, n int64) Reader

// MultiReader returns a Reader that's the logical concatenation of
// the provided input readers.
func MultiReader(readers ...Reader) Reader

// TeeReader returns a Reader that writes to w what it reads from r.
// All reads from r performed through it are matched with
// corresponding writes to w. There is no internal buffering —
// the write must complete before the read completes.
func TeeReader(r Reader, w Writer) Reader

// MultiWriter creates a writer that duplicates its writes to all the
// provided writers, similar to the Unix tee(1) command.
func MultiWriter(writers ...Writer) Writer

// Pipe creates a synchronous in-memory pipe. It can be used to connect
// code expecting an io.Reader with code expecting an io.Writer.
// Reads and Writes on the pipe are matched one to one except when multiple
// Reads are needed to consume a single Write.
func Pipe() (*PipeReader, *PipeWriter)

// Discard is a Writer on which all Write calls succeed without doing anything.
var Discard Writer
```

### NopCloser

```go
// NopCloser returns a ReadCloser with a no-op Close method wrapping the provided Reader.
// If r implements WriterTo, the returned ReadCloser will implement WriterTo
// by forwarding calls to r.
func NopCloser(r Reader) ReadCloser
```

---

## os Package

### File Types

```go
// File represents an open file descriptor.
type File struct { /* unexported fields */ }

// FileInfo describes a file and is returned by Stat and Lstat.
type FileInfo interface {
    Name() string       // base name of the file
    Size() int64        // length in bytes for regular files; system-dependent for others
    Mode() FileMode     // file mode bits
    ModTime() time.Time // modification time
    IsDir() bool        // abbreviation for Mode().IsDir()
    Sys() any           // underlying data source (can return nil)
}

// DirEntry is an entry read from a directory (ReadDir, WalkDir).
type DirEntry interface {
    Name() string               // name of the file/directory within the directory
    IsDir() bool                // abbreviation for Type().IsDir()
    Type() FileMode             // entry's type bits
    Info() (FileInfo, error)    // returns the FileInfo for the entry
}

// FileMode represents a file's mode and permission bits.
// The bits have the same definition on all systems, so that
// information about files can be moved from one system to another portably.
type FileMode uint32
```

### Open and Create

```go
// Open opens the named file for reading.
// If there is an error, it will be of type *PathError.
func Open(name string) (*File, error)

// Create creates or truncates the named file. If the file already exists,
// it is truncated. If the file does not exist, it is created with mode 0666
// (before umask). If successful, methods on the returned File can be used for I/O;
// the associated file descriptor has mode O_RDWR.
func Create(name string) (*File, error)

// OpenFile is the generalized open call; most users will use Open or Create instead.
// It opens the named file with specified flag (O_RDONLY etc.).
// If the file does not exist, and the O_CREATE flag is passed, it is created
// with mode perm (before umask). If successful, methods on the returned File
// can be used for I/O.
func OpenFile(name string, flag int, perm FileMode) (*File, error)

// File open flags (can be OR'd together)
const (
    O_RDONLY int = syscall.O_RDONLY // open the file read-only.
    O_WRONLY int = syscall.O_WRONLY // open the file write-only.
    O_RDWR  int = syscall.O_RDWR   // open the file read-write.
    O_APPEND int = syscall.O_APPEND // append data to the file when writing.
    O_CREATE int = syscall.O_CREAT  // create a new file if none exists.
    O_EXCL  int = syscall.O_EXCL   // used with O_CREATE, file must not exist.
    O_SYNC  int = syscall.O_SYNC   // open for synchronous I/O.
    O_TRUNC int = syscall.O_TRUNC  // truncate regular writable file when opened.
)
```

### File Methods

```go
// Read reads up to len(b) bytes from the File and stores them in b.
// It returns the number of bytes read and any error encountered.
// At end of file, Read returns 0, io.EOF.
func (f *File) Read(b []byte) (n int, err error)

// ReadAt reads len(b) bytes from the File starting at byte offset off.
// It returns the number of bytes read and the error, if any.
// ReadAt always returns a non-nil error when n < len(b).
// At end of file, that error is io.EOF.
func (f *File) ReadAt(b []byte, off int64) (n int, err error)

// Write writes len(b) bytes from b to the File. It returns the number
// of bytes written and an error, if any.
func (f *File) Write(b []byte) (n int, err error)

// WriteAt writes len(b) bytes to the File starting at byte offset off.
func (f *File) WriteAt(b []byte, off int64) (n int, err error)

// WriteString is like Write, but writes the contents of string s
// rather than a slice of bytes.
func (f *File) WriteString(s string) (n int, err error)

// Seek sets the offset for the next Read or Write on file to offset,
// interpreted according to whence.
func (f *File) Seek(offset int64, whence int) (ret int64, err error)

// Close closes the File, rendering it unusable for I/O.
// On files that support SetDeadline, any pending I/O operations
// will be canceled and return immediately with an ErrClosed error.
// Close will return an error if it has already been called.
func (f *File) Close() error

// Name returns the name of the file as presented to Open.
func (f *File) Name() string

// Stat returns the FileInfo structure describing file.
func (f *File) Stat() (FileInfo, error)

// Sync commits the current contents of the file to stable storage.
// Typically, this means flushing the file system's in-memory copy
// of recently written data to disk.
func (f *File) Sync() error
```

### Quick File Operations (Go 1.16+)

```go
// ReadFile reads the named file and returns the contents.
// A successful call returns err == nil, not err == io.EOF.
// Because ReadFile reads the whole file, it does not treat an EOF from Read
// as an error to be reported.
func ReadFile(name string) ([]byte, error)

// WriteFile writes data to the named file, creating it if necessary.
// If the file does not exist, WriteFile creates it with permissions perm
// (before umask); otherwise WriteFile truncates it before writing,
// without changing permissions.
// Since WriteFile requires multiple system calls to complete, a failure
// mid-operation can leave the file in a partially written state.
func WriteFile(name string, data []byte, perm FileMode) error
```

### Directory Operations

```go
// Mkdir creates a new directory with the specified name and permission bits
// (before umask). If there is an error, it will be of type *PathError.
func Mkdir(name string, perm FileMode) error

// MkdirAll creates a directory named path, along with any necessary parents,
// and returns nil, or else returns an error.
// The permission bits perm (before umask) are used for all directories that
// MkdirAll creates. If path is already a directory, MkdirAll does nothing
// and returns nil.
func MkdirAll(path string, perm FileMode) error

// Remove removes the named file or (empty) directory.
func Remove(name string) error

// RemoveAll removes path and any children it contains.
// It removes everything it can but returns the first error it encounters.
// If the path does not exist, RemoveAll returns nil.
func RemoveAll(path string) error

// Rename renames (moves) oldpath to newpath.
// If newpath already exists and is not a directory, Rename replaces it.
// OS-specific restrictions may apply when oldpath and newpath are in
// different directories.
func Rename(oldpath, newpath string) error

// ReadDir reads the named directory, returning all its directory entries
// sorted by filename.
func ReadDir(name string) ([]DirEntry, error)

// CreateTemp creates a new temporary file in the directory dir,
// opens the file for reading and writing, and returns the resulting file.
// The filename is generated by taking pattern and appending a random string to the end.
// If pattern includes a "*", the random string replaces the last "*".
// If dir is the empty string, CreateTemp uses the default directory for temporary files, as returned by TempDir.
func CreateTemp(dir, pattern string) (*File, error)

// MkdirTemp creates a new temporary directory in the directory dir.
// The directory name is generated by taking pattern and applying a random string to the end.
func MkdirTemp(dir, pattern string) (string, error)

// TempDir returns the default directory to use for temporary files.
// On Unix, it returns $TMPDIR if non-empty, else /tmp.
// On Windows, it uses GetTempPath, returning the first non-empty value from
// %TMP%, %TEMP%, %USERPROFILE%, or the Windows directory.
func TempDir() string
```

### File Information

```go
// Stat returns a FileInfo describing the named file.
// If there is an error, it will be of type *PathError.
func Stat(name string) (FileInfo, error)

// Lstat returns a FileInfo describing the named file.
// If the file is a symbolic link, the returned FileInfo describes the
// symbolic link. Lstat makes no attempt to follow the link.
func Lstat(name string) (FileInfo, error)

// IsExist returns a boolean indicating whether the error is known to report
// that a file or directory already exists. It is satisfied by ErrExist as
// well as some syscall errors.
func IsExist(err error) bool

// IsNotExist returns a boolean indicating whether the error is known to
// report that a file or directory does not exist. It is satisfied by
// ErrNotExist as well as some syscall errors.
func IsNotExist(err error) bool

// IsPermission returns a boolean indicating whether the error is known to
// report that permission is denied. It is satisfied by ErrPermission
// as well as some syscall errors.
func IsPermission(err error) bool
```

### Standard I/O and Environment

```go
// Standard file descriptors.
var (
    Stdin  = NewFile(uintptr(syscall.Stdin), "/dev/stdin")
    Stdout = NewFile(uintptr(syscall.Stdout), "/dev/stdout")
    Stderr = NewFile(uintptr(syscall.Stderr), "/dev/stderr")
)

// Getenv retrieves the value of the environment variable named by the key.
// It returns the value, which will be empty if the variable is not present.
// To distinguish between an empty value and an unset value, use LookupEnv.
func Getenv(key string) string

// LookupEnv retrieves the value of the environment variable named by the key.
// If the variable is present in the environment the value (which may be empty)
// is returned and the boolean is true. Otherwise the returned value will be
// empty and the boolean will be false.
func LookupEnv(key string) (string, bool)

// Setenv sets the value of the environment variable named by the key.
// It returns an error, if any.
func Setenv(key, value string) error

// Unsetenv unsets a single environment variable.
func Unsetenv(key string) error

// Environ returns a copy of strings representing the environment,
// in the form "key=value".
func Environ() []string

// ExpandEnv replaces ${var} or $var in the string based on the values of
// the current environment variables.
func ExpandEnv(s string) string

// Exit causes the current program to exit with the given status code.
// Conventionally, code zero indicates success, non-zero an error.
// The program terminates immediately; deferred functions are not run.
func Exit(code int)
```

### Common FileMode Values

```go
const (
    // Single character in long format: d, a, l, p, S, g, c, t, D
    ModeDir        = fs.ModeDir
    ModeAppend     = fs.ModeAppend
    ModeExclusive  = fs.ModeExclusive
    ModeTemporary  = fs.ModeTemporary
    ModeSymlink    = fs.ModeSymlink
    ModeDevice     = fs.ModeDevice
    ModeNamedPipe  = fs.ModeNamedPipe
    ModeSocket     = fs.ModeSocket
    ModeSetuid     = fs.ModeSetuid
    ModeSetgid     = fs.ModeSetgid
    ModeCharDevice = fs.ModeCharDevice
    ModeSticky     = fs.ModeSticky
    ModeIrregular  = fs.ModeIrregular
    ModeType       = fs.ModeType
    ModePerm       = fs.ModePerm // Unix permission bits, 0777
)
// Common permission values:
// 0644 — owner rw, group r, other r (regular files)
// 0600 — owner rw only (secrets, private keys)
// 0755 — owner rwx, group rx, other rx (directories, executables)
// 0700 — owner rwx only (private directories)
```

---

## bufio Package

### Scanner

```go
// NewScanner returns a new Scanner to read from r.
// The split function defaults to ScanLines.
func NewScanner(r io.Reader) *Scanner

// Scanner methods:
func (s *Scanner) Scan() bool           // advances to the next token, returns false on EOF or error
func (s *Scanner) Text() string         // returns the most recent token as a string
func (s *Scanner) Bytes() []byte        // returns the most recent token as a byte slice (may be overwritten)
func (s *Scanner) Err() error           // returns the first non-EOF error encountered
func (s *Scanner) Split(split SplitFunc) // sets the split function; must be called before first Scan
func (s *Scanner) Buffer(buf []byte, max int) // sets the initial buffer and max size

// SplitFunc is the signature of the split function used to tokenize the input.
// The arguments are an initial substring of the remaining unprocessed data and a flag,
// atEOF, that reports whether the Reader has no more data to give.
// The return values are the number of bytes to advance the input and
// the next token to return to the user, if any, plus an error, if any.
type SplitFunc func(data []byte, atEOF bool) (advance int, token []byte, err error)

// Built-in split functions:
var ScanLines  SplitFunc  // default — splits on \n, returns line without \n (\r\n handled)
var ScanWords  SplitFunc  // splits on whitespace sequences, returns words
var ScanRunes  SplitFunc  // splits on UTF-8-encoded Unicode code points
var ScanBytes  SplitFunc  // returns each byte as a token

// Maximum token size for Scanner
const MaxScanTokenSize = 64 * 1024
```

### Reader

```go
// NewReader returns a new Reader whose buffer has at least the specified size.
// If the argument io.Reader is already a Reader with large enough size, it returns the underlying Reader.
func NewReader(rd io.Reader) *Reader
func NewReaderSize(rd io.Reader, size int) *Reader

// Reader methods:
func (b *Reader) Read(p []byte) (n int, err error)
func (b *Reader) ReadByte() (byte, error)
func (b *Reader) UnreadByte() error
func (b *Reader) ReadRune() (r rune, size int, err error)
func (b *Reader) UnreadRune() error
func (b *Reader) ReadLine() (line []byte, isPrefix bool, err error)  // low-level, prefer Scanner
func (b *Reader) ReadString(delim byte) (string, error)              // reads until delim, includes delim
func (b *Reader) ReadBytes(delim byte) ([]byte, error)               // reads until delim, includes delim
func (b *Reader) Peek(n int) ([]byte, error)                         // returns next n bytes without advancing
func (b *Reader) Discard(n int) (discarded int, err error)
func (b *Reader) Buffered() int                                       // number of bytes in buffer
func (b *Reader) Reset(r io.Reader)                                   // resets reader to use new underlying reader
func (b *Reader) Size() int                                           // buffer size
```

### Writer

```go
// NewWriter returns a new Writer whose buffer has at least the specified size.
func NewWriter(w io.Writer) *Writer
func NewWriterSize(w io.Writer, size int) *Writer

// Writer methods:
func (b *Writer) Write(p []byte) (nn int, err error)
func (b *Writer) WriteByte(c byte) error
func (b *Writer) WriteRune(r rune) (size int, err error)
func (b *Writer) WriteString(s string) (int, error)
func (b *Writer) Flush() error          // writes buffered data to the underlying io.Writer
func (b *Writer) Buffered() int         // number of bytes that have been written to buffer
func (b *Writer) Available() int        // bytes available in buffer before next flush
func (b *Writer) AvailableBuffer() []byte  // returns an empty buffer with b.Available() capacity
func (b *Writer) Reset(w io.Writer)     // resets writer to write to w
func (b *Writer) Size() int             // buffer size
```

### ReadWriter

```go
// ReadWriter stores pointers to a Reader and a Writer.
// It implements io.ReadWriter.
type ReadWriter struct {
    *Reader
    *Writer
}

func NewReadWriter(r *Reader, w *Writer) *ReadWriter
```

### Default buffer sizes

```go
// defaultBufSize is the default buffer size for bufio.Reader and bufio.Writer.
// Value: 4096 bytes (4KB)
const defaultBufSize = 4096
```

---

## path/filepath Package

```go
// Join joins any number of path elements into a single path,
// separating them with an OS specific Separator. Empty elements
// are ignored. The result is Cleaned. However, if the argument list
// is empty or all its elements are empty, Join returns an empty string.
func Join(elem ...string) string

// Dir returns all but the last element of path, typically the path's directory.
// After dropping the final element, Dir calls Clean on the path and trailing
// slashes are removed.
func Dir(path string) string

// Base returns the last element of path.
// Trailing slashes are removed before extracting the last element.
// If the path is empty, Base returns ".".
// If the path consists entirely of slashes, Base returns "/".
func Base(path string) string

// Ext returns the file name extension used by path.
// The extension is the suffix beginning at the final dot in the last element of path;
// it is empty if there is no dot.
func Ext(path string) string

// Clean returns the shortest path name equivalent to path by purely lexical processing.
// Applies the following rules iteratively until no further processing can be done:
// 1. Replace multiple Separator elements with a single one.
// 2. Eliminate each . path name element (the current directory).
// 3. Eliminate each inner .. path name element (the parent directory)
//    along with the non-.. element that precedes it.
// 4. Eliminate .. elements that begin a rooted path: "/.." becomes "/" at the beginning.
func Clean(path string) string

// Abs returns an absolute representation of path. If the path is not absolute,
// it will be joined with the current working directory to turn it into an absolute path.
func Abs(path string) (string, error)

// Rel returns a relative path that is lexically equivalent to targpath when
// joined to basepath with an intervening separator.
func Rel(basepath, targpath string) (string, error)

// IsAbs reports whether the path is absolute.
func IsAbs(path string) bool

// ToSlash returns the result of replacing each separator character in path
// with a slash ('/') character.
func ToSlash(path string) string

// FromSlash returns the result of replacing each slash ('/') character
// in path with a separator character.
func FromSlash(path string) string

// Split splits path immediately following the final Separator, separating it
// into a directory and file name component.
func Split(path string) (dir, file string)

// Match reports whether name matches the shell file name pattern.
func Match(pattern, name string) (matched bool, err error)

// Glob returns the names of all files matching pattern or nil if there is no
// matching file. The syntax of patterns is the same as in Match.
func Glob(pattern string) (matches []string, err error)

// Walk walks the file tree rooted at root, calling fn for each file or
// directory in the tree, including root.
// Walk does not follow symbolic links.
// All errors that arise visiting files and directories are filtered by fn.
// If fn returns SkipDir when invoked on a non-directory file,
// Walk skips the remaining files in the containing directory.
func Walk(root string, fn WalkFunc) error

// WalkDir walks the file tree rooted at root, calling fn for each file or
// directory in the tree, including root.
// WalkDir is more efficient than Walk: if fn returns SkipDir when invoked
// on a directory, WalkDir does not call os.Lstat on the directory's contents.
func WalkDir(root string, fn fs.WalkDirFunc) error

// EvalSymlinks returns the path name after the evaluation of any symbolic links.
func EvalSymlinks(path string) (string, error)

// VolumeName returns the leading volume name on Windows.
// On other platforms it returns "".
func VolumeName(path string) string

// Path separator
const Separator     = os.PathSeparator      // '/' on Unix, '\\' on Windows
const ListSeparator = os.PathListSeparator  // ':' on Unix, ';' on Windows

// SkipDir is used as a return value from WalkFuncs to indicate that
// the directory named in the call is to be skipped.
var SkipDir = fs.SkipDir

// SkipAll is used as a return value from WalkFuncs to indicate that
// all remaining files and directories are to be skipped.
var SkipAll = fs.SkipAll

// WalkFunc is the type of the function called by Walk to visit each
// file or directory. The path argument contains the argument to Walk
// as a prefix; that is, if Walk is called with root argument "dir" and
// finds a file named "dir/file.txt", the walk function will be called
// with argument "dir/file.txt".
type WalkFunc func(path string, info fs.FileInfo, err error) error
```

---

## embed Package

```go
// FS is a read-only collection of files, usually initialized with a //go:embed directive.
// When declared without a //go:embed directive, an FS is an empty file system.
type FS struct { /* unexported fields */ }

// Open opens the named file.
// When Open returns an error, it should be of type *PathError with the Op field set to "open",
// the Path field set to name, and the Err field describing the problem.
func (f FS) Open(name string) (fs.File, error)

// ReadFile reads and returns the content of the named file.
func (f FS) ReadFile(name string) ([]byte, error)

// ReadDir reads and returns the entire named directory.
func (f FS) ReadDir(name string) ([]fs.DirEntry, error)

// Embed directives:
//
//go:embed filename
// Embeds a single file. The variable must be of type string, []byte, or embed.FS.
//
//go:embed dir
// Embeds all files under a directory into an embed.FS.
//
//go:embed pattern
// Embeds files matching a glob pattern. All patterns follow the syntax of path.Match.
//
// Rules for embed directives:
// - The directive must appear on the line immediately before the declaration.
// - The embedded file's name must be relative to the package directory.
// - Files beginning with '.' or '_' are not included in directory/glob embeds
//   unless explicitly named.
// - To include hidden files, name them explicitly or use pattern "all:dir".
```

---

## Common Error Types

```go
// PathError records an error and the operation and file path that caused it.
type PathError struct {
    Op   string // the failing operation (e.g., "open", "unlink")
    Path string // the file path
    Err  error  // the reason the operation failed
}

func (e *PathError) Error() string
func (e *PathError) Unwrap() error

// LinkError records an error during a link or symlink or rename system call
// and the paths that caused it.
type LinkError struct {
    Op  string
    Old string
    New string
    Err error
}

// SyscallError records an error from a specific system call.
type SyscallError struct {
    Syscall string
    Err     error
}
```

---

## io/fs Package (Virtual File System)

```go
// FS provides access to a hierarchical file system. The FS interface is the
// minimum implementation required of the file system.
type FS interface {
    Open(name string) (File, error)
}

// Common fs extension interfaces:
type ReadFileFS interface {
    FS
    ReadFile(name string) ([]byte, error)
}

type ReadDirFS interface {
    FS
    ReadDir(name string) ([]DirEntry, error)
}

type GlobFS interface {
    FS
    Glob(pattern string) ([]string, error)
}

type StatFS interface {
    FS
    Stat(name string) (FileInfo, error)
}

type WalkDirFunc func(path string, d DirEntry, err error) error

// WalkDir walks the file tree rooted at root, calling fn for each file or
// directory in the tree, including root.
func WalkDir(fsys FS, root string, fn WalkDirFunc) error

// ReadFile reads the named file from the file system fs and returns its contents.
func ReadFile(fsys FS, name string) ([]byte, error)

// ReadDir reads and returns the entire named directory from the FS.
func ReadDir(fsys FS, name string) ([]DirEntry, error)

// Glob returns the names of all files matching pattern or nil if there is no
// matching file. The syntax of patterns is the same as in path.Match.
func Glob(fsys FS, pattern string) (matches []string, err error)
```
