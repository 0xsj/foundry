# Strings and Text — Go

## What Is a Go String?

A Go string is an **immutable sequence of bytes**. That's the complete definition. There is no character type underneath — just bytes. A string value is a two-field struct in the runtime:

```
type StringHeader struct {
    Data uintptr  // pointer to the byte array
    Len  int      // number of bytes (not characters)
}
```

This is important: `len("café")` returns `5`, not `4`. The `é` character takes two bytes in UTF-8 encoding. If you come from JavaScript, this will surprise you — JS strings are UTF-16 and `"café".length` returns `4`.

```go
s := "café"
fmt.Println(len(s))        // 5 — bytes, not characters
fmt.Println(len([]rune(s))) // 4 — characters (runes)
```

**Immutability:** You cannot modify a string byte-by-byte. `s[0] = 'C'` is a compile error. To manipulate strings, you convert to `[]byte` or `[]rune`, modify, then convert back.

---

## The Three String Types

Go has three types you'll use for text work. Understanding when to use each eliminates a whole class of bugs.

### `string` — Immutable byte sequence

Use for:
- Storing text you don't need to mutate
- Function parameters and return values
- Map keys
- Anything that crosses API boundaries

```go
name := "alice"    // immutable
key  := "user:42"  // fine as a map key
```

**Zero value:** `""` (empty string, not nil). Strings can't be nil in Go — unlike Java or Python, there's no null string.

### `[]byte` — Mutable byte slice

Use for:
- Text manipulation (find/replace, parsing)
- I/O operations (`os.ReadFile` returns `[]byte`)
- When you need to mutate character by character (ASCII-safe)
- Building strings incrementally without `strings.Builder`

```go
b := []byte("hello world")
b[0] = 'H'           // fine — []byte is mutable
result := string(b)  // convert back when done
```

**Warning:** Indexing `[]byte` gives you a byte, not a character. For multibyte characters (emoji, accents, CJK), byte-level manipulation requires care.

### `[]rune` — Mutable Unicode code point slice

Use for:
- Character-by-character manipulation of Unicode text
- When you need `len` to return character count, not byte count
- Reversing strings, substring by character index

```go
r := []rune("café")
fmt.Println(len(r))  // 4 — character count
r[3] = 'e'           // change é to e — safe, works correctly
result := string(r)  // "cafe"
```

A `rune` is an alias for `int32`. It holds a Unicode code point (0–0x10FFFF).

### Comparison Table

| Feature | `string` | `[]byte` | `[]rune` |
|---|---|---|---|
| Mutable | No | Yes | Yes |
| `len()` returns | Bytes | Bytes | Characters |
| Map key | Yes | No | No |
| I/O friendly | Sometimes | Yes | No |
| Unicode-safe indexing | No | No | Yes |
| Memory | 16B header | 24B header | 24B header |

---

## UTF-8 Encoding — How It Works Under the Hood

Go source files are UTF-8. Go strings are UTF-8. This is baked into the language, not a library choice.

**UTF-8 encoding rules:**

| Code point range | Bytes | Byte pattern |
|---|---|---|
| U+0000–U+007F (ASCII) | 1 | `0xxxxxxx` |
| U+0080–U+07FF | 2 | `110xxxxx 10xxxxxx` |
| U+0800–U+FFFF | 3 | `1110xxxx 10xxxxxx 10xxxxxx` |
| U+10000–U+10FFFF | 4 | `11110xxx 10xxxxxx 10xxxxxx 10xxxxxx` |

So ASCII characters (a–z, A–Z, 0–9, punctuation) are always one byte. Most Latin characters with accents are two bytes. CJK ideographs are three bytes. Emoji are typically four bytes.

```go
s := "A€🎉"
// A   = 0x41           — 1 byte
// €   = U+20AC         — 3 bytes (E2 82 AC)
// 🎉  = U+1F389        — 4 bytes (F0 9F 8E 89)
fmt.Println(len(s))  // 8
```

**Key consequence:** You cannot safely index a Go string by character position. `s[2]` gives you the 3rd byte, which may be in the middle of a multibyte character.

---

## Iterating Strings

Two forms — they behave very differently:

### `for range` — Decodes runes

```go
for i, r := range "café" {
    fmt.Printf("index %d: %c (U+%04X)\n", i, r, r)
}
// index 0: c (U+0063)
// index 1: a (U+0061)
// index 2: f (U+0066)
// index 3: é (U+00E9)    ← byte index 3, rune occupies bytes 3–4
```

`i` is the **byte offset** where the rune starts, not the character index. Notice the index jumps by 2 for `é` (a 2-byte character). Use `for range` when you want to process characters correctly.

### `for i := 0; i < len(s); i++` — Iterates bytes

```go
for i := 0; i < len(s); i++ {
    fmt.Printf("byte[%d] = 0x%02X\n", i, s[i])
}
// byte[0] = 0x63  (c)
// byte[1] = 0x61  (a)
// byte[2] = 0x66  (f)
// byte[3] = 0xC3  (first byte of é)
// byte[4] = 0xA9  (second byte of é)
```

Use the byte loop when you need raw bytes: parsing binary protocols, checksums, or ASCII-only text where you're sure no multibyte characters appear.

**The classic bug:** Using byte indexing on a string that might contain non-ASCII characters.

```go
// WRONG — breaks on any non-ASCII input
func toUpperFirstChar(s string) string {
    if len(s) == 0 {
        return s
    }
    return string(s[0]-32) + s[1:]  // assumes s[0] is a single-byte ASCII char
}

// CORRECT — uses runes
func toUpperFirstChar(s string) string {
    r := []rune(s)
    if len(r) == 0 {
        return s
    }
    r[0] = unicode.ToUpper(r[0])
    return string(r)
}
```

---

## The `strings` Package

The `strings` package covers almost everything you need for string manipulation. Here are the functions you'll reach for constantly:

### Search and Test

```go
strings.Contains("webhook-error", "error")   // true
strings.HasPrefix("user:42", "user:")        // true
strings.HasSuffix("config.json", ".json")    // true
strings.Count("aaabba", "a")                 // 4
strings.Index("hello", "ll")                 // 2  (-1 if not found)
strings.LastIndex("a/b/c", "/")             // 3
```

### Split and Join

```go
// Split on separator — returns []string
parts := strings.Split("a,b,c", ",")         // ["a", "b", "c"]
parts = strings.SplitN("a:b:c", ":", 2)      // ["a", "b:c"] — max 2 parts

// Fields: split on any whitespace (handles tabs, newlines, multiple spaces)
words := strings.Fields("  hello   world  ") // ["hello", "world"]

// Join
joined := strings.Join([]string{"a", "b", "c"}, ", ") // "a, b, c"
```

**JS comparison:** `strings.Split("a,b", ",")` ≈ `"a,b".split(",")`. But Go's `strings.Fields` has no direct JS equivalent — you'd need `.split(/\s+/).filter(Boolean)`.

### Modification

```go
strings.TrimSpace("  hello  ")               // "hello"
strings.Trim("!!hello!!", "!")               // "hello" — trims leading/trailing chars in cutset
strings.TrimPrefix("user:42", "user:")       // "42"
strings.TrimSuffix("config.json", ".json")  // "config"

strings.ToLower("Hello World")               // "hello world"
strings.ToUpper("Hello World")               // "HELLO WORLD"
strings.Title("hello world")                 // "Hello World" (deprecated — use golang.org/x/text)

strings.Replace("aaabba", "a", "X", 2)      // "XXabba" — replace first 2
strings.ReplaceAll("aaabba", "a", "X")       // "XXXbbX" — replace all
```

### Comparison

```go
strings.EqualFold("Go", "go")               // true — case-insensitive compare
// Do NOT use == for case-insensitive — that's case-sensitive
```

---

## `strings.Builder` — Efficient String Construction

The single most important performance tip for string work in Go: **never concatenate strings in a loop with `+`**.

**Why `+` in a loop is O(n²):**

Strings are immutable. Every `+` allocates a new string and copies both sides into it. In a loop:
- Iteration 1: alloc 1 byte
- Iteration 2: alloc 2 bytes, copy 2
- Iteration 3: alloc 3 bytes, copy 3
- ...
- Total: O(n²) allocations and copies

```go
// BAD — O(n²) allocations
result := ""
for _, line := range lines {
    result += line + "\n"  // new allocation every iteration
}

// GOOD — O(n) with Builder
var b strings.Builder
b.Grow(estimatedSize) // optional: pre-allocate to avoid resizing
for _, line := range lines {
    b.WriteString(line)
    b.WriteByte('\n')
}
result := b.String()
```

`strings.Builder` maintains a `[]byte` internally. `WriteString` appends bytes. `String()` converts to string at the end — single allocation.

**When to use what:**

| Approach | When to use |
|---|---|
| `+` | Simple, one-off concatenation (2–3 strings) |
| `fmt.Sprintf` | Formatting with mixed types |
| `strings.Builder` | Loop concatenation, many appends |
| `strings.Join` | Joining a known slice with a separator |
| `bytes.Buffer` | When you also need `io.Writer` interface |

---

## `fmt` Package — Formatting

The `fmt` package is Go's printf-style formatting library. The key function is `fmt.Sprintf` for building formatted strings, `fmt.Printf` for printing to stdout, `fmt.Fprintf` for writing to any `io.Writer`.

### Format Verbs

| Verb | Meaning | Example |
|---|---|---|
| `%v` | Default format | `fmt.Sprintf("%v", 42)` → `"42"` |
| `%+v` | Default format with field names (structs) | `fmt.Sprintf("%+v", u)` → `{Name:alice Age:30}` |
| `%#v` | Go syntax representation | `fmt.Sprintf("%#v", u)` → `main.User{Name:"alice", Age:30}` |
| `%T` | Type | `fmt.Sprintf("%T", 42)` → `"int"` |
| `%s` | String (no quotes) | `fmt.Sprintf("%s", "hi")` → `"hi"` |
| `%q` | Quoted string | `fmt.Sprintf("%q", "hi")` → `"\"hi\""` |
| `%d` | Integer, base 10 | `fmt.Sprintf("%d", 42)` → `"42"` |
| `%x` | Hex, lowercase | `fmt.Sprintf("%x", 255)` → `"ff"` |
| `%X` | Hex, uppercase | `fmt.Sprintf("%X", 255)` → `"FF"` |
| `%b` | Binary | `fmt.Sprintf("%b", 10)` → `"1010"` |
| `%f` | Float, decimal | `fmt.Sprintf("%f", 3.14)` → `"3.140000"` |
| `%.2f` | Float, 2 decimal places | `fmt.Sprintf("%.2f", 3.14159)` → `"3.14"` |
| `%e` | Scientific notation | `fmt.Sprintf("%e", 3.14)` → `"3.140000e+00"` |
| `%t` | Boolean | `fmt.Sprintf("%t", true)` → `"true"` |
| `%p` | Pointer | `fmt.Sprintf("%p", &x)` → `"0x......"` |
| `%w` | Wrap an error (errors.Errorf) | `fmt.Errorf("loading: %w", err)` |

**Width and padding:**

```go
fmt.Sprintf("%10s", "hi")   // "        hi" — right-align, width 10
fmt.Sprintf("%-10s", "hi")  // "hi        " — left-align, width 10
fmt.Sprintf("%010d", 42)    // "0000000042" — zero-pad
fmt.Sprintf("%+d", 42)      // "+42" — always show sign
```

### `fmt.Fprintf` — Writing to Writers

Any `io.Writer` works with `fmt.Fprintf`: files, HTTP response writers, network connections, `strings.Builder`.

```go
// Write to a file
f, _ := os.Create("out.txt")
fmt.Fprintf(f, "timestamp: %d\n", time.Now().Unix())

// Write to a strings.Builder
var sb strings.Builder
fmt.Fprintf(&sb, "request_id=%s status=%d", reqID, 200)
log.Println(sb.String())
```

---

## `strconv` Package — Type Conversions

`strconv` handles conversions between strings and numeric types. This comes up constantly when parsing configuration, reading CSV, processing web form data.

```go
// String → int
n, err := strconv.Atoi("42")    // simple int parsing
n, err := strconv.ParseInt("FF", 16, 64)  // base 16, 64-bit int

// int → String
s := strconv.Itoa(42)           // "42"
s := strconv.FormatInt(255, 16) // "ff"
s := strconv.FormatInt(255, 2)  // "11111111"

// String → float
f, err := strconv.ParseFloat("3.14", 64)

// float → String
s := strconv.FormatFloat(3.14159, 'f', 2, 64)  // "3.14"
s := strconv.FormatFloat(3.14159, 'e', 3, 64)  // "3.142e+00"

// String → bool
b, err := strconv.ParseBool("true")   // true
b, err = strconv.ParseBool("1")       // true
b, err = strconv.ParseBool("false")   // false

// bool → string
s := strconv.FormatBool(true)   // "true"
```

**Always check the error.** `strconv` functions return `(value, error)`. Ignoring the error is how you get silent zero values from failed parses.

```go
// Don't do this
n, _ := strconv.Atoi(userInput)  // silently 0 if input is invalid

// Do this
n, err := strconv.Atoi(userInput)
if err != nil {
    return fmt.Errorf("invalid count %q: %w", userInput, err)
}
```

---

## `bytes` Package

The `bytes` package mirrors `strings` but operates on `[]byte`. Use it when you already have `[]byte` (from I/O) and want to avoid the round-trip `string(b)` → operate → `[]byte(result)`.

```go
import "bytes"

data := []byte("hello, world")
bytes.Contains(data, []byte("world"))     // true
bytes.Split(data, []byte(","))            // [[]byte("hello"), []byte(" world")]
bytes.TrimSpace([]byte("  hi  "))        // []byte("hi")
bytes.ToUpper(data)                       // []byte("HELLO, WORLD")
```

### `bytes.Buffer` vs `strings.Builder`

Both build up content incrementally. The difference:

- `bytes.Buffer` implements `io.Reader` and `io.Writer` — use it when you need both read and write, or when passing to functions that expect `io.Reader`
- `strings.Builder` is write-only and only converts to `string` — simpler API when that's all you need

```go
// bytes.Buffer — when you need io.Reader/Writer
var buf bytes.Buffer
buf.WriteString("line 1\n")
buf.WriteString("line 2\n")
io.Copy(w, &buf)  // pass to io.Writer
scanner := bufio.NewScanner(&buf)  // treat as io.Reader

// strings.Builder — when you just want a string
var sb strings.Builder
sb.WriteString("line 1\n")
sb.WriteString("line 2\n")
result := sb.String()
```

---

## Regular Expressions

Go's `regexp` package uses RE2 syntax — no lookaheads, no backreferences. This is a deliberate choice: RE2 guarantees O(n) matching time, eliminating catastrophic backtracking.

### Compiling and Using

```go
import "regexp"

// Compile once at startup — cache the compiled regexp
var logPattern = regexp.MustCompile(`^(\d{4}-\d{2}-\d{2}) \[(\w+)\] (.+)$`)

// MustCompile panics on invalid patterns — appropriate for static patterns
// Use Compile (returns error) for patterns from user input

func parseLogLine(line string) (date, level, message string, ok bool) {
    m := logPattern.FindStringSubmatch(line)
    if m == nil {
        return "", "", "", false
    }
    return m[1], m[2], m[3], true
}
```

### Key Functions

```go
re := regexp.MustCompile(`\d+`)

re.MatchString("abc123")                     // true
re.FindString("abc123def")                   // "123"
re.FindAllString("1 and 2 and 3", -1)       // ["1", "2", "3"]
re.FindStringSubmatch("2024-01-15")          // full match + capture groups
re.FindAllStringSubmatch("key=val", -1)      // all matches with groups
re.ReplaceAllString("hello world", "X")      // "X X"
re.ReplaceAllStringFunc("1 2 3", func(s string) string {
    n, _ := strconv.Atoi(s)
    return strconv.Itoa(n * 2)
})  // "2 4 6"
re.Split("a1b2c3", -1)                       // ["a", "b", "c", ""]
```

### The Hot Loop Trap

**Never compile a regexp inside a loop:**

```go
// BAD — compiles a new regexp object on every iteration
for _, line := range lines {
    re := regexp.MustCompile(`\d+`)  // compiled fresh every time
    if re.MatchString(line) { ... }
}

// GOOD — compile once, reuse
re := regexp.MustCompile(`\d+`)
for _, line := range lines {
    if re.MatchString(line) { ... }
}
```

`regexp.Compile` is expensive — it builds a state machine. Compiling in a loop is O(n×compile_cost) when O(n) is all you need.

**Pattern for package-level regexp:**

```go
// Declare at package level — compiled once at program start
var (
    reIP   = regexp.MustCompile(`\b\d{1,3}(\.\d{1,3}){3}\b`)
    reUUID = regexp.MustCompile(`[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}`)
)
```

---

## `text/template` and `html/template`

Go's template packages provide a data-driven text generation engine. They're used for: code generation, email bodies, report rendering, configuration files.

### `text/template` basics

```go
import "text/template"

const tmpl = `
Service: {{.ServiceName}}
Status:  {{.Status}}
Uptime:  {{.UptimeHours}}h
`

type StatusReport struct {
    ServiceName string
    Status      string
    UptimeHours int
}

t := template.Must(template.New("report").Parse(tmpl))
data := StatusReport{ServiceName: "payment-svc", Status: "healthy", UptimeHours: 72}

var sb strings.Builder
t.Execute(&sb, data)
fmt.Println(sb.String())
```

### `html/template` — Critical for web output

`html/template` has the same API as `text/template` but **automatically HTML-escapes values**. Always use `html/template` when rendering HTML.

```go
import "html/template"

// text/template would render this as-is — XSS vulnerability
// html/template escapes < > " & automatically
const htmlTmpl = `<div>Hello, {{.Name}}!</div>`

t := template.Must(template.New("page").Parse(htmlTmpl))
// If data.Name = "<script>alert(1)</script>", html/template renders:
// <div>Hello, &lt;script&gt;alert(1)&lt;/script&gt;!</div>
```

**Common mistake:** Using `text/template` for HTML output. The packages have identical APIs — it's easy to accidentally import the wrong one. If you're rendering HTML, `html/template` is non-negotiable.

---

## Performance: String Building Approaches

Let's be concrete. Here's what each approach costs when building a 10,000-line string:

| Approach | Relative performance | Allocations | When to use |
|---|---|---|---|
| `s += line` in loop | ~500× slower | O(n²) | Never in loops |
| `fmt.Sprintf` accumulate | ~50× slower | Many | Never in loops |
| `strings.Join(slice, sep)` | Fast | 1 | When you already have `[]string` |
| `strings.Builder` | Fastest | 1 (after Grow) | General purpose in loops |
| `bytes.Buffer` | ~Same as Builder | 1 | When you need `io.Writer` |

**The rule:** If you're building a string in a loop, use `strings.Builder`. If you have a slice and want to join it, use `strings.Join`. One-off formatting: `fmt.Sprintf`.

### Conversion Costs

Converting between `string`, `[]byte`, and `[]rune` **always allocates** because strings are immutable and slices are mutable — they can't share memory safely.

```go
s := "hello"
b := []byte(s)   // alloc: copies 5 bytes
r := []rune(s)   // alloc: copies 5 rune values (20 bytes)
s2 := string(b)  // alloc: copies 5 bytes back
```

**In hot paths**, minimize conversions. If you're doing a lot of byte manipulation, work in `[]byte` throughout and convert to `string` once at the end.

One escape hatch: the compiler optimizes away conversions in certain patterns (like `string(b)` used directly in a comparison or map lookup). But don't rely on this — measure if it matters.

---

## JS/TS vs Go: String Mental Model

| Concept | JavaScript | Go |
|---|---|---|
| Encoding | UTF-16 (UCS-2 legacy) | UTF-8 |
| `length` / `len()` | Code units (UTF-16) | Bytes |
| Character access | `s[i]` = character | `s[i]` = byte |
| Iteration | `for...of` decodes code points | `for range` decodes runes |
| Template literals | `` `Hello ${name}` `` | `fmt.Sprintf("Hello %s", name)` |
| Concatenation in loop | Engines optimize; still O(n²) worst case | Use `strings.Builder` |
| Null string | `null` or `undefined` | Not possible — zero value is `""` |
| Substring | `s.slice(1, 3)` | `s[1:3]` (bytes, not chars!) |
| Replace all | `s.replaceAll(a, b)` | `strings.ReplaceAll(s, a, b)` |
| Split | `s.split(",")` | `strings.Split(s, ",")` |
| Regex | `/pattern/flags` built-in | `regexp.MustCompile("pattern")` |

**The most common cross-language bug:** Treating Go byte indices as character indices. `s[1:3]` works perfectly for ASCII but corrupts multibyte characters. Always use `[]rune` slicing when you mean character positions.

---

## Common Pitfalls

### 1. String indexing returns bytes, not characters

```go
s := "café"
fmt.Println(s[3])  // 195 (0xC3) — first byte of é, not 'é'
```

Fix: convert to `[]rune` for character-indexed access.

### 2. `len()` returns bytes, not characters

```go
fmt.Println(len("café"))  // 5, not 4
```

Fix: `len([]rune("café"))` or `utf8.RuneCountInString("café")`.

### 3. String concatenation in a loop

```go
// Each + allocates a new string — O(n²)
for _, word := range words {
    result += word + " "
}
```

Fix: `strings.Builder`.

### 4. Regexp compiled in a loop

```go
for _, line := range lines {
    re := regexp.MustCompile(`\w+`)  // expensive, every iteration
}
```

Fix: declare at package level or before the loop.

### 5. `text/template` for HTML (XSS)

Using the wrong template package for HTML output allows cross-site scripting. Always use `html/template` for HTML.

### 6. `[]byte` to `string` conversion copies

Every `string(b)` allocates. In a hot loop, accumulate as `[]byte` and convert once.

### Your notes
<!-- Add your insights here as you work through the exercises -->

---

## Related Concepts

- [[fundamentals/go/strings-and-text]] — Go-specific deep dive
- [[pitfalls/go-string-byte-index]] — The byte vs character indexing trap
- [[pitfalls/go-regexp-in-loop]] — Compiling regexp in hot loops
- [[fundamentals/strings-and-text]] — Cross-language comparison
