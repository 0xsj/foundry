# Go Reference — Modules and Packages

> Extracted from [The Go Reference Manual](https://go.dev/ref/spec),
> [Go Modules Reference](https://go.dev/ref/mod), and
> [Go Command Reference](https://pkg.go.dev/cmd/go)
> for the `modules-and-packages` module.

---

## Packages

Source: [spec#Packages](https://go.dev/ref/spec#Packages)

A Go program is constructed by linking together packages. A package in turn is
constructed from one or more source files that together declare constants, types,
variables and functions belonging to the package and which are accessible in all
files of the same package.

### Package Clause

Source: [spec#Package_clause](https://go.dev/ref/spec#Package_clause)

Every source file begins with a package clause:

```
PackageClause  = "package" PackageName .
PackageName    = identifier .
```

The `PackageName` must not be the blank identifier. It is used as the default package
name for import declarations in files importing the package.

```go
package math
```

A set of files sharing the same PackageName form the implementation of a package.
An implementation may require that all source files for a package inhabit the same
directory.

### Package Naming Convention

Source: [Effective Go — Package names](https://go.dev/doc/effective_go#package-names)

- Package names are **lowercase, single-word** names
- No underscores or camelCase
- Short and descriptive
- The package name is the default name for imports; it need not be unique across
  all packages compiled into a binary

```go
// Good
package config
package checker
package reporter

// Avoid
package configPackage
package my_config
package ConfigPackage
```

---

## Import Declarations

Source: [spec#Import_declarations](https://go.dev/ref/spec#Import_declarations)

An import declaration makes identifiers exported by another package accessible
in the current source file.

```
ImportDecl       = "import" ( ImportSpec | "(" { ImportSpec ";" } ")" ) .
ImportSpec       = [ "." | PackageName ] ImportPath .
ImportPath       = string_lit .
```

### Forms

```go
import   "lib/math"           // access as: math.Sin
import m "lib/math"           // access as: m.Sin (local alias)
import . "lib/math"           // access as: Sin (dot import — avoid)
import _ "lib/math"           // import for side effects only (no access)
```

The interpretation of the `ImportPath` is implementation-dependent but is
conventionally a slash-separated sequence of identifiers.

### Import Rules

- The PackageName is used in qualified identifiers to access exported identifiers
  of the package within the importing source file
- If the PackageName is omitted, it defaults to the identifier specified in the
  `package` clause of the imported package
- If an explicit period (`.`) appears instead of a name, all the package's exported
  identifiers will be declared in the current file's block and can be accessed without
  a qualifier (strongly discouraged in production code)
- If the PackageName is a blank identifier (`_`), the package is imported solely for
  its side-effects (initialization)

---

## Exported Identifiers

Source: [spec#Exported_identifiers](https://go.dev/ref/spec#Exported_identifiers)

An identifier may be **exported** to permit access to it from another package.
An identifier is exported if both:

1. The first character of the identifier's name is a **Unicode uppercase letter**
2. The identifier is declared at **package block**, or it is a field name or method name

All other identifiers are not exported.

```go
package example

var exported = "visible outside"   // WRONG: starts lowercase
var Exported = "visible outside"   // correct: starts uppercase

type Config struct {
    Host string  // exported field
    port int     // unexported field
}
```

---

## The `internal` Directory

Source: [cmd/go#Internal_directories](https://pkg.go.dev/cmd/go#hdr-Internal_Directories)

Code in or below a directory named `internal` is importable only by code in the
directory tree rooted at the **parent of `internal`**.

```
/a/b/c/internal/d/e/f    importable by code in /a/b/c
                         NOT importable by code in /a/b/g
                         NOT importable by external modules
```

This means:

```
mymodule/
└── internal/
    └── config/         # importable only by code within mymodule/
        └── config.go
```

An external module that writes `import "github.com/acme/mymodule/internal/config"`
will get a compile error: **"use of internal package not allowed"**.

---

## The `init` Function

Source: [spec#Package_initialization](https://go.dev/ref/spec#Package_initialization)

Variables may also be initialized using functions named `init` declared in the
package block, with no arguments and no result parameters.

```go
func init() { ... }
```

Multiple such functions may be defined per file, even in a single source file.
In the package block, the `init` identifier itself is not declared. Thus `init`
functions cannot be referred to from anywhere in a program.

### Initialization Order

A package with no imports is initialized by assigning initial values to all its
package-level variables and then calling all `init` functions in the order they
appear in the source, as presented to the compiler.

If a package has imports, the imported packages are initialized before initializing
the package itself. If multiple packages import a package, the imported package
will be initialized only once.

Package initialization — variable initialization and the invocation of `init`
functions — happens in a single goroutine, sequentially, one package at a time.
An `init` function may launch other goroutines, which can run concurrently with
the initialization code.

### init Execution Sequence

1. All imported packages' `init` functions run first (recursively)
2. Package-level variables are initialized in declaration order (accounting for
   inter-variable dependencies)
3. `init()` functions in the current package run, in source file order

---

## Go Modules Reference

Source: [Go Modules Reference](https://go.dev/ref/mod)

### Module Definition

A **module** is a collection of packages that are released, versioned, and
distributed together. A module is identified by a **module path**, which is
declared in a `go.mod` file, together with information about the module's
dependencies.

### go.mod File Format

Source: [go.mod file reference](https://go.dev/ref/mod#go-mod-file)

```
GoModFile = { Directive } .
Directive = ModuleDirective
          | GoDirective
          | RequireDirective
          | ExcludeDirective
          | ReplaceDirective
          | RetractDirective .
```

#### module directive

```
ModuleDirective = "module" ModulePath newline .
ModulePath      = /* module path string */ .
```

Declares the module path for the current module. Serves as the import path prefix
for all packages within the module.

```
module github.com/acme/webhookd
```

#### go directive

```
GoDirective = "go" GoVersion newline .
GoVersion   = /* semver-like */ .
```

Indicates the minimum version of Go required to compile the module. Also enables
language features available in that version.

```
go 1.22
```

#### require directive

```
RequireDirective = "require" ( RequireSpec | "(" newline { RequireSpec } ")" ) newline .
RequireSpec     = ModulePath Version [ "// indirect" ] newline .
```

Declares a required module dependency and its minimum version.

```
require (
    github.com/go-chi/chi/v5 v5.0.11
    golang.org/x/text v0.14.0 // indirect
)
```

#### replace directive

```
ReplaceDirective = "replace" ( ReplaceSpec | "(" newline { ReplaceSpec } ")" ) newline .
ReplaceSpec      = ModulePath [ Version ] "=>" FilePath newline
                 | ModulePath [ Version ] "=>" ModulePath Version newline .
```

Replaces the content of a specific version of a module, or all versions:

```
// Replace with local directory
replace github.com/acme/mylib => ../mylib

// Replace specific version
replace github.com/acme/mylib v1.2.3 => github.com/fork/mylib v1.2.3-fixed
```

#### retract directive

```
RetractDirective = "retract" ( RetractSpec | "(" newline { RetractSpec } ")" ) newline .
RetractSpec      = Version | "[" Version "," Version "]" .
```

Indicates that a version (or range) should not be depended upon. Used when a
published version contains a serious bug or was published accidentally.

```
retract v1.0.5              // single version
retract [v1.0.0, v1.0.4]   // inclusive range
```

---

## go.sum File Format

Source: [go.sum file](https://go.dev/ref/mod#go-sum-files)

Each line in a `go.sum` file has three fields separated by spaces:

```
<module> <version>[/go.mod] <hash>
```

- `<module>`: module path
- `<version>`: the version
- `<hash>`: `h1:` followed by a base64-encoded SHA-256 hash

Two kinds of entries per module version:
1. Hash of the module zip file (`.zip` contents)
2. Hash of just the `go.mod` file (used for fetching `go.mod` without full download)

```
github.com/go-chi/chi/v5 v5.0.11 h1:BnpYbFZ3T3...
github.com/go-chi/chi/v5 v5.0.11/go.mod h1:0sCRol3h...
```

---

## Minimum Version Selection (MVS)

Source: [MVS algorithm](https://go.dev/ref/mod#minimal-version-selection)

Go uses **Minimum Version Selection (MVS)** to select the set of module versions
to use in a build.

### Algorithm

For each module required (directly or transitively):
- Collect all minimum version requirements for that module from all `go.mod` files
- Select the **maximum of all these minimums** — the smallest version that satisfies
  all requirements simultaneously

This is deterministic, reproducible, and never selects a version newer than the
**maximum of all stated minimums**.

### Properties

- **Reproducible**: given the same `go.mod` file, always selects the same versions
- **High-fidelity**: selects versions that module authors have tested
- **No surprising upgrades**: adding a dependency never upgrades an unrelated one
  beyond what's minimally required

### Comparison with npm

npm uses a SAT solver to find versions satisfying semver ranges; this can produce
different results over time as new versions are published. MVS always selects a
fixed, predictable version.

---

## Module Version Syntax

Source: [Versions](https://go.dev/ref/mod#versions)

Module versions take the form `vMAJOR.MINOR.PATCH`:

```
v0.0.1
v1.0.0
v1.2.3
v2.0.0-beta.1        // pre-release
v2.0.0-20240101abcdef  // pseudo-version (from commit hash)
```

### Major Version Suffixes

Source: [Major version suffixes](https://go.dev/ref/mod#major-version-suffixes)

For modules at `v2` or higher, the module path must end with `/vN`:

```
module github.com/acme/lib/v2

require (
    github.com/acme/lib/v2 v2.0.1
)
```

Import path:
```go
import "github.com/acme/lib/v2/config"
```

This allows v1 and v2 to coexist in the same build — they are considered separate
modules with different import paths.

### Pseudo-Versions

When depending on a specific commit (not a tagged release):

```
v0.0.0-20240101150405-abcdef123456
```

Format: `vX.Y.Z-YYYYMMDDHHMMSS-<commit>`

---

## Go Command Reference

### Module Commands

| Command | Description |
|---|---|
| `go mod init [module-path]` | Initialize a new module |
| `go mod tidy` | Add missing requirements, remove unused |
| `go mod download` | Download modules to local cache |
| `go mod vendor` | Copy dependencies into `vendor/` |
| `go mod verify` | Verify checksums against `go.sum` |
| `go mod graph` | Print the module dependency graph |
| `go mod why <pkg>` | Explain why a package is a dependency |
| `go get <pkg>[@version]` | Add or update a dependency |
| `go get <pkg>@none` | Remove a dependency |

### Build Commands Relevant to Packages

| Command | Description |
|---|---|
| `go build ./...` | Build all packages in module |
| `go run ./cmd/server` | Run a specific package |
| `go test ./...` | Test all packages |
| `go list ./...` | List all packages |
| `go vet ./...` | Run static analysis |

### Build Tags

Source: [Build constraints](https://pkg.go.dev/cmd/go#hdr-Build_constraints)

A build constraint (also known as a build tag) is a condition under which a file
should be included in the package:

```go
//go:build linux
//go:build linux && amd64
//go:build !cgo
//go:build ignore
```

The constraint must appear before the package clause and be followed by a blank line:

```go
//go:build linux

package main
```

Supply constraints at build time:

```bash
go build -tags "integration production" ./...
go test -tags integration ./...
```

---

## Package Documentation Convention

Source: [Go Doc Comments](https://go.dev/doc/comment)

Package documentation is written as a comment immediately before the `package`
clause with no intervening blank lines:

```go
// Package config provides configuration loading for the webhookd service.
// It reads values from environment variables, applies defaults, and validates
// required fields. Use New to create a Loader and Load to parse a Config.
package config
```

For exported identifiers, the doc comment immediately precedes the declaration:

```go
// Loader reads configuration from environment variables.
// Create one with New; use Load to produce a Config.
type Loader struct { ... }

// Load reads environment variables and returns a populated Config.
// It returns an error if required variables are missing or malformed.
func (l *Loader) Load() (*Config, error) { ... }
```

Conventions:
- Start with the name of the thing being documented
- First sentence is a complete sentence ending in a period
- Use `//go:doc` links for cross-references
- Run `go doc .` to view rendered output

---

## Internal Directory Enforcement

The Go toolchain enforces `internal` visibility at build time. Import of an
internal package from outside its parent tree produces:

```
imports github.com/acme/mylib/internal/config:
  use of internal package github.com/acme/mylib/internal/config not allowed
```

This check is performed by the `go` command when loading packages, not by the
compiler itself. It applies to all `go` commands: `build`, `test`, `run`, etc.
