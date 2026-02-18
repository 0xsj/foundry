# Iterator / Generator Pattern -- Go Reference

## `iter` Package (Go 1.23+)

**Import:** `import "iter"`

**Source:** https://pkg.go.dev/iter

### Type Definitions

```go
// Seq is an iterator over sequences of individual values.
// When called as range Seq, the iterator function is invoked once
// with a yield function. The iterator must call yield for each
// value in the sequence, stopping early if yield returns false.
type Seq[V any] func(yield func(V) bool)

// Seq2 is an iterator over sequences of pairs of values,
// most commonly key-value pairs.
type Seq2[K, V any] func(yield func(K, V) bool)
```

### `iter.Pull`

```go
func Pull[V any](seq Seq[V]) (next func() (V, bool), stop func())
```

Converts a push-based `Seq` into a pull-based pair of functions. `next` returns the next value and whether it's valid. `stop` must be called when done to release resources.

- `next()` returns `(value, true)` while there are values, then `(zero, false)` when done.
- `stop()` must always be called, typically via `defer stop()`.
- Calling `next` after `stop` returns `(zero, false)`.
- Calling `stop` multiple times is safe.
- Internally creates a goroutine -- failing to call `stop` leaks it.

```go
func Pull2[K, V any](seq Seq2[K, V]) (next func() (K, V, bool), stop func())
```

Same as `Pull` but for `Seq2`.

### Range-Over-Function Rules

A function can be used with `range` if it has one of these signatures:

```go
func(yield func() bool)          // no values (just iteration count)
func(yield func(V) bool)         // single value (iter.Seq[V])
func(yield func(K, V) bool)      // key-value pair (iter.Seq2[K, V])
```

**Behavior:**
- `range` calls the iterator function, passing a `yield` function as the argument
- The iterator calls `yield(v)` for each element
- If `yield` returns `false`, the iterator must stop calling yield and return
- `yield` returns `false` when the loop body executes `break`, `return`, or `goto` out of the loop
- The iterator function runs in the same goroutine as the `range` loop
- Panics in the iterator propagate to the `range` loop
- Calling `yield` after the iterator function returns panics

**Loop variables:**

```go
for v := range seq {           // iter.Seq[V]: v is V
for k, v := range seq2 {      // iter.Seq2[K,V]: k is K, v is V
for range seq {                // discard values, just count iterations
```

---

## `slices` Package Iterator Functions (Go 1.23+)

**Import:** `import "slices"`

**Source:** https://pkg.go.dev/slices

### Conversion Functions

```go
// Values returns an iterator over the slice elements.
func Values[Slice ~[]E, E any](s Slice) iter.Seq[E]

// All returns an iterator over index-element pairs.
func All[Slice ~[]E, E any](s Slice) iter.Seq2[int, E]

// Backward returns an iterator that ranges over a slice backward,
// from s[len(s)-1] to s[0].
func Backward[Slice ~[]E, E any](s Slice) iter.Seq2[int, E]

// Collect collects values from an iterator into a new slice.
func Collect[E any](seq iter.Seq[E]) []E

// AppendSeq appends values from an iterator to an existing slice.
func AppendSeq[Slice ~[]E, E any](s Slice, seq iter.Seq[E]) Slice

// Sorted collects values from an iterator into a new sorted slice.
func Sorted[E cmp.Ordered](seq iter.Seq[E]) []E

// SortedFunc collects values from an iterator into a new slice sorted by f.
func SortedFunc[E any](seq iter.Seq[E], cmp func(E, E) int) []E

// SortedStableFunc is like SortedFunc but uses a stable sort.
func SortedStableFunc[E any](seq iter.Seq[E], cmp func(E, E) int) []E

// Chunk returns an iterator over consecutive sub-slices of up to n elements.
func Chunk[Slice ~[]E, E any](s Slice, n int) iter.Seq[Slice]
```

---

## `maps` Package Iterator Functions (Go 1.23+)

**Import:** `import "maps"`

**Source:** https://pkg.go.dev/maps

```go
// Keys returns an iterator over the keys of m.
func Keys[M ~map[K]V, K comparable, V any](m M) iter.Seq[K]

// Values returns an iterator over the values of m.
func Values[M ~map[K]V, K comparable, V any](m M) iter.Seq[V]

// All returns an iterator over key-value pairs from m.
func All[M ~map[K]V, K comparable, V any](m M) iter.Seq2[K, V]

// Collect collects key-value pairs from an iterator into a new map.
func Collect[K comparable, V any](seq iter.Seq2[K, V]) map[K]V

// Insert adds all key-value pairs from an iterator to m.
func Insert[M ~map[K]V, K comparable, V any](m M, seq iter.Seq2[K, V])
```

---

## `bufio.Scanner` API

**Import:** `import "bufio"`

**Source:** https://pkg.go.dev/bufio#Scanner

### Constructor

```go
func NewScanner(r io.Reader) *Scanner
```

### Methods

```go
// Scan advances to the next token. Returns false when done or on error.
func (s *Scanner) Scan() bool

// Text returns the most recent token as a string.
func (s *Scanner) Text() string

// Bytes returns the most recent token as a byte slice.
// The underlying array may be overwritten on the next Scan.
func (s *Scanner) Bytes() []byte

// Err returns the first non-EOF error encountered by the Scanner.
// If scanning completed normally (reached EOF), Err returns nil.
// IMPORTANT: Always call Err() after the Scan loop.
func (s *Scanner) Err() error

// Split sets the split function for the Scanner.
// Must be called before Scan. Default is ScanLines.
func (s *Scanner) Split(split SplitFunc)

// Buffer sets the initial and max buffer size.
// Default max is 64KB. Call before Scan for larger tokens.
func (s *Scanner) Buffer(buf []byte, max int)
```

### Built-in Split Functions

```go
func ScanLines(data []byte, atEOF bool) (advance int, token []byte, err error)
func ScanWords(data []byte, atEOF bool) (advance int, token []byte, err error)
func ScanBytes(data []byte, atEOF bool) (advance int, token []byte, err error)
func ScanRunes(data []byte, atEOF bool) (advance int, token []byte, err error)
```

### Typical Usage Pattern

```go
scanner := bufio.NewScanner(reader)
for scanner.Scan() {
    line := scanner.Text()
    // process line
}
if err := scanner.Err(); err != nil {
    // handle error -- do NOT ignore this
}
```

---

## `database/sql.Rows` Iteration Pattern

**Import:** `import "database/sql"`

**Source:** https://pkg.go.dev/database/sql#Rows

### Methods

```go
// Next prepares the next result row for reading with Scan.
// Returns false when no more rows or an error occurred.
func (rs *Rows) Next() bool

// Scan copies columns from the current row into dest.
func (rs *Rows) Scan(dest ...any) error

// Close closes the Rows, preventing further enumeration.
// Close is idempotent and safe to call multiple times.
func (rs *Rows) Close() error

// Err returns the error, if any, encountered during iteration.
func (rs *Rows) Err() error

// Columns returns the column names.
func (rs *Rows) Columns() ([]string, error)
```

### Typical Usage Pattern

```go
rows, err := db.QueryContext(ctx, "SELECT id, name FROM users WHERE active = $1", true)
if err != nil {
    return fmt.Errorf("query users: %w", err)
}
defer rows.Close()

for rows.Next() {
    var id int
    var name string
    if err := rows.Scan(&id, &name); err != nil {
        return fmt.Errorf("scan row: %w", err)
    }
    // process id, name
}
if err := rows.Err(); err != nil {
    return fmt.Errorf("iterate rows: %w", err)
}
```

---

## Channel Range Semantics

### Ranging Over a Channel

```go
// range over a channel receives values until the channel is closed
for v := range ch {
    // v is the received value
    // loop exits when ch is closed and drained
}
```

**Rules:**
- `range ch` receives from `ch` until it's closed
- If `ch` is never closed, the loop blocks forever
- If `ch` is nil, the loop blocks forever
- `break` exits the loop but does NOT close the channel
- The sender must close the channel (never the receiver)

### Buffered vs Unbuffered

| Type | Behavior | Use For |
|------|----------|---------|
| `make(chan T)` | Sender blocks until receiver is ready | Synchronization, backpressure |
| `make(chan T, n)` | Sender blocks when buffer is full | Decoupling speed differences |

---

## `filepath.WalkDir` Callback Pattern

**Import:** `import "path/filepath"`

**Source:** https://pkg.go.dev/path/filepath#WalkDir

```go
func WalkDir(root string, fn fs.WalkDirFunc) error

type WalkDirFunc func(path string, d fs.DirEntry, err error) error
```

### Callback Return Values

| Return Value | Effect |
|-------------|--------|
| `nil` | Continue walking |
| `fs.SkipDir` | Skip this directory (if `d.IsDir()`), skip this file otherwise |
| `fs.SkipAll` | Stop walking entirely |
| Any other error | Stop walking, `WalkDir` returns this error |

### Typical Usage

```go
err := filepath.WalkDir("/var/log", func(path string, d fs.DirEntry, err error) error {
    if err != nil {
        return err // propagate permission errors etc.
    }
    if d.IsDir() && d.Name() == ".git" {
        return fs.SkipDir // don't descend into .git
    }
    if !d.IsDir() && filepath.Ext(path) == ".log" {
        processLogFile(path)
    }
    return nil
})
if err != nil {
    log.Fatal(err)
}
```

---

## `encoding/json.Decoder` Streaming Pattern

**Import:** `import "encoding/json"`

```go
func NewDecoder(r io.Reader) *Decoder

// Token reads the next JSON token (delimiters, strings, numbers, etc.)
func (dec *Decoder) Token() (json.Token, error)

// Decode reads the next JSON-encoded value and stores it in v
func (dec *Decoder) Decode(v any) error

// More reports whether there are more elements in the current array/object
func (dec *Decoder) More() bool
```

### Streaming Array Decoding

```go
dec := json.NewDecoder(reader)

// Read opening bracket
if _, err := dec.Token(); err != nil {
    return err
}

// Decode elements one at a time
for dec.More() {
    var item MyStruct
    if err := dec.Decode(&item); err != nil {
        return err
    }
    process(item)
}

// Read closing bracket
if _, err := dec.Token(); err != nil {
    return err
}
```

---

## Quick Reference: Creating Iterators

### From a Slice

```go
seq := slices.Values(mySlice)       // iter.Seq[T]
seq2 := slices.All(mySlice)         // iter.Seq2[int, T]
```

### From a Map

```go
keys := maps.Keys(myMap)            // iter.Seq[K]
vals := maps.Values(myMap)          // iter.Seq[V]
pairs := maps.All(myMap)            // iter.Seq2[K, V]
```

### From a Function

```go
func myIter() iter.Seq[int] {
    return func(yield func(int) bool) {
        for i := 0; i < 10; i++ {
            if !yield(i) { return }
        }
    }
}
```

### Consuming an Iterator

```go
// Into a slice
result := slices.Collect(seq)

// Into a map
result := maps.Collect(seq2)

// With a loop
for v := range seq { /* ... */ }
for k, v := range seq2 { /* ... */ }
```
