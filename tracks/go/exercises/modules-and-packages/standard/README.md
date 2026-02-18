# Exercise: HTTP Health Checker — Package Restructure

## Scenario

Your team inherited a small internal CLI tool that pings a list of URLs and reports their HTTP status. It started as a single-file prototype, but it's grown to 300+ lines, mixing HTTP logic, output formatting, config parsing, and the CLI entry point in one file. A new engineer is joining who needs to extend the reporter with a JSON output mode, and a separate team wants to reuse the config loader in their tool. It's time to restructure.

## Brief

Split the monolithic `main.go` into a well-organized multi-package project following this layout:

```
healthcheck/
├── go.mod
├── cmd/
│   └── healthcheck/
│       └── main.go            # CLI entry point — thin wiring layer only
├── internal/
│   ├── checker/
│   │   └── checker.go         # HTTP health check logic
│   └── reporter/
│       └── reporter.go        # Output formatting (text table)
└── pkg/
    └── config/
        └── config.go          # Config loader — reusable, could be used by other tools
```

The starter `main.go` contains the complete working implementation. Your job is to understand the seams between concerns and split the code along package boundaries.

## Acceptance Criteria

- [ ] `pkg/config` provides a `Config` type and a `Load` function that reads from a `map[string]string` (simulating env vars). It is the only package that should be importable by external modules.
- [ ] `internal/checker` exposes a `Result` type and a `Check(url string, timeoutSec float64) Result` function. Does not import `reporter` or `config` (it is a pure function of a URL and timeout).
- [ ] `internal/reporter` exposes a `PrintResults(results []checker.Result)` function that formats output to stdout as a text table. Does not import `checker` for anything other than the `Result` type.
- [ ] `cmd/healthcheck/main.go` is the wiring layer: loads config, calls checker for each URL, passes results to reporter. It is ≤ 40 lines.
- [ ] No import cycles. Draw the dependency arrow: `cmd` → `internal/checker`, `cmd` → `internal/reporter`, `cmd` → `pkg/config`. Reporter → checker (for the type only). That's it.
- [ ] All exported identifiers have doc comments.
- [ ] `go build ./...` succeeds from the module root.
- [ ] The tests in `starter/main_test.go` (adapted for the new package structure) still pass.

## Constraints

- Do not change the external behavior — the tool should produce identical output before and after the restructure.
- `internal/checker` must not import `internal/reporter` or `pkg/config`. It should be independently testable.
- `pkg/config` must not import any `internal/` package — it's meant to be importable externally.
- Use only the standard library.

## Package Boundary Decisions

As you work through the split, ask yourself:

- **What's the seam?** Where does one concern end and another begin? HTTP checking has nothing to do with formatting — they should be separate.
- **Who owns the type?** `Result` needs to flow from `checker` → `reporter` → `cmd`. The type must live in `checker` (the producer), not in a shared `types` package.
- **internal vs pkg?** `checker` and `reporter` are specific to this tool's behavior — `internal/`. `config` reads env vars in a generic way that any tool could use — `pkg/`.
- **How thin is thin?** `cmd/healthcheck/main.go` should contain no business logic — just: load config, loop over URLs, call checker, pass to reporter, handle errors.

## Hints

<details>
<summary>Hint 1: Start with the types</summary>

Before moving any code, identify the types that cross package boundaries. `Result` needs to be visible to both `checker` (which produces it) and `reporter` (which formats it). Put `Result` in `checker` — the producer owns its output type.

</details>

<details>
<summary>Hint 2: Dependency direction matters</summary>

Draw the import graph before writing any code:

```
cmd/healthcheck  →  pkg/config
cmd/healthcheck  →  internal/checker
cmd/healthcheck  →  internal/reporter
internal/reporter  →  internal/checker   (for Result type only)
```

If you find yourself wanting `checker` to import `reporter`, stop — that's a cycle.

</details>

<details>
<summary>Hint 3: The go.mod path affects all imports</summary>

If your `go.mod` says `module github.com/yourname/healthcheck`, then:
- `pkg/config` is imported as `"github.com/yourname/healthcheck/pkg/config"`
- `internal/checker` is imported as `"github.com/yourname/healthcheck/internal/checker"`

Every import path is rooted at the module path. There are no relative imports in Go.

</details>

<details>
<summary>Hint 4: cmd/healthcheck/main.go structure</summary>

```go
func main() {
    cfg, err := config.Load(envMap())
    // handle err

    var results []checker.Result
    for _, url := range cfg.URLs {
        results = append(results, checker.Check(url, cfg.TimeoutSec))
    }

    reporter.PrintResults(results)
}
```

If your `main.go` is doing HTTP requests, parsing status codes, or building table strings — those belong in `checker` and `reporter`.

</details>
