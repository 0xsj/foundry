# Testing Fundamentals — Go Reference

Extracted from the official Go documentation: [testing package](https://pkg.go.dev/testing), [go test command](https://pkg.go.dev/cmd/go#hdr-Test_packages), and [Fuzzing documentation](https://go.dev/doc/fuzz/).

---

## Package `testing`

`import "testing"`

Package `testing` provides support for automated testing of Go packages. It is intended to be used in concert with the `go test` command.

---

## Test Functions

### Signature

```go
func TestXxx(t *testing.T)
```

- Function must begin with `Test` followed by a name starting with an uppercase letter (or a non-lowercase character)
- Parameter must be `*testing.T`
- No return value

### `*testing.T` Methods

| Method | Description |
|--------|-------------|
| `t.Error(args ...any)` | Mark test as failed; log message; continue execution |
| `t.Errorf(format string, args ...any)` | Like `Error` with `fmt.Sprintf` formatting |
| `t.Fatal(args ...any)` | Mark test as failed; log message; stop current test immediately (calls `runtime.Goexit`) |
| `t.Fatalf(format string, args ...any)` | Like `Fatal` with formatting |
| `t.Log(args ...any)` | Log a message (only shown on failure or with `-v`) |
| `t.Logf(format string, args ...any)` | Like `Log` with formatting |
| `t.Skip(args ...any)` | Mark test as skipped; stop current test |
| `t.Skipf(format string, args ...any)` | Like `Skip` with formatting |
| `t.SkipNow()` | Mark test as skipped; stop immediately |
| `t.Helper()` | Mark calling function as a test helper; errors reported at caller |
| `t.Parallel()` | Signal this test can run concurrently with other parallel tests |
| `t.Run(name string, f func(*testing.T)) bool` | Run f as a subtest of t named name |
| `t.Cleanup(f func())` | Register function to be called when test and all subtests complete |
| `t.TempDir() string` | Return a temporary directory; cleaned up automatically when test ends |
| `t.Setenv(key, value string)` | Set env var for duration of test; restored after |
| `t.Chdir(dir string)` | Change working directory for duration of test (Go 1.24+) |
| `t.Name() string` | Return name of current test or subtest |
| `t.Failed() bool` | Report whether the test has failed |
| `t.Skipped() bool` | Report whether the test was skipped |
| `t.Short() bool` | Report whether `-short` flag is set |

---

## Benchmark Functions

### Signature

```go
func BenchmarkXxx(b *testing.B)
```

- Function must begin with `Benchmark`
- Parameter must be `*testing.B`
- Must loop `b.N` times

### `*testing.B` Methods

| Method | Description |
|--------|-------------|
| `b.N` | Number of iterations (set by benchmark runner) |
| `b.ResetTimer()` | Reset elapsed time and memory stats; does not affect goroutine setup |
| `b.StartTimer()` | Start timing (called automatically before loop) |
| `b.StopTimer()` | Stop timing |
| `b.ReportAllocs()` | Enable allocation statistics for this benchmark |
| `b.ReportMetric(n float64, unit string)` | Report a custom metric |
| `b.SetBytes(n int64)` | Set number of bytes processed per operation (for MB/s calculation) |
| `b.RunParallel(body func(*testing.PB))` | Run benchmark in parallel |
| `b.Helper()`, `b.Log()`, `b.Error()`, etc. | Same as `*testing.T` |

### Benchmark Output Format

```
BenchmarkTokenBucket-8   10000000   142 ns/op   0 B/op   0 allocs/op
```

| Field | Meaning |
|-------|---------|
| `BenchmarkTokenBucket-8` | Name + GOMAXPROCS |
| `10000000` | Iterations (b.N) |
| `142 ns/op` | Nanoseconds per operation |
| `0 B/op` | Bytes allocated per operation (with -benchmem) |
| `0 allocs/op` | Heap allocations per operation (with -benchmem) |

---

## Fuzz Tests

### Signature

```go
func FuzzXxx(f *testing.F)
```

- Function must begin with `Fuzz`
- Parameter must be `*testing.F`
- Must call `f.Fuzz(func(t *testing.T, ...))`

### `*testing.F` Methods

| Method | Description |
|--------|-------------|
| `f.Add(args ...any)` | Add seed corpus entry |
| `f.Fuzz(ff any)` | Run fuzz function; ff signature: `func(t *testing.T, <corpus types>)` |
| `f.Helper()`, `f.Log()`, `f.Error()`, `f.Fatal()`, `f.Skip()` | Same as `*testing.T` |

### Supported Corpus Types

```
[]byte, string, bool,
int, int8, int16, int32, int64,
uint, uint8, uint16, uint32, uint64,
float32, float64, rune, byte
```

### Fuzz Corpus Directory

```
testdata/fuzz/<FuzzTestName>/<hash>
```

Each file in this directory is a corpus entry — one line per argument (typed). Files are checked in to source control and run as regression tests on every `go test` invocation.

---

## `TestMain`

```go
func TestMain(m *testing.M)
```

If defined, `TestMain` is called instead of running tests directly. It must call `m.Run()` and `os.Exit(m.Run())`.

```go
var testDB *sql.DB

func TestMain(m *testing.M) {
    var err error
    testDB, err = setupTestDB()
    if err != nil {
        log.Fatalf("setup: %v", err)
    }

    code := m.Run()

    testDB.Close()
    os.Exit(code)
}
```

### `*testing.M` Methods

| Method | Description |
|--------|-------------|
| `m.Run() int` | Run the tests, returning an exit code |

---

## `go test` Command Reference

### Synopsis

```
go test [build/test flags] [packages] [binary flags]
```

### Build Flags (selected)

| Flag | Description |
|------|-------------|
| `-race` | Enable data race detection |
| `-cover` | Enable coverage analysis |
| `-coverprofile file` | Write coverage profile to file |
| `-covermode mode` | Coverage mode: `set`, `count`, `atomic` |
| `-tags tag,list` | Build with specified build tags |

### Test Binary Flags

| Flag | Description |
|------|-------------|
| `-v` | Verbose: print each test name as it runs |
| `-run regexp` | Run only tests matching regexp |
| `-bench regexp` | Run benchmarks matching regexp (tests are skipped) |
| `-benchtime d` | Run each benchmark for duration d or N times (e.g., `5s`, `100x`) |
| `-benchmem` | Print memory allocation stats for benchmarks |
| `-count n` | Run each test/benchmark n times (disables caching when n≥1) |
| `-fuzz regexp` | Fuzz the fuzz function matching regexp |
| `-fuzztime d` | Run fuzzing for duration d (default: run forever until failure) |
| `-fuzzcachedir dir` | Directory for generated fuzz corpus |
| `-short` | Run shorter variants of tests; tests check `t.Short()` |
| `-timeout d` | Fail package after duration d (default: 10m) |
| `-parallel n` | Number of parallel test goroutines (default: GOMAXPROCS) |
| `-failfast` | Stop after first test failure |
| `-shuffle off|on|N` | Randomize test execution order (N is seed) |

### Package Patterns

| Pattern | Matches |
|---------|---------|
| `./...` | All packages in current module |
| `./pkg/...` | All packages under ./pkg |
| `github.com/foo/bar` | Specific package |
| `.` | Current package only |

---

## Test File Naming and Package Conventions

### File naming

- Test files must end in `_test.go`
- Only compiled when running `go test`
- Can exist alongside any package

### Package declaration options

```go
// Option 1: Same package (white-box testing — can access unexported identifiers)
package ratelimit

// Option 2: External test package (black-box testing — only exported API)
package ratelimit_test
```

A single package can have both `_test.go` files using both package declarations. This is a common pattern: internal tests for fine-grained unit tests, external tests for API-level integration tests.

---

## `testdata/` Directory

- Any directory named `testdata` is reserved for test support files
- Ignored by the Go build system (not compiled)
- Available to tests at the relative path `testdata/`
- Convention for fixture files, golden files, input samples
- Fuzz corpus saved to `testdata/fuzz/<FuzzTestName>/`

---

## Test Caching

`go test` caches test results. A test result is cached if:
- The test binary inputs (source, flags) haven't changed
- All file reads during the test were declared via `testing.T` or from `testdata/`
- Environment variables accessed were declared via `t.Setenv`

Force re-run with `-count=1`.

---

## `testing/iotest` Package

Provides `io.Reader` and `io.Writer` implementations useful for testing:

| Type/Function | Description |
|---------------|-------------|
| `iotest.ErrReader(err)` | Returns a reader that always returns err |
| `iotest.HalfReader(r)` | Returns a reader that reads half as many bytes as requested |
| `iotest.TimeoutReader(r)` | Returns a reader that returns `ErrTimeout` on second read |
| `iotest.TruncateWriter(w, n)` | Returns a writer that discards after n bytes |
| `iotest.TestReader(t, r, content)` | Test that r correctly implements io.Reader |

---

## `testing/fstest` Package

```go
import "testing/fstest"

// MapFS is an in-memory filesystem
fs := fstest.MapFS{
    "config.json": &fstest.MapFile{Data: []byte(`{"key": "value"}`)},
    "data/file.txt": &fstest.MapFile{Data: []byte("content")},
}
```

Use `fstest.MapFS` to test code that reads from an `fs.FS` without touching the real filesystem.

---

## `testing/slogtest` Package (Go 1.22+)

```go
import "testing/slogtest"

// Test that a custom slog.Handler correctly handles all log records
err := slogtest.TestHandler(handler, func() []map[string]any {
    return results
})
```

---

## Common Patterns

### Golden Files

```go
func TestRenderReport(t *testing.T) {
    got := RenderReport(testData)

    golden := filepath.Join("testdata", t.Name()+".golden")

    if *update {  // go test -update flag
        os.WriteFile(golden, []byte(got), 0644)
    }

    want, _ := os.ReadFile(golden)
    if string(want) != got {
        t.Errorf("output mismatch:\ngot:\n%s\nwant:\n%s", got, want)
    }
}
```

### Using `t.Cleanup` for Teardown

```go
func TestWithServer(t *testing.T) {
    server := startTestServer()
    t.Cleanup(func() {
        server.Close()  // guaranteed to run when test ends
    })

    // ... test against server ...
}
```

Prefer `t.Cleanup` over `defer` in tests — it also runs when subtests complete, and it runs in LIFO order (like defer).

### Using `t.TempDir()`

```go
func TestFileWriter(t *testing.T) {
    dir := t.TempDir()  // unique temp dir, cleaned up automatically
    path := filepath.Join(dir, "output.log")

    err := WriteLog(path, "hello")
    if err != nil {
        t.Fatalf("WriteLog: %v", err)
    }
    // ...
}
```

### Skipping Expensive Tests

```go
func TestWithExternalDB(t *testing.T) {
    if testing.Short() {
        t.Skip("skipping database test in short mode")
    }
    // ... expensive test ...
}
```

Run with `go test -short` to skip these.

---

## Source Links

- [Package testing](https://pkg.go.dev/testing)
- [Package testing/fstest](https://pkg.go.dev/testing/fstest)
- [Package testing/iotest](https://pkg.go.dev/testing/iotest)
- [go test command](https://pkg.go.dev/cmd/go#hdr-Test_packages)
- [Fuzzing in Go](https://go.dev/doc/fuzz/)
- [Go Blog: Table Driven Tests](https://go.dev/wiki/TableDrivenTests)
- [Go Blog: Subtests and Sub-benchmarks](https://go.dev/blog/subtests)
- [Go Blog: Fuzzing](https://go.dev/blog/fuzz-beta)
