# Modules and Packages — Go

## How the Module System Works Under the Hood

Before Go modules (pre-1.11), Go used `GOPATH` — a single global directory where all your code and all your dependencies lived together. You couldn't have two projects using different versions of the same library. You couldn't vendor dependencies reliably. It was messy, and the community knew it.

Go modules, introduced as the official solution in Go 1.13, changed the unit of dependency management from a `GOPATH` workspace to a per-project `go.mod` file. Every Go project is now its own module, and modules declare exactly which versions of other modules they depend on.

The shift mirrors what happened in JavaScript when npm moved from a global install model to `package.json` per project — but Go's approach is stricter and more reproducible by design.

### The Module Identity Problem

The first thing to understand: **a module is identified by a path, not a name**. When you write:

```go
module github.com/acme/webhookd
```

That string `github.com/acme/webhookd` is the module's identity. It's used to:
- Resolve import paths (`import "github.com/acme/webhookd/internal/config"`)
- Identify the module in the Go module proxy
- Distinguish your module from all other modules in the universe

For public open-source code, using a domain you control (like a GitHub URL) ensures uniqueness. For private code or local experiments, you can use anything — but `github.com/yourname/projectname` is conventional even for private repos.

### Your notes
<!-- -->

---

## go.mod — The Module Manifest

Every module has exactly one `go.mod` at its root. Here's a realistic one:

```
module github.com/acme/webhookd

go 1.22

require (
    github.com/go-chi/chi/v5 v5.0.11
    github.com/jmoiron/sqlx v1.3.5
    github.com/lib/pq v1.10.9
)

require (
    github.com/mattn/go-sqlite3 v1.14.22 // indirect
)
```

**Breaking it down:**

`module github.com/acme/webhookd` — The module path. This is the prefix for all import paths within this module.

`go 1.22` — The minimum Go version this module expects. Also affects language feature availability (generics require `go 1.18`+, etc.)

`require (...)` — Direct dependencies. The versions here are locked — everyone who builds this module uses exactly these versions.

`// indirect` — A dependency of a dependency. Go records these for reproducibility; you usually don't edit them manually.

**Comparison to npm's package.json:**

| Concept | npm | Go modules |
|---|---|---|
| Project manifest | `package.json` | `go.mod` |
| Lock file | `package-lock.json` | `go.sum` |
| Install command | `npm install` | `go mod download` |
| Add dependency | `npm install pkg` | `go get pkg@version` |
| Remove unused | `npm prune` | `go mod tidy` |
| Vendor | `node_modules/` (default) | optional: `vendor/` |
| Version range | `^1.2.3` (semver range) | `v1.2.3` (exact, by default) |

The key philosophical difference: **Go modules use exact versions by default**. npm's `^1.2.3` means "any compatible 1.x version." Go's `v1.2.3` means exactly that version, period. Reproducible builds are a first-class goal.

### go.sum — The Cryptographic Lock File

`go.sum` contains the cryptographic checksums of every dependency you use:

```
github.com/go-chi/chi/v5 v5.0.11 h1:BnpYbFZ3T3S1WMpD79r7R5/+DP=...
github.com/go-chi/chi/v5 v5.0.11/go.mod h1:0sCRol3haJ1vd=...
```

Each line has three fields: `<module> <version> h1:<checksum>`. The checksum covers either the module zip (first line) or the `go.mod` file alone (second line).

**Why it matters:** When anyone builds your project, the Go toolchain verifies each downloaded dependency against these checksums. If a package is tampered with (supply chain attack), the build fails. This is Go's answer to the [left-pad incident](https://qz.com/646467/how-one-programmer-broke-the-internet-by-deleting-a-tiny-piece-of-code/) and npm-style supply chain attacks.

**You should commit `go.sum` to version control.** Never delete it or gitignore it.

### Your notes
<!-- -->

---

## Packages — The Unit of Compilation

A package is a directory of `.go` files that all share the same `package` declaration. Every file in a directory must be in the same package (with one exception: test files can use `package foo_test`).

```go
// All files in tracks/go/fundamentals/example-project/internal/config/
// must start with:
package config
```

**Key rules:**

1. **One package per directory.** The package name is typically the last element of the directory path: `internal/config` → package `config`.

2. **Package name ≠ import path.** The import path is what you write in the `import` statement. The package name is what you use to reference identifiers. Usually they match, but not always:
   ```go
   import "github.com/go-chi/chi/v5"  // import path
   chi.NewRouter()                     // package name is "chi"
   ```

3. **Package names are short and lowercase.** `config`, `checker`, `reporter` — not `ConfigPackage`, not `pkg_config`.

4. **No circular imports.** Package A can import B, B can import C, but C cannot import A. The Go compiler enforces a strict directed acyclic graph (DAG) of imports.

### Naming Conventions

The Go community has strong opinions on package names:

```
Good:           config, checker, reporter, handler, store
Avoid:          util, common, helpers, misc  (too vague, becomes a junk drawer)
Avoid stutter:  config.Config, reporter.Reporter  (see below)
```

**The stutter problem.** If your package is `config`, don't name your main type `Config` and export it as `config.Config` — that reads awkwardly. Instead, name the type for what it *is* in context, or accept the stutter when it's genuinely clear:

```go
// Bad: config.ConfigLoader — stutters
package config
type ConfigLoader struct{}

// Better: config.Loader — reads as "config's Loader"
package config
type Loader struct{}

// Or just: config.Load() as a function — no type needed
```

### Your notes
<!-- -->

---

## Exported vs Unexported — Go's Visibility System

Go's access control is purely lexical: **identifiers starting with an uppercase letter are exported (public); lowercase are unexported (package-private).**

No `public`, `private`, `protected` keywords. No access modifier syntax. Just case.

```go
package config

// Exported — visible to anyone who imports this package
type Loader struct {
    Path string  // exported field
    port int     // unexported field — only accessible within this package
}

// Exported function
func New(path string) *Loader { ... }

// Unexported function — only callable from within package config
func parseEnvFile(path string) (map[string]string, error) { ... }
```

This applies to:
- Types (`Config` vs `config`)
- Functions and methods (`Parse` vs `parse`)
- Struct fields (`Host` vs `host`)
- Constants (`DefaultPort` vs `defaultPort`)
- Variables

**What "package-private" actually means:** Unexported identifiers are accessible from *any file in the same package* — including test files in the same package (`package config`, not `package config_test`). It's not file-level privacy; it's package-level.

**Comparison to TypeScript:**

| Concept | TypeScript | Go |
|---|---|---|
| Public | `export` keyword on everything | Uppercase first letter |
| Private | `private` keyword or `#` prefix | Lowercase first letter |
| Package-level | No direct equivalent | Default for lowercase |
| File-level | Non-exported in module | No — Go has no file privacy |

In TypeScript, you export individual identifiers explicitly. In Go, visibility is determined entirely by naming convention, and everything in the same package can see everything else.

### Design Implication

This shapes how you design your APIs. Everything you export is a commitment — a public API surface that other packages depend on. Be conservative: start unexported and promote to exported only when you know external consumers need it.

```go
// internal implementation detail — unexported
func (l *Loader) readFile() ([]byte, error) { ... }

// public API — exported
func (l *Loader) Load() (*Config, error) { ... }
```

### Your notes
<!-- -->

---

## The `internal/` Directory

Go has one hardcoded visibility rule beyond the uppercase/lowercase distinction: **packages inside an `internal/` directory can only be imported by code within the parent of that `internal/` directory**.

```
myproject/
├── go.mod
├── main.go                         # can import internal/...
├── cmd/
│   └── server/
│       └── main.go                 # can import internal/...
├── internal/
│   ├── config/                     # restricted
│   │   └── config.go
│   └── auth/                       # restricted
│       └── auth.go
└── pkg/
    └── webhook/                    # public, anyone can import
        └── webhook.go
```

In this layout:
- `main.go` and `cmd/server/main.go` can import `internal/config` — they're inside the module
- **External modules cannot import `internal/config`** — the compiler enforces this
- Any module can import `pkg/webhook`

**Why this matters:** It lets you structure your codebase with clear public vs private package boundaries. You can refactor `internal/` packages freely without worrying about breaking external consumers. It's the Go equivalent of TypeScript's "barrel file" pattern, but enforced at the compiler level.

**Convention:**
- `internal/` — business logic, implementation details, domain code that shouldn't be consumed externally
- `pkg/` — genuinely reusable packages that external modules could import (though the `pkg/` name itself is debated — more on that below)

### Your notes
<!-- -->

---

## `init()` Functions

Every package can define one or more `init()` functions. They run automatically when the package is loaded, before `main()` executes.

```go
package config

import "os"

var defaultRegion string

func init() {
    // runs once when package config is first imported
    defaultRegion = os.Getenv("AWS_DEFAULT_REGION")
    if defaultRegion == "" {
        defaultRegion = "us-east-1"
    }
}
```

**Execution order:**
1. Package-level variables are initialized (in dependency order)
2. `init()` functions run (in source file order within a package, and dependency order across packages)
3. After all imports' `init()` functions complete, `main()` runs

**Multiple `init()` functions:** A single file can have multiple `init()` functions. A single package can have `init()` in multiple files. They all run — you can't call `init()` yourself, and you can't reference it from code.

```go
// Both of these run on package load:
func init() {
    registerMetrics()
}

func init() {
    validateEnvironment()
}
```

**The blank import — side effect imports:**

```go
import _ "github.com/lib/pq"  // registers the postgres driver via init()
```

The underscore means "import for side effects only — I don't use any exported names." Database drivers, image format decoders, and observability packages use this pattern to register themselves without the caller needing to explicitly invoke anything.

**When `init()` is appropriate:**
- Registering things in global registries (database drivers, codec formats)
- Validating environment at startup (with a panic if misconfigured)
- Setting up logging format

**When to avoid `init()`:**
- Any initialization that could fail in ways you want to handle gracefully
- Initialization that depends on configuration not available at package load
- Anything that makes testing hard (init() runs even in tests — you can't disable it)

The anti-pattern: using `init()` to set up global state that tests need to control. It makes tests non-deterministic and order-dependent.

### Your notes
<!-- -->

---

## Package Organization Patterns

There's no single "correct" Go project layout, but some patterns have become conventional.

### The Standard Project Layout (and its controversy)

The [golang-standards/project-layout](https://github.com/golang-standards/project-layout) repo popularized this structure:

```
myservice/
├── cmd/
│   └── myservice/
│       └── main.go        # entry point — thin, just wires things together
├── internal/
│   ├── handler/           # HTTP handlers
│   ├── store/             # database layer
│   └── domain/            # business logic
├── pkg/
│   └── httputil/          # reusable HTTP helpers (if truly reusable)
├── go.mod
├── go.sum
└── README.md
```

**The `cmd/` directory:** When you have multiple binaries (a server, a CLI tool, a worker), each gets its own subdirectory under `cmd/`. Each is a separate `package main` with its own entry point. They share code through `internal/` and optionally `pkg/`.

```
cmd/
├── server/main.go    # go run ./cmd/server
├── worker/main.go    # go run ./cmd/worker
└── migrate/main.go   # go run ./cmd/migrate
```

**The `pkg/` debate:** Some argue `pkg/` is meaningless — everything is a package. The Go standard library itself doesn't use it. Others use it to signal "this is genuinely reusable, not internal." If you're building a library for others, `pkg/` makes the public API discoverable. For an application, you might skip it entirely and put reusable code in `internal/`.

**Flat layout:** For small projects, flat is fine:

```
myservice/
├── config.go
├── handler.go
├── store.go
├── main.go
└── go.mod
```

Don't over-engineer. Structure your code for the reader, not for the organizational chart.

### Flat vs Layered

The question "how should I split packages?" comes up constantly. Two failure modes:

**Too granular:** Each concept gets its own package (`package errors`, `package types`, `package constants`). Now you have circular import problems and tiny files that tell you nothing about the codebase's purpose.

**Too coarse:** Everything in one package. Works for small codebases, becomes a junk drawer at scale.

**The practical rule:** Split by domain capability, not technical layer. Instead of `package handlers` + `package models` + `package database`, consider `package webhook` + `package billing` + `package auth` — each containing its own types, handlers, and data access code.

### Your notes
<!-- -->

---

## Dependency Management Commands

```bash
# Initialize a new module
go mod init github.com/yourname/projectname

# Add a dependency (fetches and updates go.mod + go.sum)
go get github.com/go-chi/chi/v5@v5.0.11

# Add latest version
go get github.com/some/package@latest

# Remove unused dependencies and add missing ones
go mod tidy

# Download dependencies to local cache without building
go mod download

# Vendor dependencies (copies into vendor/ directory)
go mod vendor

# Verify checksums in go.sum
go mod verify

# Show dependency graph
go mod graph

# Explain why a package is a dependency
go mod why github.com/some/package
```

**`go mod tidy` is the workhorse.** Run it after adding, removing, or updating dependencies. It ensures `go.mod` lists exactly what your code imports, no more, no less.

### Replace Directives — Local Development

When you're developing two modules simultaneously (e.g., your app and a library it uses), you don't want to push to GitHub just to test a change. The `replace` directive lets you point to a local copy:

```
module github.com/acme/webhookd

go 1.22

require (
    github.com/acme/go-webhooks v1.2.0
)

replace github.com/acme/go-webhooks => ../go-webhooks
```

Now imports of `github.com/acme/go-webhooks` use the local directory instead of the downloaded version. **Don't commit replace directives for local paths** — they break other developers' builds. Use them only during local development and remove before merging.

### Your notes
<!-- -->

---

## Build Tags and Conditional Compilation

Build tags let you include or exclude files from compilation based on conditions. Common uses: platform-specific code, excluding debug/test helpers, feature flags at build time.

```go
//go:build linux

package main

// This file is only compiled on Linux
```

Old-style (pre-Go 1.17, still seen in older code):
```go
// +build linux
```

**Syntax:** The `//go:build` comment must be on the first non-blank, non-comment line. It uses boolean expressions:

```go
//go:build linux && amd64         // compile on Linux amd64 only
//go:build !windows               // compile on everything except Windows
//go:build integration            // compile when: go test -tags integration
```

**Running tests with custom tags:**

```bash
go test -tags integration ./...
go build -tags production ./cmd/server
```

**Common real-world uses:**

```go
//go:build ignore  // exclude this file entirely (useful for generator scripts)

//go:build !production  // development-only debug helpers

//go:build integration  // integration tests that hit real databases
```

### Your notes
<!-- -->

---

## Comparison: npm/Node.js vs Go Modules

If you're coming from Node.js (which you are), here's the mental model translation:

| Concept | Node.js | Go |
|---|---|---|
| Project definition | `package.json` | `go.mod` |
| Lock file | `package-lock.json` / `yarn.lock` | `go.sum` |
| Dependencies live in | `node_modules/` | Module cache (`~/go/pkg/mod/`) |
| Import syntax | `import { thing } from 'package'` | `import "github.com/pkg"` |
| Relative imports | `import './utils'` | `import "github.com/mymod/utils"` |
| Default export | `export default` | First-letter uppercase |
| Named exports | `export const foo = ...` | First-letter uppercase |
| Private | Not exported in the file | Lowercase first letter |
| Internal package | No equivalent | `internal/` directory |
| Side-effect import | `import 'some-polyfill'` | `import _ "some/package"` |
| Monorepo | workspaces in `package.json` | `replace` directives or workspace mode |

**The key difference in import paths:**

```typescript
// TypeScript: relative or bare specifier
import { parse } from './config'
import { z } from 'zod'
```

```go
// Go: always the full module path
import "github.com/acme/webhookd/internal/config"
import "github.com/go-playground/validator/v10"
```

Go has no bare specifiers for third-party packages. The full module path is the import path. This eliminates an entire class of "which package is this?" ambiguity.

**ESM/CJS vs Go packages:**

Node's module system has the ESM/CommonJS split — whether a file is a module depends on `.mjs`, `.cjs`, file extension, or `"type": "module"` in `package.json`. Go has none of this complexity. Every `.go` file is part of a package; the `package` declaration at the top says which one.

### Your notes
<!-- -->

---

## Version Selection: Minimum Version Selection (MVS)

Go's dependency resolution algorithm is called **Minimum Version Selection (MVS)**. It's worth understanding because it's different from npm's approach.

When you have:
- App requires `lib-a@v1.5` and `lib-b@v2.0`
- `lib-a@v1.5` requires `lib-c@v1.2`
- `lib-b@v2.0` requires `lib-c@v1.7`

Go selects `lib-c@v1.7` — the **minimum version that satisfies all requirements**. This is the highest of all the minimum requirements.

npm's resolver tries to find the *latest compatible* version (subject to semver ranges). Go always picks the *minimum sufficient* version. The difference:

- npm: if `^1.2.0` and the latest is `1.9.5`, you get `1.9.5` (unless locked)
- Go: you get exactly the version you `require`, or the minimum that satisfies a transitive requirement

MVS makes builds more reproducible and upgrades explicit: you always know exactly which version you're getting, and upgrades happen only when you explicitly ask for them with `go get`.

### Your notes
<!-- -->

---

## Working With the Example Project

The `example-project/` directory in this module shows a minimal multi-package layout:

```
example-project/
├── go.mod                              # module declaration
├── main.go                             # entry point — package main
├── internal/
│   └── config/
│       └── config.go                  # unexportable outside module
└── pkg/
    └── greeting/
        └── greeting.go                # reusable, importable externally
```

Run it from the module root:

```bash
cd example-project
go run .
```

Notice:
- `main.go` imports both `internal/config` and `pkg/greeting` using their full module-relative paths
- `internal/config` is invisible to the outside world — only code inside `example-project/` can use it
- `pkg/greeting` is exported and could be used by external modules
- The package names (`config`, `greeting`) are the last path element by convention

### Your notes
<!-- -->
