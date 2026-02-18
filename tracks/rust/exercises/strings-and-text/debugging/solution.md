# Solution: Log Processor Bugs

## Bug 1: String slice panic at non-char boundary in `extract_level`

### Root Cause

```rust
let start = line.find('[')?;
let end = line.find(']')?;
Some(&line[start + 1..end])
```

`str::find()` returns a byte index. `start + 1` adds one byte to the byte position of
`[`. For ASCII-only input this is always correct — `[` is a 1-byte character so
`start + 1` lands on the next character boundary. But for input containing multibyte
characters before `[`, if the character immediately after `[` happens to straddle a
non-aligned byte boundary, the `+1` offset can still land mid-character.

More subtly: the test here exposes that `start + 1` is only safe if `[` is always
followed by an ASCII character. In this log format the level name is always ASCII, so
`start + 1` actually works in practice. However the general pattern (`byte_offset + 1`)
is dangerous and the correct idiom is to advance past `[` using character-aware methods.

### Fix

```rust
fn extract_level(line: &str) -> Option<&str> {
    let start = line.find('[')?;
    let end = line.find(']')?;
    // Advance past '[' using a character-safe method
    let after_bracket = start + '['.len_utf8();
    Some(&line[after_bracket..end])
}
```

`char::len_utf8()` returns the byte length of the character, so `start + '['.len_utf8()`
is the byte position immediately after `[`. Since `[` is ASCII (1 byte),
`'['.len_utf8() == 1`, and this produces the same result as `start + 1` — but the intent
is explicit and the pattern generalizes correctly to any character.

The idiomatic way to advance past a known delimiter without arithmetic:

```rust
fn extract_level(line: &str) -> Option<&str> {
    let inner = line.strip_prefix(|_| true)   // not ideal
        .and_then(|_| {                        // ... or just use find + split
            let after_open = line[line.find('[')?+ 1..].as_str();
            after_open.split_once(']').map(|(level, _)| level)
        });
    inner
}
```

Or more cleanly:

```rust
fn extract_level(line: &str) -> Option<&str> {
    let start = line.find('[')?;
    let from_bracket = &line[start + 1..];   // '[' is ASCII — safe
    let (level, _) = from_bracket.split_once(']')?;
    Some(level)
}
```

`split_once(']')` splits at the first `]` and returns `(level, rest)`. This eliminates
the need to find `]` separately and ensures correct handling of the boundary.

### Lesson

- `str::find()` returns a byte index into UTF-8 encoded text.
- Slicing at `byte_index + n` is only safe if you know the byte offset is at a character
  boundary. Use `char::len_utf8()` to advance past a known character, or use
  `split_once` / `strip_prefix` / `strip_suffix` which handle boundaries automatically.
- In Go, `s[i:j]` on a string is equivalent to slicing bytes — the same issue exists
  but Go does not panic (you silently get partial bytes). Rust panics at runtime, which
  is preferable: you find the bug.

---

## Bug 2: `&String` where `&str` is expected in `line_contains_service`

### Root Cause

```rust
fn line_contains_service(line: &str, service: &String) -> bool {
    line.contains(service.as_str())
}
```

The parameter type `&String` requires callers to have a `String` they can borrow from.
String literals (`"auth-service"`) are `&str`, not `&String`. The test calls:

```rust
line_contains_service(line, "auth-service")  // "auth-service" is &str, not &String
```

This does not compile — the types do not match.

### Fix

```rust
fn line_contains_service(line: &str, service: &str) -> bool {
    line.contains(service)
}
```

Change `service: &String` to `service: &str`. A `&String` coerces to `&str` via Deref
automatically, so all existing callers that pass `&String` still work. Callers that
previously had to write `String::from("auth-service")` or `"auth-service".to_string()`
can now pass a literal directly. The body also simplifies: `.contains(service.as_str())`
becomes `.contains(service)` since `service` is already `&str`.

### Lesson

- Accept `&str` in function parameters, not `&String`. `&String` is more restrictive
  than necessary — it requires callers to have an owned `String`. `&str` accepts string
  literals, `&String` (via Deref coercion), and any other `&str` slice.
- The rule: *if a function only needs to read the string, accept `&str`*.
- In Go, there is no equivalent distinction — all strings are value types. In Rust,
  the `String` / `&str` split is explicit, and accepting `&str` is always the more
  flexible and idiomatic choice for parameters.

---

## Bug 3: Unnecessary `.to_string()` in `count_keyword_hits`

### Root Cause

```rust
for keyword in keywords {
    let kw_owned: String = keyword.to_string();  // allocates a String every iteration
    if lower.contains(&kw_owned) {
        hits += 1;
    }
}
```

`keyword` is already `&&str` (a reference to a `&str` from the `keywords` slice).
`keyword.to_string()` creates a new heap-allocated `String` — once per keyword per line.
For 500 lines and 500 keywords this is 250,000 unnecessary allocations.

`str::contains()` already accepts `&str` as its pattern argument. `*keyword` dereferences
`&&str` to `&str`, which is all that's needed.

### Fix

```rust
fn count_keyword_hits(lines: &[&str], keywords: &[&str]) -> usize {
    let mut hits = 0usize;
    for line in lines {
        let lower = line.to_lowercase();   // this allocation IS needed — we need lowercase
        for keyword in keywords {
            if lower.contains(*keyword) {  // *keyword: &&str → &str, no allocation
                hits += 1;
            }
        }
    }
    hits
}
```

`*keyword` dereferences `&&str` to `&str`. `str::contains` accepts `&str` as a `Pattern`
directly, so no owned `String` is needed.

### Lesson

- `.to_string()` always allocates. Before calling it, ask: "does the callee actually
  need an owned `String`, or will `&str` suffice?"
- `str::contains`, `str::find`, `str::starts_with`, etc. all accept `&str` as their
  pattern argument. You never need to convert to `String` to use them.
- This class of allocation often appears when developers coming from languages without
  the `String` / `&str` distinction default to `to_string()` defensively. In Rust, think
  "what ownership does this operation actually require?" before converting.

---

## Bug 4: `to_string_lossy()` silently masks non-UTF-8 in `is_log_extension`

### Root Cause

```rust
fn is_log_extension(path: &std::path::Path) -> bool {
    match path.extension() {
        None      => false,
        Some(ext) => ext.to_string_lossy() == "log",
    }
}
```

`OsStr::to_string_lossy()` replaces invalid UTF-8 byte sequences with the Unicode
replacement character U+FFFD (`\u{fffd}`). For a path whose extension bytes are
exactly the ASCII bytes for `"log"`, this works correctly. But for an extension whose
bytes *include* non-UTF-8 bytes, `to_string_lossy()` silently substitutes replacement
characters and the comparison becomes incorrect.

More importantly, the design intent of comparing a file extension is:
- If the extension is valid UTF-8 AND equals `"log"` → `true`
- Anything else → `false`

`to_string_lossy()` cannot express this: it converts `None` UTF-8 sequences to
`"<?>"`-style garbage and then compares. The result is unpredictable for non-UTF-8 paths.

### Fix

```rust
fn is_log_extension(path: &std::path::Path) -> bool {
    match path.extension() {
        None      => false,
        Some(ext) => ext.to_str() == Some("log"),
    }
}
```

`OsStr::to_str()` returns `Option<&str>`:
- `Some(s)` if the bytes are valid UTF-8
- `None` if they are not

Comparing `Option<&str>` to `Some("log")` is `true` only when the extension is valid
UTF-8 and equals `"log"`. Non-UTF-8 extensions correctly return `None != Some("log")` →
`false`.

### Lesson

- `to_string_lossy()` exists for display purposes — showing a path to a user even if it
  contains non-UTF-8 bytes. It is not for comparison: the replacement characters it
  inserts make equality checks unreliable.
- For logic that depends on the string value, use `.to_str()` and handle the `None` case:
  ```rust
  ext.to_str() == Some("expected")   // clean, explicit
  ```
- `to_string_lossy()` allocates when replacement is needed (returns `Cow::Owned`). For
  pure ASCII extensions, it returns `Cow::Borrowed` — but you still shouldn't use it for
  equality comparisons.

---

## Summary

| Bug | Category | Concept | Prevention |
|-----|----------|---------|------------|
| `start + 1` on byte index | Dangerous pattern | UTF-8 slice boundaries | Use `char::len_utf8()`, `split_once`, or `strip_prefix` to advance past chars |
| `&String` parameter | Compile error | `String` vs `&str` | Accept `&str` in function parameters — it's always more general |
| `.to_string()` in loop | Unnecessary allocation | Avoiding allocations | Ask "does this callee need ownership?" before calling `.to_string()` |
| `to_string_lossy()` for comparison | Silent wrong result | OsStr → &str conversion | Use `.to_str() == Some(...)` for equality checks; `to_string_lossy()` is for display only |

## Related Pitfalls

- [[rust-str-slice-boundary]] — slicing str at non-char boundaries panics
- [[rust-string-vs-str-params]] — accept &str in parameters, not &String
- [[rust-tostring-in-loops]] — to_string() allocates; avoid in hot paths
- [[rust-osstr-to-str]] — use .to_str() for OsStr comparisons, not .to_string_lossy()
