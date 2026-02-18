# Go Reference — Error Handling

> Extracted from [The Go Programming Language Specification](https://go.dev/ref/spec),
> [The `errors` package](https://pkg.go.dev/errors), and
> [The `fmt` package](https://pkg.go.dev/fmt).
> Covers: the error interface, built-in error functions, error wrapping, panic/recover.

---

## The error Interface

Source: [spec#Errors](https://go.dev/ref/spec#Errors)

The predeclared type `error` is defined as:

```go
type error interface {
    Error() string
}
```

It is the conventional interface for representing an error condition, with the zero value `nil` representing no error.

---

## errors Package

Source: [pkg.go.dev/errors](https://pkg.go.dev/errors)

### errors.New

```go
func New(text string) error
```

`New` returns an error that formats as the given text. Each call to `New` returns a distinct error value even if the text is identical.

```go
err1 := errors.New("oops")
err2 := errors.New("oops")
fmt.Println(err1 == err2)  // false
```

### errors.Is

```go
func Is(err, target error) bool
```

`Is` reports whether any error in `err`'s tree matches `target`. An error is considered to match a target if it is equal to that target or if it implements a method `Is(error) bool` such that `Is(target)` returns `true`.

The tree consists of `err` itself, followed by the errors obtained by repeatedly calling `Unwrap`. When `err` wraps multiple errors, `Is` examines `err` followed by a breadth-first traversal of its children.

An error type might provide an `Is` method to override the default `==` match:

```go
// Example: match any error in the same category
type ErrorCode int

func (c ErrorCode) Is(target error) bool {
    t, ok := target.(ErrorCode)
    return ok && c == t
}
```

### errors.As

```go
func As(err error, target any) bool
```

`As` finds the first error in `err`'s tree that matches `target`, and if one is found, sets `target` to that error value and returns `true`. Otherwise returns `false`.

The tree consists of `err` itself, followed by the errors obtained by repeatedly calling `Unwrap`. When `err` wraps multiple errors, `As` examines `err` followed by a breadth-first traversal.

An error matches `target` if the error's concrete value is assignable to the value pointed to by `target`, or if the error has a method `As(interface{}) bool` such that `As(target)` returns `true`.

`target` must be a non-nil pointer to either a type that implements `error`, or to any interface type. `As` panics if target is not such a pointer.

```go
var pathErr *fs.PathError
if errors.As(err, &pathErr) {
    fmt.Println(pathErr.Path)
}
```

### errors.Unwrap

```go
func Unwrap(err error) error
```

`Unwrap` returns the result of calling the `Unwrap` method on `err`, if `err`'s type contains an `Unwrap` method returning `error`. Otherwise, `Unwrap` returns `nil`.

`Unwrap` only calls a method of the form:
```go
Unwrap() error
```

In particular, `Unwrap` does not call the multi-error form `Unwrap() []error`.

### errors.Join (Go 1.20+)

```go
func Join(errs ...error) error
```

`Join` returns an error that wraps the given errors. Any nil error values are discarded. `Join` returns `nil` if every value in `errs` is `nil`. The error formats each wrapped error's message on a separate line.

A non-nil error returned by `Join` implements:
```go
Unwrap() []error
```

```go
err1 := errors.New("field1 is required")
err2 := errors.New("field2 must be positive")

joined := errors.Join(err1, err2)
fmt.Println(joined)
// field1 is required
// field2 must be positive

errors.Is(joined, err1)  // true
errors.Is(joined, err2)  // true
```

---

## fmt Package — Error Formatting

Source: [pkg.go.dev/fmt](https://pkg.go.dev/fmt)

### fmt.Errorf

```go
func Errorf(format string, a ...any) error
```

`Errorf` formats according to a format specifier and returns the string as a value that satisfies `error`.

If the format specifier includes a `%w` verb with an error operand, the returned error will implement an `Unwrap` method returning the operand. If there is more than one `%w` verb, the returned error will implement an `Unwrap` method returning a `[]error` containing all the `%w` operands in the order they appear in the arguments.

It is invalid to use the `%w` verb with an operand that does not implement the `error` interface. The `%w` verb is otherwise a synonym for `%v`.

```go
// Single wrap — Unwrap() error
err := fmt.Errorf("open %s: %w", name, ErrPermission)

// Multiple wrap (Go 1.20+) — Unwrap() []error
err := fmt.Errorf("read %w and %w", err1, err2)
```

### %w vs %v

| Verb | Creates wrapped error | errors.Is works | errors.As works |
|------|--------------------- |-----------------|-----------------|
| `%w` | Yes | Yes | Yes |
| `%v` | No | No | No |

---

## Implementing the error Interface

### Minimum Implementation

```go
type MyError struct {
    Message string
}

func (e *MyError) Error() string {
    return e.Message
}
```

### With Unwrapping Support

```go
type WrappingError struct {
    Context string
    Cause   error
}

func (e *WrappingError) Error() string {
    return fmt.Sprintf("%s: %v", e.Context, e.Cause)
}

// Unwrap makes errors.Is and errors.As traverse this error
func (e *WrappingError) Unwrap() error {
    return e.Cause
}
```

### With Multi-Error Unwrapping (Go 1.20+)

```go
type MultiError struct {
    Errors []error
}

func (e *MultiError) Error() string {
    msgs := make([]string, len(e.Errors))
    for i, err := range e.Errors {
        msgs[i] = err.Error()
    }
    return strings.Join(msgs, "; ")
}

// Multi-error Unwrap — returns []error
func (e *MultiError) Unwrap() []error {
    return e.Errors
}
```

### Custom Is Method

```go
type NotFoundError struct {
    Resource string
    ID       string
}

func (e *NotFoundError) Error() string {
    return fmt.Sprintf("%s %q not found", e.Resource, e.ID)
}

// Is allows matching against ErrNotFound sentinel without knowing the resource/ID
func (e *NotFoundError) Is(target error) bool {
    _, ok := target.(*NotFoundError)
    return ok
}
```

---

## panic and recover

Source: [spec#Handling_panics](https://go.dev/ref/spec#Handling_panics)

### Built-in panic

```go
func panic(v any)
```

The `panic` built-in function stops normal execution of the current goroutine. When a function `F` calls `panic`, normal execution of `F` stops immediately. Any functions whose execution was deferred by `F` are run in the usual way, and then `F` returns to its caller. To the caller `G`, the invocation of `F` then behaves like a call to `panic`, terminating `G`'s execution and running any deferred functions. This continues until all functions in the executing goroutine have stopped, in reverse order. At that point, the program is terminated with a non-zero exit code.

The termination sequence can be controlled by the built-in function `recover`.

### Built-in recover

```go
func recover() any
```

The `recover` built-in function allows a program to manage behavior of a panicking goroutine. Executing a call to `recover` inside a deferred function (but not any function called by it) stops the panicking sequence by restoring normal execution and retrieves the error value passed to the call of `panic`.

If `recover` is called outside the deferred function it does not stop a panicking sequence.

```go
func f() {
    defer func() {
        if r := recover(); r != nil {
            fmt.Println("Recovered:", r)
        }
    }()
    panic("something went wrong")
}
```

### recover Return Value

| Condition | recover() returns |
|---|---|
| Called during panic | The panic value (what was passed to `panic`) |
| Called outside deferred function | `nil` |
| Called during normal execution | `nil` |
| No panic is occurring | `nil` |

---

## Standard Library Sentinel Errors

Commonly used sentinel errors from the standard library:

| Package | Variable | Meaning |
|---------|----------|---------|
| `io` | `io.EOF` | End of input — not an actual error, expected condition |
| `io` | `io.ErrUnexpectedEOF` | EOF in the middle of reading structured data |
| `io` | `io.ErrClosedPipe` | Read/write on closed pipe |
| `os` | `os.ErrNotExist` | File or directory does not exist |
| `os` | `os.ErrPermission` | Permission denied |
| `os` | `os.ErrExist` | File already exists |
| `context` | `context.Canceled` | Context was canceled |
| `context` | `context.DeadlineExceeded` | Context deadline passed |
| `net` | `net.ErrClosed` | Operation on closed network connection |
| `database/sql` | `sql.ErrNoRows` | Query returned no rows |
| `database/sql` | `sql.ErrTxDone` | Transaction already committed or rolled back |

---

## Error Inspection Functions Summary

| Function | Signature | Purpose |
|----------|-----------|---------|
| `errors.New` | `(string) error` | Create a new simple error |
| `errors.Is` | `(err, target error) bool` | Check if error (or chain) matches target |
| `errors.As` | `(err error, target any) bool` | Find first error of given type in chain |
| `errors.Unwrap` | `(err error) error` | Get the wrapped error (one level) |
| `errors.Join` | `(...error) error` | Combine multiple errors into one |
| `fmt.Errorf` | `(string, ...any) error` | Format error with optional wrapping via `%w` |

---

## Error Wrapping Chain Traversal

`errors.Is` and `errors.As` perform tree traversal using the `Unwrap` interface:

```
Single-error unwrap:
err → Unwrap() → err2 → Unwrap() → err3 → Unwrap() → nil

Multi-error unwrap (errors.Join / fmt.Errorf with multiple %w):
err → Unwrap() → []error{err1, err2}
      Breadth-first: checks err1, err2, then their children
```

### Depth of Traversal

There is no depth limit. `errors.Is` and `errors.As` will traverse arbitrarily deep chains. For most programs, chains are 3-5 levels deep. Performance is not a concern in practice.

---

## Conventions

Source: [Go blog: Error handling and Go](https://go.dev/blog/error-handling-and-go), [Effective Go](https://go.dev/doc/effective_go#errors)

### Error String Style

> Error strings should not be capitalized (unless beginning with proper nouns or acronyms) or end with punctuation, since they are usually printed following other context.
>
> That is, use `errors.New("something bad")` not `errors.New("Something bad.")`.
>
> — Effective Go

### Exported Error Variables

Error variables that are part of a public API should be exported (capitalized) so callers can test for them:

```go
// package store

var (
    ErrNotFound   = errors.New("not found")
    ErrConflict   = errors.New("conflict")
    ErrForbidden  = errors.New("forbidden")
)
```

### Error Variable Naming

The convention is `ErrXxx` for package-level sentinel errors and `XxxError` for custom error types:

```go
var ErrTimeout = errors.New("timeout")   // sentinel: ErrXxx

type ValidationError struct { ... }       // type: XxxError
type ParseError struct { ... }
```
