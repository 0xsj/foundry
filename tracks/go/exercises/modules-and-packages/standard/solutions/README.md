# Solution: HTTP Health Checker — Package Restructure

## Approach

The refactor is a classic concern-separation exercise. The monolith's logic
naturally breaks along three seams:

1. **What to check** → `pkg/config` (URLs, timeout, verbosity)
2. **How to check** → `internal/checker` (HTTP mechanics, Result type)
3. **How to display** → `internal/reporter` (table formatting)
4. **How to wire it all** → `cmd/healthcheck/main.go` (entry point only)

## Package Boundary Decisions

### Why `pkg/config` and not `internal/config`?

`config.Load` reads from a `map[string]string` with no dependency on any other
internal package. Another tool in a monorepo could reuse this exact pattern to
load its own configuration. Putting it in `pkg/` signals that intent and makes it
importable by external code.

Compare to `internal/checker` and `internal/reporter` — those are specific to this
tool's behavior. `checker` makes HTTP requests; `reporter` knows the output table
format. Neither is a general-purpose utility.

### Why does `Result` live in `checker`, not a shared `types` package?

The producer owns the output type. `checker.Check` creates `Result`; `reporter.PrintResults`
consumes it. The dependency arrow should point from consumer to producer:

```
reporter  →  checker   (to access checker.Result)
```

A shared `types` package would invert this — `checker` and `reporter` would both
import `types`, adding indirection without benefit. In Go, small shared types belong
in the package that produces them.

### Why is `cmd/healthcheck/main.go` only 40 lines?

`main.go` should be a wiring layer, not a logic layer. The test for this:
if you can't understand what the binary does from `main.go` in 30 seconds, it has
too much logic in it. Business logic in `main` is untestable — you can't call `main`
from a test.

### The `envFromOS()` helper

`config.Load` takes `map[string]string` rather than calling `os.Getenv` internally.
This is a deliberate testability decision. Tests pass a constructed map:

```go
cfg, err := config.Load(map[string]string{
    "HEALTHCHECK_URLS": "https://example.com",
})
```

No need to set environment variables in tests, no test ordering issues, no
cleanup required. `envFromOS()` in `main.go` is the single point where the real
environment is accessed.

## Import Graph

```
cmd/healthcheck/main.go
    ├── pkg/config
    ├── internal/checker
    └── internal/reporter
            └── internal/checker  (for Result type)
```

`checker` is a leaf node — it imports only the standard library. This is ideal:
the most frequently imported package has the fewest dependencies, keeping the
dependency graph shallow.

## Key Decisions and Tradeoffs

| Decision | Rationale | Alternative |
|---|---|---|
| `config.Load(map[string]string)` | Testable without env | `os.Getenv` calls inside config |
| `Result` in `checker` | Producer owns type | Shared `types` package |
| `CheckAll` in `checker` | Keeps looping logic centralized | Loop in `main.go` |
| `reporter` imports `checker` | Natural flow of data | Use interface for Result |

## What Would Change for JSON Output?

If `reporter` needed a `--json` flag, the change is contained entirely in `reporter`:

```go
// reporter/reporter.go
func PrintText(results []checker.Result) { /* existing code */ }
func PrintJSON(results []checker.Result) { /* new json marshaling */ }
```

`cmd/healthcheck/main.go` would switch between them based on a flag. No other
package changes. This is the value of the separation: one concern changes, one
package changes.
