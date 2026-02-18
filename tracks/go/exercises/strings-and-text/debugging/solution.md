# Debugging Solutions

## Bug 1: String index gives a byte, not a rune

**Root cause:** `name[0]` returns the first **byte** of the string as a `uint8`. For a multibyte character like `å` (U+00E5, encoded as `0xC3 0xA5` in UTF-8), `name[0]` returns `0xC3` — the high byte of the two-byte sequence. Casting that byte to `rune` gives `U+00C3` (Ã), not the intended character. `unicode.ToUpper(0xC3)` produces `0xC3` again (it's already "uppercase" from the BMP tables), resulting in a corrupted first character.

**Fix:** Decode the first rune using `utf8.DecodeRuneInString` (or convert to `[]rune` and index by character):

```go
func CapitalizeDisplayName(name string) string {
    if name == "" {
        return name
    }
    // DecodeRuneInString returns the first rune and its byte width
    r, size := utf8.DecodeRuneInString(name)
    return string(unicode.ToUpper(r)) + name[size:]
}
```

Or equivalently:
```go
runes := []rune(name)
runes[0] = unicode.ToUpper(runes[0])
return string(runes)
```

The `[]rune` approach is slightly less efficient (allocates the whole rune slice) but clearer if you're doing further character manipulation.

**Lesson:** `s[i]` in Go always returns a `byte`. To work with characters, use `for range` (which yields `rune` values) or convert to `[]rune`. This is one of the most common Go string bugs.

**Related pitfall:** [[pitfalls/go-string-byte-index]]

---

## Bug 2: String concatenation in a loop — O(n²) allocations

**Root cause:** Go strings are immutable. Every `result += ","` creates a new string, copies the old `result`, and appends the new character. In a loop over N items, you perform roughly N copies of 0, 1, 2, ..., N characters — totaling O(N²) work and O(N²) allocations.

For 10,000 labels at ~15 chars each, this is ~750 million characters of copying.

**Fix:** Use `strings.Builder`, which maintains a `[]byte` internally and converts to string exactly once:

```go
func BuildMetricLabels(labels map[string]string) string {
    keys := make([]string, 0, len(labels))
    for k := range labels {
        keys = append(keys, k)
    }
    sort.Strings(keys)

    var b strings.Builder
    b.Grow(len(labels) * 20) // rough capacity hint
    for i, k := range keys {
        if i > 0 {
            b.WriteByte(',')
        }
        b.WriteString(k)
        b.WriteByte('=')
        b.WriteString(labels[k])
    }
    return b.String()
}
```

`Grow` is optional but avoids internal resizing. Alternatively, `strings.Join` works if you build the `"k=v"` strings first into a `[]string` — but Builder is more memory-efficient since it doesn't allocate intermediate strings.

**Lesson:** Any `string +=` inside a loop is O(n²). Always reach for `strings.Builder` for loop-based string construction. The IDE/linter can flag this: `govet`, `gocritic`, or `noctx` rules will catch common variants.

---

## Bug 3: regexp.MustCompile in a hot loop

**Root cause:** `regexp.MustCompile` builds a finite automaton (NFA/DFA) from the pattern string. This is significantly more expensive than a simple string comparison — it involves parsing the regex syntax, building state machines, and allocating internal data structures. Calling it inside a loop of 50,000 iterations compiles the same pattern 50,000 times.

**Fix:** Move the `regexp.MustCompile` call to package level:

```go
// Compiled once at program start. Safe for concurrent use.
var reError = regexp.MustCompile(`\bERROR\b`)

func FilterErrorLines(lines []string) []string {
    result := make([]string, 0)
    for _, line := range lines {
        if reError.MatchString(line) {
            result = append(result, line)
        }
    }
    return result
}
```

Package-level `*regexp.Regexp` values are safe for concurrent use without synchronization.

If the pattern is only needed in one function and that function is rarely called, a `sync.Once` or function-scoped `var once sync.Once; once.Do(func() { re = ... })` also works, but package-level is simpler and idiomatic.

**Lesson:** Static regexp patterns belong at package level. The rule: if a `MustCompile` call is inside a function that gets called more than once, it should be hoisted out. Tools like `go-staticcheck` (SA6000) and `revive` can detect this.

---

## Bug 4: Unnecessary []byte → string conversion

**Root cause:** `strings.TrimSpace(string(raw))` involves two allocations:
1. `string(raw)` — copies all bytes from `raw` into a new string
2. `strings.TrimSpace` returns a sub-slice of the allocated string (no allocation, just pointer arithmetic)
3. `fmt.Sprintf("%s", trimmed)` — allocates another string (copies the trimmed content again)

The `bytes` package has `bytes.TrimSpace` which operates on `[]byte` directly and returns a sub-slice of the original — zero copies, zero allocations:

```go
import "bytes"

func NormalizeHeaderValue(raw []byte) string {
    return string(bytes.TrimSpace(raw))
}
```

The single `string(...)` at the end is unavoidable (the return type is `string`) but it's one allocation instead of three.

**Lesson:** The `bytes` package mirrors the `strings` package for `[]byte`. When you already have `[]byte` from I/O (file reads, network reads, HTTP bodies), work in `[]byte` and convert to `string` once at the end. The `string(b)` conversion before a `bytes` operation is a common signal that the wrong package was chosen.

See also: the compiler eliminates the `string(b)` allocation when `b` is used in a map lookup or comparison — but not in function calls.

---

## Summary

| # | Function | Bug Category | Concept |
|---|---|---|---|
| 1 | `CapitalizeDisplayName` | Byte vs rune indexing | `s[i]` returns a byte; use `[]rune` or `utf8.DecodeRuneInString` for characters |
| 2 | `BuildMetricLabels` | O(n²) string concatenation | `strings.Builder` for loop-based construction |
| 3 | `FilterErrorLines` | Regexp compiled in hot loop | Package-level `var re = regexp.MustCompile(...)` |
| 4 | `NormalizeHeaderValue` | Unnecessary string/[]byte conversion | Use `bytes.TrimSpace` when input is already `[]byte` |
