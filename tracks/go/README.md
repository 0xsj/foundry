# Go Track

Primary language. Go's simplicity means patterns are expressed with less abstraction — interfaces are implicit, concurrency is built-in, and error handling is explicit. The goal here is idiomatic Go, not Java-in-Go.

## Key Language Characteristics

- Static typing with type inference
- Implicit interface satisfaction
- CSP concurrency (goroutines + channels)
- Explicit error handling (no exceptions)
- Composition over inheritance (embedding)
- Fast compilation, single binary output
- Opinionated formatting (gofmt)

## What To Pay Attention To

- **Interfaces should be small.** One or two methods. Defined by the consumer, not the implementer.
- **Error handling is a feature, not a burden.** Wrapping, sentinel errors, custom types — learn all three.
- **Goroutine lifecycle management.** Always know how a goroutine ends. Context propagation is critical.
- **Package design matters.** Avoid circular dependencies. Think about your public API surface.
- **Zero values are useful.** Design structs so their zero value is valid.

## Directory Structure

```
go/
├── fundamentals/    # Language basics exercises
├── patterns/        # Design pattern implementations
├── exercises/       # Generated exercises
└── builds/          # Larger architecture projects
```

## Tools & Setup

- `go mod init foundry/tracks/go` at the track root
- Use `go test ./...` to run all tests
- `golangci-lint` for linting
- Standard library first — reach for dependencies only when justified

## Idiomatic Conventions

- Accept interfaces, return structs
- Use `context.Context` as the first parameter for anything that does I/O
- Table-driven tests
- Functional options for complex configuration
- `internal/` package for unexported implementation details
