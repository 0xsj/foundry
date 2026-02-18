# Strings and Text — Go Reference

> Extracted from the [Go specification](https://go.dev/ref/spec), [Go standard library docs](https://pkg.go.dev/), and the [Go blog](https://go.dev/blog/strings). Consult these sources for authoritative and up-to-date information.

---

## String Types

### Specification: String Type

> A string type represents the set of string values. A string value is a (possibly empty) sequence of bytes. The number of bytes is called the length of the string and is never negative. Strings are immutable: once created, it is impossible to change the contents of a string.
>
> — [Go Specification: String types](https://go.dev/ref/spec#String_types)

```
StringType = "string" .
```

**Zero value:** `""`

**Length:** `len(s)` returns the number of **bytes** (not characters). For character count: `utf8.RuneCountInString(s)`.

**Indexing:** `s[i]` returns the i-th **byte** (type `uint8`/`byte`). Indexing out of range causes a runtime panic.

**Slicing:** `s[low:high]` returns a string containing bytes `low` through `high-1`. Both `low` and `high` default to `0` and `len(s)` respectively.

**Comparison:** Strings are comparable with `==`, `!=`, `<`, `>`, `<=`, `>=`. Comparison is lexicographic, byte by byte.

### Specification: String Literals

**Interpreted string literals** use double quotes. They process escape sequences:

```go
"\n"    // newline
"\t"    // tab
"\\"    // backslash
"\""    // double quote
"\x61"  // hex byte: 'a'
"\u0041" // Unicode code point (4 digits): 'A'
"\U0001F600" // Unicode code point (8 digits): 😀
```

**Raw string literals** use backticks. No escape processing. May contain newlines. Cannot contain backtick.

```go
`Hello\nWorld`  // the literal characters \, n — not a newline
`path\to\file`  // backslashes are literal — useful for Windows paths, regexps
```

### Specification: Rune Type

> `rune` is an alias for `int32` and, in all ways, equivalent to `int32`. By convention, it is used to distinguish character values from integer values.
>
> — [Go Specification: Numeric types](https://go.dev/ref/spec#Numeric_types)

**Rune literals** denote a Unicode code point. They are `int32` values:

```go
'a'        // 97
'é'        // 233
'€'        // 8364
'\n'       // 10
'\u0041'   // 65 ('A')
'\U0001F600' // 128512 (😀)
```

---

## UTF-8 and the `unicode/utf8` Package

**Go source code is always UTF-8.** String literals in Go source are UTF-8 encoded.

**Key functions** ([pkg.go.dev/unicode/utf8](https://pkg.go.dev/unicode/utf8)):

```go
utf8.RuneCountInString(s string) int
// Returns the number of runes in s. Invalid bytes count as one rune each.

utf8.ValidString(s string) bool
// Reports whether s consists entirely of valid UTF-8-encoded runes.

utf8.RuneLen(r rune) int
// Returns the number of bytes required to encode r. Returns -1 for invalid runes.

utf8.DecodeRuneInString(s string) (r rune, size int)
// Returns the first rune of s and its byte width.

utf8.EncodeRune(p []byte, r rune) int
// Writes r's UTF-8 encoding into p and returns the number of bytes written.

utf8.UTFMax = 4
// Maximum bytes per UTF-8-encoded rune.
```

**Conversion between string and []rune:**

```go
r := []rune(s)    // decode all runes into a slice
s := string(r)    // encode rune slice to UTF-8 string
```

Converting `string` → `rune` at a single index: use `for range` or `utf8.DecodeRuneInString`.

---

## `strings` Package

Full reference: [pkg.go.dev/strings](https://pkg.go.dev/strings)

### Search

```go
strings.Contains(s, substr string) bool
strings.ContainsAny(s, chars string) bool     // any Unicode code point in chars
strings.ContainsRune(s string, r rune) bool
strings.Count(s, substr string) int           // non-overlapping occurrences
strings.HasPrefix(s, prefix string) bool
strings.HasSuffix(s, suffix string) bool
strings.Index(s, substr string) int           // -1 if not found
strings.IndexByte(s string, c byte) int
strings.IndexRune(s string, r rune) int
strings.IndexAny(s, chars string) int         // first occurrence of any char
strings.LastIndex(s, substr string) int
strings.LastIndexAny(s, chars string) int
```

### Modification

```go
strings.Replace(s, old, new string, n int) string   // n=-1 for all
strings.ReplaceAll(s, old, new string) string        // shorthand for n=-1
strings.ToLower(s string) string
strings.ToUpper(s string) string
strings.ToTitle(s string) string                     // Unicode title case
strings.Title(s string) string                       // Deprecated: use golang.org/x/text
strings.Map(mapping func(rune) rune, s string) string // apply function to each rune
strings.Repeat(s string, count int) string
```

### Trim

```go
strings.Trim(s, cutset string) string          // leading and trailing chars in cutset
strings.TrimLeft(s, cutset string) string      // leading only
strings.TrimRight(s, cutset string) string     // trailing only
strings.TrimSpace(s string) string             // leading/trailing whitespace
strings.TrimPrefix(s, prefix string) string    // remove prefix if present
strings.TrimSuffix(s, suffix string) string    // remove suffix if present
strings.TrimFunc(s string, f func(rune) bool) string
```

### Split and Join

```go
strings.Split(s, sep string) []string          // splits all occurrences
strings.SplitN(s, sep string, n int) []string  // at most n substrings
strings.SplitAfter(s, sep string) []string     // include sep in each substring
strings.SplitAfterN(s, sep string, n int) []string
strings.Fields(s string) []string              // split on whitespace, trim empty
strings.FieldsFunc(s string, f func(rune) bool) []string
strings.Join(elems []string, sep string) string
strings.Cut(s, sep string) (before, after string, found bool)  // Go 1.18+
strings.CutPrefix(s, prefix string) (after string, found bool) // Go 1.20+
strings.CutSuffix(s, suffix string) (before string, found bool) // Go 1.20+
```

> `strings.Cut` is the idiomatic way to split on the first occurrence of a separator and get both halves. Prefer it over `Index` + slicing.

### Builder

```go
type Builder struct { /* unexported */ }

func (b *Builder) Grow(n int)                 // pre-allocate n bytes
func (b *Builder) Write(p []byte) (int, error)
func (b *Builder) WriteByte(c byte) error
func (b *Builder) WriteRune(r rune) (int, error)
func (b *Builder) WriteString(s string) (int, error)
func (b *Builder) String() string             // returns accumulated string
func (b *Builder) Len() int                   // current byte length
func (b *Builder) Reset()                     // reset to empty
func (b *Builder) Cap() int                   // current capacity
```

### Reader

```go
strings.NewReader(s string) *strings.Reader
// Implements io.Reader, io.ReaderAt, io.ByteReader, io.ByteScanner,
// io.RuneReader, io.RuneScanner, io.Seeker, io.WriterTo.
// Use to treat a string as an io.Reader without copying.
```

---

## `bytes` Package

Full reference: [pkg.go.dev/bytes](https://pkg.go.dev/bytes)

The `bytes` package mirrors `strings` for `[]byte` values. Functions have identical signatures with `string` replaced by `[]byte`.

**Key difference from strings:** `bytes` functions return `[]byte`, not `string`. Use when you're already working in `[]byte` to avoid round-trip conversions.

```go
bytes.Contains(b, subslice []byte) bool
bytes.Split(s, sep []byte) [][]byte
bytes.Join(s [][]byte, sep []byte) []byte
bytes.TrimSpace(s []byte) []byte
bytes.ToUpper(s []byte) []byte
bytes.Equal(a, b []byte) bool               // equivalent to bytes.Compare(a,b) == 0
bytes.Compare(a, b []byte) int              // -1, 0, or +1
bytes.HasPrefix(s, prefix []byte) bool
bytes.HasSuffix(s, suffix []byte) bool
bytes.Index(s, sep []byte) int
bytes.Count(s, sep []byte) int
bytes.Replace(s, old, new []byte, n int) []byte
bytes.ReplaceAll(s, old, new []byte) []byte
bytes.Cut(s, sep []byte) (before, after []byte, found bool)
```

### `bytes.Buffer`

```go
type Buffer struct { /* unexported */ }

func NewBuffer(buf []byte) *Buffer
func NewBufferString(s string) *Buffer

func (b *Buffer) Write(p []byte) (n int, err error)
func (b *Buffer) WriteString(s string) (n int, err error)
func (b *Buffer) WriteByte(c byte) error
func (b *Buffer) WriteRune(r rune) (n int, err error)
func (b *Buffer) Read(p []byte) (n int, err error)
func (b *Buffer) ReadByte() (byte, error)
func (b *Buffer) ReadRune() (r rune, size int, err error)
func (b *Buffer) ReadString(delim byte) (line string, err error)
func (b *Buffer) Bytes() []byte
func (b *Buffer) String() string
func (b *Buffer) Len() int
func (b *Buffer) Cap() int
func (b *Buffer) Grow(n int)
func (b *Buffer) Reset()
func (b *Buffer) Truncate(n int)
```

`bytes.Buffer` implements `io.Reader`, `io.Writer`, `io.ByteReader`, `io.ByteWriter`, `io.RuneReader`, `io.ReaderFrom`, and `io.WriterTo`.

---

## `fmt` Package

Full reference: [pkg.go.dev/fmt](https://pkg.go.dev/fmt)

### Printing Functions

```go
fmt.Print(a ...any) (n int, err error)         // no newline, spaces between non-string args
fmt.Println(a ...any) (n int, err error)       // newline, spaces between all args
fmt.Printf(format string, a ...any) (n int, err error)
fmt.Fprint(w io.Writer, a ...any) (n int, err error)
fmt.Fprintln(w io.Writer, a ...any) (n int, err error)
fmt.Fprintf(w io.Writer, format string, a ...any) (n int, err error)
fmt.Sprint(a ...any) string
fmt.Sprintln(a ...any) string
fmt.Sprintf(format string, a ...any) string
```

### Error Functions

```go
fmt.Errorf(format string, a ...any) error
// Use %w to wrap an error: fmt.Errorf("open %s: %w", path, err)
// Wrapped errors are unwrappable with errors.Is and errors.As
```

### Scanning Functions

```go
fmt.Scan(a ...any) (n int, err error)
fmt.Scanln(a ...any) (n int, err error)
fmt.Scanf(format string, a ...any) (n int, err error)
fmt.Fscan(r io.Reader, a ...any) (n int, err error)
fmt.Sscan(str string, a ...any) (n int, err error)
fmt.Sscanf(str string, format string, a ...any) (n int, err error)
```

### Format Verbs — Complete Reference

**General:**

| Verb | Description |
|---|---|
| `%v` | Default format |
| `%+v` | With field names (structs) |
| `%#v` | Go syntax representation |
| `%T` | Go type |
| `%%` | Literal `%` |

**Boolean:** `%t` → `true` or `false`

**Integer:**

| Verb | Description |
|---|---|
| `%b` | Base 2 |
| `%c` | Character represented by Unicode code point |
| `%d` | Base 10 |
| `%o` | Base 8 |
| `%O` | Base 8 with `0o` prefix |
| `%q` | Single-quoted character literal, safely escaped |
| `%x` | Base 16, lowercase |
| `%X` | Base 16, uppercase |
| `%U` | Unicode format: `U+1234` |

**Float:**

| Verb | Description |
|---|---|
| `%e` | Scientific notation: `-1.234456e+78` |
| `%E` | Scientific notation: `-1.234456E+78` |
| `%f` | Decimal: `-123.456` |
| `%F` | Decimal: `-123.456` (same as `%f`) |
| `%g` | `%e` for large exponents, `%f` otherwise |
| `%G` | `%E` for large exponents, `%F` otherwise |

**String / byte slice:**

| Verb | Description |
|---|---|
| `%s` | Unquoted string or slice |
| `%q` | Double-quoted, safely escaped |
| `%x` | Hex encoding, lowercase |
| `%X` | Hex encoding, uppercase |

**Pointer:** `%p` → base 16 with `0x` prefix

**Width and Precision:**

```
%[flags][width][.precision]verb

Flags:
  +    always print sign for numbers
  -    left-align (default: right-align)
  #    alternate format (%#x adds 0x prefix, %#v uses Go syntax)
  ' '  space before positive numbers
  0    pad with zeros (not spaces)

Width:  minimum field width (in bytes)
Precision: for floats — decimal places; for strings — max bytes printed
```

### Stringer and GoStringer Interfaces

```go
// Implement to control %v / %s output
type Stringer interface {
    String() string
}

// Implement to control %#v output
type GoStringer interface {
    GoString() string
}
```

---

## `strconv` Package

Full reference: [pkg.go.dev/strconv](https://pkg.go.dev/strconv)

### Integer Conversions

```go
strconv.Atoi(s string) (int, error)
strconv.Itoa(i int) string

strconv.ParseInt(s string, base int, bitSize int) (int64, error)
// base: 0 (auto-detect), 2, 8, 10, 16
// bitSize: 0 (int), 8, 16, 32, 64

strconv.ParseUint(s string, base int, bitSize int) (uint64, error)

strconv.FormatInt(i int64, base int) string
strconv.FormatUint(i uint64, base int) string
strconv.AppendInt(dst []byte, i int64, base int) []byte    // appends to dst — no alloc
```

### Float Conversions

```go
strconv.ParseFloat(s string, bitSize int) (float64, error)
// bitSize: 32 or 64

strconv.FormatFloat(f float64, fmt byte, prec int, bitSize int) string
// fmt: 'b' (binary), 'e' (scientific), 'E', 'f' (decimal), 'g', 'G', 'x', 'X'
// prec: decimal digits (-1 = shortest representation)
// bitSize: 32 or 64

strconv.AppendFloat(dst []byte, f float64, fmt byte, prec int, bitSize int) []byte
```

### Bool Conversions

```go
strconv.ParseBool(str string) (bool, error)
// Accepts: "1", "t", "T", "TRUE", "true", "True",
//          "0", "f", "F", "FALSE", "false", "False"

strconv.FormatBool(b bool) string  // "true" or "false"
strconv.AppendBool(dst []byte, b bool) []byte
```

### String Quoting

```go
strconv.Quote(s string) string          // returns double-quoted Go string literal
strconv.QuoteToASCII(s string) string   // non-ASCII → Unicode escapes
strconv.Unquote(s string) (string, error) // inverse of Quote
strconv.CanBackquote(s string) bool     // can s be represented as a raw string literal?
```

### Error Type

```go
type NumError struct {
    Func string  // function name ("ParseInt", "ParseFloat", etc.)
    Num  string  // input that failed
    Err  error   // the actual error (ErrRange, ErrSyntax)
}

var (
    strconv.ErrRange  = errors.New("value out of range")
    strconv.ErrSyntax = errors.New("invalid syntax")
)
```

---

## `regexp` Package

Full reference: [pkg.go.dev/regexp](https://pkg.go.dev/regexp)

Go uses **RE2 syntax**: no lookaheads, no lookbehinds, no backreferences. Guaranteed O(n) time.

RE2 syntax reference: [github.com/google/re2/wiki/Syntax](https://github.com/google/re2/wiki/Syntax)

### Compilation

```go
regexp.Compile(expr string) (*Regexp, error)
regexp.MustCompile(str string) *Regexp     // panics on error — use for static patterns
regexp.CompilePOSIX(expr string) (*Regexp, error)  // leftmost-longest match
regexp.MustCompilePOSIX(str string) *Regexp
```

### Matching

```go
re.Match(b []byte) bool
re.MatchString(s string) bool
re.MatchReader(r io.RuneReader) bool

// Package-level (compiles internally — not for hot loops)
regexp.MatchString(pattern, s string) (matched bool, err error)
```

### Finding

```go
// Return first match
re.Find(b []byte) []byte
re.FindString(s string) string
re.FindIndex(b []byte) []int          // [start, end]
re.FindStringIndex(s string) []int    // [start, end]

// Return all matches (n=-1 for all)
re.FindAll(b []byte, n int) [][]byte
re.FindAllString(s string, n int) []string
re.FindAllIndex(b []byte, n int) [][]int
re.FindAllStringIndex(s string, n int) [][]int

// Include submatches (capture groups)
// m[0] = full match, m[1] = group 1, m[2] = group 2, ...
re.FindSubmatch(b []byte) [][]byte
re.FindStringSubmatch(s string) []string
re.FindAllSubmatch(b []byte, n int) [][][]byte
re.FindAllStringSubmatch(s string, n int) [][]string
```

### Replacing

```go
re.ReplaceAll(src, repl []byte) []byte
re.ReplaceAllString(src, repl string) string
// In repl: $1 = first group, ${name} = named group, $$ = literal $

re.ReplaceAllFunc(src []byte, repl func([]byte) []byte) []byte
re.ReplaceAllStringFunc(src string, repl func(string) string) string

re.ReplaceAllLiteral(src, repl []byte) []byte  // no $ expansion
re.ReplaceAllLiteralString(src, repl string) string
```

### Splitting

```go
re.Split(s string, n int) []string     // split on matches; n=-1 for all
```

### Named Groups

```go
re := regexp.MustCompile(`(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})`)

match := re.FindStringSubmatch("2024-01-15")
names := re.SubexpNames()  // ["", "year", "month", "day"]

// Build named capture map
result := make(map[string]string)
for i, name := range names {
    if i != 0 && name != "" {
        result[name] = match[i]
    }
}
// result: {"year": "2024", "month": "01", "day": "15"}
```

---

## `text/template` Package

Full reference: [pkg.go.dev/text/template](https://pkg.go.dev/text/template)

### Template Syntax

```
{{.FieldName}}          access field or method on dot
{{.Method arg}}         call method with argument
{{$var := .Field}}      assign to variable
{{if .Condition}} ... {{else}} ... {{end}}
{{range .Items}} ... {{end}}     iterate slice/map/channel
{{range $i, $v := .Items}} ...  with index
{{with .Value}} ... {{end}}     set dot to Value if non-zero
{{template "name" .}}   invoke named sub-template
{{define "name"}} ... {{end}}   define named template
{{block "name" .}} ... {{end}}  define and invoke
- or {{- trim whitespace adjacent to action
{{/* comment */}}
```

### Core Types

```go
// Template
type Template struct { /* unexported */ }

template.New(name string) *Template
t.Parse(text string) (*Template, error)
t.ParseFiles(filenames ...string) (*Template, error)
t.ParseGlob(pattern string) (*Template, error)
template.Must(t *Template, err error) *Template

t.Execute(wr io.Writer, data any) error
t.ExecuteTemplate(wr io.Writer, name string, data any) error

t.Funcs(funcMap FuncMap) *Template  // add custom functions
t.Clone() (*Template, error)
t.Lookup(name string) *Template
```

### Custom Functions

```go
funcMap := template.FuncMap{
    "upper": strings.ToUpper,
    "formatDate": func(t time.Time) string {
        return t.Format("2006-01-02")
    },
}
t := template.New("report").Funcs(funcMap).Parse(`{{upper .Name}}`)
```

---

## `html/template` Package

Full reference: [pkg.go.dev/html/template](https://pkg.go.dev/html/template)

**Identical API to `text/template`**. Key differences:

1. **Auto-escaping:** Values substituted into HTML context are HTML-escaped. Values in URL context are URL-escaped. Values in JS context are JS-escaped. This is contextual escaping — the template engine tracks which context each substitution appears in.

2. **`template.HTML` type:** Use `template.HTML(s)` to mark a string as already safe HTML that should NOT be escaped. Use with extreme caution — only for strings you have fully sanitized.

3. **`template.URL` type:** Marks a string as a safe URL.

```go
import "html/template"

// This is safe — user input in HTML context is auto-escaped
const tmpl = `<p>Hello, {{.UserName}}!</p>`

// This would XSS if you used text/template instead
// html/template renders: <p>Hello, &lt;script&gt;alert(1)&lt;/script&gt;!</p>
```

**Import rule:** If your template outputs HTML, always import `html/template`. If it outputs plaintext, email bodies without HTML, config files, code, etc., use `text/template`.

---

## String Conversion Rules

> From the Go specification ([Conversions](https://go.dev/ref/spec#Conversions)):

```go
// string ↔ integer: converts the integer to its UTF-8 representation
string(65)      // "A"
string('\u4e16') // "世"

// string ↔ []byte: converts between UTF-8 string and byte slice
[]byte("hello")  // [104 101 108 108 111]
string([]byte{104, 101, 108, 108, 111})  // "hello"

// string ↔ []rune: converts between UTF-8 string and Unicode code points
[]rune("hello")  // [104 101 108 108 111]
string([]rune{104, 101, 108, 108, 111})  // "hello"
```

**All these conversions allocate.** The compiler may optimize away allocations in some patterns (temporary conversions in comparisons, map lookups), but this is not guaranteed.

---

## Performance Reference

### String Concatenation Benchmarks

From the Go blog and standard benchmarks (approximate, Go 1.21, amd64):

| Method | 1000 concatenations | Notes |
|---|---|---|
| `+=` in loop | ~500μs | O(n²) — never use in loops |
| `fmt.Sprintf` accumulate | ~100μs | Overhead from format parsing |
| `strings.Builder` | ~5μs | Amortized O(n) |
| `strings.Join(slice, "")` | ~4μs | Single allocation |

### Profiling Strings

```bash
# CPU profile — find string hotspots
go test -bench=. -cpuprofile=cpu.prof ./...
go tool pprof cpu.prof

# Memory profile — find allocation hotspots
go test -bench=. -memprofile=mem.prof ./...
go tool pprof mem.prof

# Benchmark specific functions
go test -bench=BenchmarkMyFunc -benchmem ./...
```

---

## Official Documentation Links

| Package | URL |
|---|---|
| `strings` | https://pkg.go.dev/strings |
| `bytes` | https://pkg.go.dev/bytes |
| `fmt` | https://pkg.go.dev/fmt |
| `strconv` | https://pkg.go.dev/strconv |
| `regexp` | https://pkg.go.dev/regexp |
| `text/template` | https://pkg.go.dev/text/template |
| `html/template` | https://pkg.go.dev/html/template |
| `unicode/utf8` | https://pkg.go.dev/unicode/utf8 |
| `unicode` | https://pkg.go.dev/unicode |
| Go specification: Strings | https://go.dev/ref/spec#String_types |
| Go blog: Strings, bytes, runes and characters | https://go.dev/blog/strings |
| RE2 syntax | https://github.com/google/re2/wiki/Syntax |
