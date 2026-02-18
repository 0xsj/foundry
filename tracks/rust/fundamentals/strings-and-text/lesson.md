# Strings and Text — Rust

## Two String Types: The Foundation

Rust has two primary string types, and the distinction between them is not just a
syntactic detail — it reflects actual differences in ownership, memory layout, and
lifetime. Understanding both is essential for writing idiomatic Rust.

```rust
let owned: String = String::from("hello");   // heap-allocated, owned
let borrowed: &str = "hello";                // borrowed view of some string data
```

`String` owns its contents. `&str` borrows a view into existing string data somewhere —
it could be a string literal in the binary, a slice of a `String`, or a slice of any
other contiguous UTF-8 memory.

### How They Work Under the Hood

`String` is internally `Vec<u8>` with a UTF-8 validity guarantee. Three fields on the stack:

```
Stack:                       Heap:
┌─────────────────┐          ┌───┬───┬───┬───┬───┐
│ ptr ────────────────────── → h │ e │ l │ l │ o │
│ len: 5          │          └───┴───┴───┴───┴───┘
│ capacity: 8     │
└─────────────────┘
```

`&str` is a fat pointer — two fields on the stack:

```
Stack:
┌─────────────────┐
│ ptr ─────────────── → points somewhere into existing UTF-8 memory
│ len: 5          │
└─────────────────┘
```

`&str` has no capacity and allocates nothing. It is purely a read-only view.

### When to Use Which

```rust
// Function parameters: accept &str — more flexible than &String
fn process(input: &str) {       // callers can pass &String, &str, literals, slices
    println!("{}", input);
}

// Owned data in structs: use String
struct Config {
    host: String,               // owns the data, lives as long as Config
    port: u16,
}

// Return type: if you're building something new, return String
fn greeting(name: &str) -> String {
    format!("Hello, {}!", name) // format! always produces a new String
}
```

The rule: accept `&str` in function parameters (you can always get a `&str` from a
`String` via Deref coercion). Store `String` in structs when the struct needs to own
the data. Return `String` when creating new string data.

### Your notes
<!-- -->


---

## Why You Can't Index a String

In JavaScript and Go, you can write `s[0]` on a string. In Rust, this is a compile error:

```rust
let s = String::from("café");
// let ch = s[0];  // ERROR: the type `str` cannot be indexed by `{integer}`
```

This is not a limitation — it's a deliberate design decision that prevents a class of
silent bug. Here's why:

UTF-8 uses variable-width encoding: ASCII characters (a-z, 0-9, most punctuation)
occupy 1 byte. Characters from Latin scripts, Greek, Hebrew, Arabic occupy 2 bytes.
Chinese, Japanese, Korean characters occupy 3 bytes. Emoji and some others occupy 4 bytes.

```
"café" in UTF-8:
┌────┬────┬────┬────┬────┐
│ 99 │ 97 │102 │195 │169 │  (5 bytes total)
└────┴────┴────┴────┴────┘
  c    a    f   [é = two bytes]
```

If `s[2]` returned a `u8`, you'd get `102` (the letter `f`) — which looks fine here,
but for multibyte characters it would return a partial byte of a character. If `s[3]`
returned `u8`, you'd get `195` — one half of `é`, which is meaningless on its own.

If `s[2]` returned a `char`, Rust would need to scan from the start of the string to
find the 3rd character, which is O(n), not the O(1) that array indexing implies.

Rust's answer: don't index strings by integer at all. Be explicit about what you want.

### Your notes
<!-- -->


---

## Iterating Strings: chars() vs bytes()

Rust gives you two explicit iteration modes depending on what "element" means for your
use case:

```rust
let s = "café";

// chars(): iterate over Unicode scalar values (what you usually want)
for ch in s.chars() {
    print!("{} ", ch);   // c a f é
}
println!();
println!("char count: {}", s.chars().count());   // 4

// bytes(): iterate over raw UTF-8 bytes (for binary protocols, parsing)
for b in s.bytes() {
    print!("{} ", b);    // 99 97 102 195 169
}
println!();
println!("byte count: {}", s.len());   // 5 — .len() returns bytes, not chars
```

Note the asymmetry: `.len()` returns **byte length**, not character count. This is a
common source of confusion coming from JavaScript or Go.

```rust
let emoji = "🦀";
println!("len: {}", emoji.len());          // 4 — bytes
println!("chars: {}", emoji.chars().count()); // 1 — characters

// In JavaScript: "🦀".length === 2 (UTF-16 surrogate pairs)
// In Go: len("🦀") === 4 (same as Rust, byte length)
```

If you need the nth character, use `.chars().nth(n)`:

```rust
let s = "hello, world";
let third: Option<char> = s.chars().nth(2);  // Some('l') — O(n), not O(1)
```

Note that `nth(n)` is O(n) — it must iterate from the start. For heavy character-level
indexing, consider storing the string as `Vec<char>` instead.

### Your notes
<!-- -->


---

## String Slicing

You can take a `&str` slice of a `String` or another `&str` using byte-range syntax:

```rust
let s = "hello world";
let hello: &str = &s[0..5];    // bytes 0..5 — "hello"
let world: &str = &s[6..11];   // bytes 6..11 — "world"
```

The slice is a `&str` that borrows from `s`. No allocation.

**The danger:** slicing at a non-character boundary panics at runtime:

```rust
let s = "café";
// let bad = &s[3..4];  // PANIC: byte index 3 is not a char boundary
//                      // because 'é' starts at byte 3 and is 2 bytes long
```

This is the kind of bug you avoid in Go by being careless with `s[i:j]` when the string
contains multibyte content. In Rust, it still panics (not silent corruption), but the
lesson is the same: slice by characters, not bytes, when your data may contain non-ASCII.

Safe patterns:

```rust
let s = "hello world";

// Split by a delimiter — safe, returns &str slices
for word in s.split_whitespace() {
    println!("{}", word);
}

// Find a byte position that IS a char boundary, then slice
if let Some(pos) = s.find("world") {
    let rest = &s[pos..];   // pos is the start of a valid char
    println!("{}", rest);   // "world"
}
```

The `str::find` and `str::split*` methods always return positions at character
boundaries, so slicing by those positions is always safe.

### Your notes
<!-- -->


---

## String Creation

Several ways to create a `String`, each with a slightly different use case:

```rust
// From a string literal
let s = String::from("hello");
let s = "hello".to_string();       // same — both allocate and copy the bytes
let s = "hello".to_owned();        // same — explicit "I want an owned copy"

// Building from parts
let name = "world";
let s = format!("Hello, {}!", name);       // format! never panics, always returns String
let s = format!("{name}:{}", 8080);        // variable capture + positional

// Concatenation
let mut s = String::from("hello");
s.push(' ');                               // append a single char
s.push_str("world");                       // append a &str in place

let s1 = String::from("hello ");
let s2 = String::from("world");
let s3 = s1 + &s2;                        // s1 is MOVED — you cannot use it after this
// let s4 = s1 + &s2;                     // would not compile

// When concatenating many strings, prefer format!
let s = format!("{} {} {}", "hello", "world", "!");  // none are moved
```

The `+` operator is a gotcha: it takes ownership of its left-hand side (`s1` is moved),
then appends `&s2` to the internal buffer. This is efficient — no extra allocation for
`s1` — but it means `s1` cannot be used afterward. Use `format!` when you need to keep
all the inputs alive.

### Your notes
<!-- -->


---

## Conversion: &str ↔ String

The two most common conversions:

```rust
// &str → String (allocates a copy)
let s: String = "hello".to_string();
let s: String = String::from("hello");
let s: String = "hello".to_owned();    // preferred when the intent is "I want my own copy"

// String → &str (borrows, no allocation)
let owned = String::from("hello");
let borrowed: &str = &owned;          // explicit borrow
let borrowed: &str = owned.as_str();  // explicit method

// Deref coercion: &String → &str automatically
fn take_str(s: &str) {}
let owned = String::from("hello");
take_str(&owned);      // &String coerces to &str — no .as_str() needed
```

Deref coercion means you never need to write `take_str(owned.as_str())` — passing
`&owned` works because `String` implements `Deref<Target = str>`. This is why the
recommendation "accept `&str` in function parameters" works so cleanly.

### Your notes
<!-- -->


---

## Formatting: format!, Display, Debug

Rust's formatting is controlled by traits:

```rust
use std::fmt;

// Display — for end users. Used by {} in format strings.
// Debug — for developers. Used by {:?} and {:#?}.

#[derive(Debug)]   // auto-generates a Debug impl
struct LogEntry {
    level: &'static str,
    message: String,
}

// Manual Display impl
impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.level, self.message)
    }
}

let entry = LogEntry { level: "ERROR", message: "disk full".to_string() };

println!("{}", entry);    // [ERROR] disk full         — uses Display
println!("{:?}", entry);  // LogEntry { level: "ERROR", message: "disk full" } — uses Debug
println!("{:#?}", entry); // pretty-printed Debug
```

Key points:
- `#[derive(Debug)]` is almost free — use it on every struct you own.
- `Display` is opt-in and user-facing. Implement it when you have a meaningful text
  representation for the type.
- `write!(f, ...)` in `fmt` uses the same syntax as `format!` but writes into the
  formatter's buffer instead of allocating a new `String`.

### Formatting Specifiers

```rust
// Padding and alignment
println!("{:>10}", "right");    //      right
println!("{:<10}", "left");     // left
println!("{:^10}", "center");   //   center
println!("{:0>5}", 42);         // 00042

// Floats
println!("{:.2}", 3.14159);     // 3.14
println!("{:8.2}", 3.14159);    //     3.14

// Debug with alternate form (pretty-print)
let v = vec![1, 2, 3];
println!("{:#?}", v);           // multi-line pretty format
```

### write! vs format!

When building strings incrementally, `write!` into a `String` avoids intermediate
allocations:

```rust
use std::fmt::Write;

let mut output = String::with_capacity(256);
for level in &["DEBUG", "INFO", "WARN", "ERROR"] {
    write!(output, "{}: ok\n", level).unwrap();   // appends to output, no alloc per call
}
```

`format!` always returns a new `String` with a new allocation. For tight loops building
large strings, `write!` into a pre-allocated `String` is more efficient.

### Your notes
<!-- -->


---

## Parsing: str::parse() and FromStr

```rust
// parse() parses a &str into any type that implements FromStr
let n: i32 = "42".parse().unwrap();
let n: f64 = "3.14".parse().unwrap();
let b: bool = "true".parse().unwrap();

// With the turbofish — when type can't be inferred
let n = "42".parse::<i32>().unwrap();

// parse() returns Result<T, T::Err>
match "abc".parse::<i32>() {
    Ok(n) => println!("parsed: {n}"),
    Err(e) => println!("error: {e}"),
}
```

### Implementing FromStr for Your Types

```rust
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum LogLevel { Debug, Info, Warn, Error }

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO"  => Ok(LogLevel::Info),
            "WARN"  => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            other   => Err(format!("unknown log level: {}", other)),
        }
    }
}

let level: LogLevel = "warn".parse().unwrap();   // LogLevel::Warn
assert_eq!("ERROR".parse::<LogLevel>(), Ok(LogLevel::Error));
```

Implementing `FromStr` automatically gives you `.parse()` on `&str` for your type.
This is idiomatic for any type that has a canonical string representation.

### Your notes
<!-- -->


---

## Cow\<str\>: Avoiding Unnecessary Allocations

`Cow<'a, str>` (Copy-on-Write) is used when you have a string that is *sometimes*
borrowed and *sometimes* owned, and you don't want to pay for an allocation when
borrowing suffices.

```rust
use std::borrow::Cow;

fn normalize(s: &str) -> Cow<str> {
    if s.chars().all(|c| c.is_lowercase()) {
        Cow::Borrowed(s)      // already lowercase — no allocation needed
    } else {
        Cow::Owned(s.to_lowercase())   // allocate a new String
    }
}

let a = normalize("hello");    // Cow::Borrowed — no heap activity
let b = normalize("Hello");    // Cow::Owned — one allocation

// In both cases, you use it the same way:
println!("{}", a);   // hello
println!("{}", b);   // hello
```

The key insight: `Cow<str>` dereferences to `str`, so it works anywhere `&str` does.
The allocation only happens if needed, and the caller's code is the same either way.

### When to Use Cow

```rust
// Good fit: a function that sanitizes strings — most inputs pass through unchanged
fn sanitize_header(value: &str) -> Cow<str> {
    if value.contains('\n') || value.contains('\r') {
        // Rare case: strip bad characters and return a new String
        Cow::Owned(value.replace(['\n', '\r'], ""))
    } else {
        // Common case: valid header, just borrow it
        Cow::Borrowed(value)
    }
}

// Not a good fit: when you always allocate anyway
// Just return String — Cow adds complexity for no benefit
fn build_request_id(prefix: &str, id: u64) -> String {
    format!("{}-{}", prefix, id)
}
```

`Cow` is worth reaching for when: the common path is borrow (no allocation), the
uncommon path needs a modification, and callers treat both cases identically.

### Your notes
<!-- -->


---

## OsString and Path

Not all text in a program is UTF-8. Filenames on Linux can be arbitrary bytes; on
Windows they are UTF-16. The standard library provides specialized types for these.

### OsString / OsStr

```rust
use std::ffi::{OsStr, OsString};

// OsString: owned OS-native string (not guaranteed UTF-8)
// OsStr: borrowed view of OS-native string

// From a regular &str (always valid — UTF-8 is a subset of OS strings)
let os: OsString = OsString::from("config.yaml");

// Converting to &str (may fail if not valid UTF-8)
let path_str: Option<&str> = os.to_str();

// Comparing: use OsStr directly, not &str
let arg: &OsStr = OsStr::new("--verbose");
if arg == OsStr::new("--verbose") {
    println!("verbose mode");
}
```

`OsStr::new` from a string literal always works. The reverse — converting `OsStr` to
`&str` — may fail for filenames containing non-UTF-8 bytes (uncommon on macOS/Linux,
possible on Windows).

### Path / PathBuf

```rust
use std::path::{Path, PathBuf};

// PathBuf: owned path (builds on OsString)
// Path: borrowed path slice (builds on OsStr)

let config_dir = PathBuf::from("/etc/myapp");
let config_file = config_dir.join("config.yaml");   // /etc/myapp/config.yaml

// Path methods
println!("{:?}", config_file.extension());    // Some("yaml")
println!("{:?}", config_file.file_name());    // Some("config.yaml")
println!("{:?}", config_file.parent());       // Some("/etc/myapp")

// Converting to &str — may return None for non-UTF-8 paths
if let Some(s) = config_file.to_str() {
    println!("path: {}", s);
}

// For display without unwrapping
println!("{}", config_file.display());   // always works, even for non-UTF-8 paths
```

Use `Path`/`PathBuf` for any path you'll pass to file system operations. Do not use
`String` for paths — platform differences will catch you eventually.

### Your notes
<!-- -->


---

## Common String Operations

Quick reference for patterns you'll reach for repeatedly:

```rust
let s = "  Hello, World!  ";

// Searching and checking
s.contains("World")              // true
s.starts_with("  Hello")         // true
s.ends_with("!  ")               // true
s.find("World")                  // Some(9) — byte index

// Trimming
s.trim()                          // "Hello, World!"
s.trim_start()                    // "Hello, World!  "
s.trim_end()                      // "  Hello, World!"

// Case conversion
"hello".to_uppercase()            // "HELLO"
"HELLO".to_lowercase()            // "hello"

// Splitting
"a,b,c".split(',').collect::<Vec<_>>()            // ["a", "b", "c"]
"  hello   world  ".split_whitespace().collect::<Vec<_>>() // ["hello", "world"]
"a::b::c".splitn(2, "::").collect::<Vec<_>>()     // ["a", "b::c"] — max 2 parts

// Replacing
"hello world".replace("world", "Rust")           // "hello Rust"
"aabbaab".replacen("aa", "X", 1)                 // "Xbbaab" — replace first n

// Joining
let parts = ["a", "b", "c"];
parts.join(", ")                  // "a, b, c"

// Repeating
"ha".repeat(3)                    // "hahaha"

// Char operations
"hello".chars().count()           // 5
"hello".chars().rev().collect::<String>()  // "olleh"
```

### Your notes
<!-- -->


---

## Comparison to Go

| Feature | Go | Rust |
|---|---|---|
| String type | `string` | `String` (owned) |
| String view | `string` (all strings are copies) | `&str` (borrowed) |
| Byte view | `[]byte` | `&[u8]` |
| Length | `len(s)` — bytes | `s.len()` — bytes |
| Char count | `utf8.RuneCountInString(s)` | `s.chars().count()` |
| Indexing | `s[i]` returns `byte`, `s[i:j]` safe only if on char boundary | not allowed; must use `.chars()` or byte-range |
| Char iteration | `for _, ch := range s` — iterates runes | `for ch in s.chars()` |
| Byte iteration | `for i := 0; i < len(s); i++ { b := s[i] }` | `for b in s.bytes()` |
| Concatenation | `s1 + s2` or `strings.Builder` | `format!()` or `push_str` |
| Formatting | `fmt.Sprintf(...)` | `format!(...)` |
| Parsing int | `strconv.Atoi(s)` | `s.parse::<i32>()` |
| String builder | `strings.Builder` | `String` + `write!` / `push_str` |
| UTF-8 validity | Assumed, not enforced | Guaranteed by type system |
| Nil/empty | `""` is falsy in comparisons, `s == ""` | `s.is_empty()` |

Key insight: Go's `string` type is an immutable byte slice — there's no separate "owned
vs borrowed" distinction in the type system. Every string assignment copies the header
(pointer + length), not the bytes. In Rust, the ownership split (`String` vs `&str`) is
explicit and enforced at compile time. You can't accidentally hold a dangling string view
in Rust the way you can in C or C++.

Both Go and Rust guarantee UTF-8 for their primary string types. The difference is that
Go's `string` can technically hold arbitrary bytes (the encoding is unchecked at the
type level), while Rust's `String` guarantees valid UTF-8 at the API boundary — you
cannot construct an invalid `String` using safe code.

### Your notes
<!-- -->
