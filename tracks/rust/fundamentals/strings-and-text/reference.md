# Rust Reference — Strings and Text

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Standard Library](https://doc.rust-lang.org/std/), and
> [The Rustonomicon](https://doc.rust-lang.org/nomicon/) for string-related topics.
> Covers: `str`, `String`, `&str`, formatting traits, `Cow<str>`, `OsStr`, `Path`.

---

## Primitive Type: str

Source: [std::str](https://doc.rust-lang.org/std/str/index.html)

`str` is the most primitive string type. It is almost always seen as the borrowed slice
form `&str`. It is a sequence of UTF-8 bytes. Rust guarantees that `str` is always valid
UTF-8.

`str` is an *unsized* type (the compiler does not know its size at compile time), which
is why you will almost always work with `&str` rather than a bare `str`.

### Properties

- Always valid UTF-8 (enforced by the type system and safe APIs)
- Borrowed — does not own the underlying bytes
- Passed by reference: `&str` is a fat pointer `(ptr: *const u8, len: usize)`
- Can point to string literal data in the binary (has `'static` lifetime), to a `String`
  on the heap, or to any other contiguous UTF-8 memory

### Creation

```rust
// String literals are &'static str
let s: &str = "hello";

// Slicing an existing String or &str
let owned = String::from("hello world");
let slice: &str = &owned[0..5];   // borrows from owned

// From raw bytes (unsafe, requires valid UTF-8)
let bytes = b"hello";
let s = std::str::from_utf8(bytes).unwrap();  // validates UTF-8
```

### Core Methods on &str

Source: [std::str::pattern](https://doc.rust-lang.org/std/str/index.html#methods)

| Method | Signature | Notes |
|---|---|---|
| `len` | `(&self) -> usize` | Byte length, not char count |
| `is_empty` | `(&self) -> bool` | True if `len == 0` |
| `contains` | `<P: Pattern>(&self, pat: P) -> bool` | Substring or char check |
| `starts_with` | `<P: Pattern>(&self, pat: P) -> bool` | Prefix check |
| `ends_with` | `<P: Pattern>(&self, pat: P) -> bool` | Suffix check |
| `find` | `<P: Pattern>(&self, pat: P) -> Option<usize>` | First match byte index |
| `rfind` | `<P: Pattern>(&self, pat: P) -> Option<usize>` | Last match byte index |
| `split` | `<P: Pattern>(&self, pat: P) -> Split<P>` | Iterator of subslices |
| `splitn` | `<P: Pattern>(&self, n: usize, pat: P) -> SplitN<P>` | At most n subslices |
| `trim` | `(&self) -> &str` | Strip leading/trailing whitespace |
| `trim_start` | `(&self) -> &str` | Strip leading whitespace |
| `trim_end` | `(&self) -> &str` | Strip trailing whitespace |
| `to_uppercase` | `(&self) -> String` | Allocates new String |
| `to_lowercase` | `(&self) -> String` | Allocates new String |
| `replace` | `<P: Pattern>(&self, from: P, to: &str) -> String` | Allocates |
| `replacen` | `<P: Pattern>(&self, from: P, to: &str, n: usize) -> String` | First n |
| `chars` | `(&self) -> Chars` | Iterator of `char` (Unicode scalar values) |
| `bytes` | `(&self) -> Bytes` | Iterator of `u8` (raw UTF-8 bytes) |
| `char_indices` | `(&self) -> CharIndices` | Iterator of `(usize, char)` pairs |
| `parse` | `<F: FromStr>(&self) -> Result<F, F::Err>` | Parses to any FromStr type |
| `lines` | `(&self) -> Lines` | Iterator of lines, stripping `\n` and `\r\n` |
| `split_whitespace` | `(&self) -> SplitWhitespace` | Split on any Unicode whitespace |
| `repeat` | `(&self, n: usize) -> String` | Concatenates n copies |

---

## Struct: String

Source: [std::string::String](https://doc.rust-lang.org/std/string/struct.String.html)

A growable, heap-allocated, UTF-8 encoded string type.

**Internal representation:** `String` is a wrapper around `Vec<u8>` that maintains the
UTF-8 validity invariant. Its memory layout is identical to `Vec<u8>`:

```
String { ptr: NonNull<u8>, length: usize, capacity: usize }
```

### Creation

```rust
String::new()                    // empty string, no heap allocation
String::with_capacity(n)         // empty, reserved for n bytes
String::from("hello")            // allocate + copy from &str
"hello".to_string()              // same as String::from — by ToString trait
"hello".to_owned()               // same — by ToOwned trait
format!("hello {}", name)        // format macro, always returns String
```

### Core Methods

| Method | Signature | Notes |
|---|---|---|
| `push` | `(&mut self, ch: char)` | Appends single char |
| `push_str` | `(&mut self, s: &str)` | Appends string slice |
| `pop` | `(&mut self) -> Option<char>` | Removes and returns last char |
| `insert` | `(&mut self, idx: usize, ch: char)` | Inserts char at byte index |
| `insert_str` | `(&mut self, idx: usize, s: &str)` | Inserts &str at byte index |
| `remove` | `(&mut self, idx: usize) -> char` | Removes char at byte index |
| `len` | `(&self) -> usize` | Byte length |
| `capacity` | `(&self) -> usize` | Allocated byte capacity |
| `is_empty` | `(&self) -> bool` | True if len == 0 |
| `clear` | `(&mut self)` | Removes all characters |
| `truncate` | `(&mut self, new_len: usize)` | Shortens to new_len bytes |
| `as_str` | `(&self) -> &str` | Borrows as &str |
| `as_bytes` | `(&self) -> &[u8]` | Borrows as byte slice |
| `as_mut_str` | `(&mut self) -> &mut str` | Mutable borrow |
| `into_bytes` | `(self) -> Vec<u8>` | Consume, take ownership of bytes |
| `retain` | `(&mut self, f: impl FnMut(char) -> bool)` | Keep only chars matching predicate |
| `drain` | `(&mut self, range: R) -> Drain` | Remove byte range, return iterator |

### Deref to str

`String` implements `Deref<Target = str>`. This means:
- `&String` coerces to `&str` automatically
- All `str` methods are accessible on `String` directly
- Functions that accept `&str` work with `&String` via coercion

```rust
let s = String::from("hello");
let _: &str = &s;          // coercion
let _: &str = s.as_str();  // explicit
let _ = s.len();           // str method called directly on String
```

---

## The `+` Operator on Strings

```rust
let s1 = String::from("hello ");
let s2 = String::from("world");
let s3 = s1 + &s2;   // s1 is moved; s2 is borrowed
```

The `+` operator calls `fn add(self, rhs: &str) -> String`. It takes ownership of `s1`,
appends the bytes of `s2` (which is auto-deref'd from `&String` to `&str`), and returns
the result. This avoids allocating a new buffer when `s1` has sufficient capacity.

After `s1 + &s2`, `s1` is moved and cannot be used.

---

## UTF-8 Encoding Details

Source: [The Rustonomicon — Working with Unsafe](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)

UTF-8 encodes Unicode scalar values (code points U+0000 to U+D7FF and U+E000 to
U+10FFFF) using 1 to 4 bytes per character:

| Code point range | Bytes | Pattern |
|---|---|---|
| U+0000 – U+007F | 1 | `0xxxxxxx` |
| U+0080 – U+07FF | 2 | `110xxxxx 10xxxxxx` |
| U+0800 – U+FFFF | 3 | `1110xxxx 10xxxxxx 10xxxxxx` |
| U+10000 – U+10FFFF | 4 | `11110xxx 10xxxxxx 10xxxxxx 10xxxxxx` |

Consequence: `&str[i..j]` panics if `i` or `j` is not on a character boundary (i.e., if
either byte is a continuation byte starting with `10`).

`String::len()` returns byte count. `str::chars().count()` returns Unicode scalar value
count. Neither counts *grapheme clusters* — a user-visible character like `é` can be
either one code point (U+00E9 LATIN SMALL LETTER E WITH ACUTE) or two (U+0065 LATIN
SMALL LETTER E + U+0301 COMBINING ACUTE ACCENT). The `unicode-segmentation` crate
provides grapheme cluster iteration.

---

## Formatting Traits

Source: [std::fmt](https://doc.rust-lang.org/std/fmt/index.html)

| Trait | Specifier | Purpose |
|---|---|---|
| `Display` | `{}` | User-facing string representation |
| `Debug` | `{:?}` / `{:#?}` | Developer-facing, auto-derivable |
| `LowerHex` | `{:x}` | Lowercase hex |
| `UpperHex` | `{:X}` | Uppercase hex |
| `Binary` | `{:b}` | Binary |
| `Octal` | `{:o}` | Octal |
| `Pointer` | `{:p}` | Memory address |
| `LowerExp` | `{:e}` | Scientific notation |

### Display Implementation

```rust
use std::fmt;

impl fmt::Display for MyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "...")
    }
}
```

`fmt::Result` is `Result<(), fmt::Error>`. The `write!` macro writes into the formatter
and returns the result. `fmt::Error` carries no message — formatting errors are
considered exceptional; if write to formatter fails, something is catastrophically wrong.

### Format String Syntax

```
format_spec := [[fill]align][sign][#][0][width]['.' precision][type]
fill        := character
align       := '<' | '^' | '>'
sign        := '+' | '-'
width       := integer | variable ('$' suffix)
precision   := integer | variable ('$' suffix) | '*'
type        := '' | '?' | 'x?' | 'X?' | identifier
```

Examples:

```rust
format!("{:>10}")         // right-align in 10-char field
format!("{:0>5}", 42)     // zero-pad to width 5: "00042"
format!("{:.3}", 3.14)    // 3 decimal places
format!("{:08.2}", 3.14)  // "00003.14"
format!("{:+}", 42)       // force sign: "+42"
```

---

## Trait: FromStr

Source: [std::str::FromStr](https://doc.rust-lang.org/std/str/trait.FromStr.html)

```rust
pub trait FromStr: Sized {
    type Err;
    fn from_str(s: &str) -> Result<Self, Self::Err>;
}
```

Implementing `FromStr` for a type enables `.parse::<T>()` on any `&str`.

Standard library types that implement `FromStr`: all numeric primitives, `bool`, `char`,
`String`, `IpAddr`, `SocketAddr`, `PathBuf`, and others.

---

## Enum: Cow\<'a, B\>

Source: [std::borrow::Cow](https://doc.rust-lang.org/std/borrow/enum.Cow.html)

```rust
pub enum Cow<'a, B: ?Sized + 'a>
where
    B: ToOwned,
{
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```

`Cow<'a, str>` is either a `Borrowed(&'a str)` or an `Owned(String)`.

`Cow<str>` implements `Deref<Target = str>`, so it can be used anywhere `&str` is
accepted without pattern matching.

```rust
use std::borrow::Cow;

fn maybe_escape(s: &str) -> Cow<str> {
    if s.contains('"') {
        Cow::Owned(s.replace('"', "\\\""))
    } else {
        Cow::Borrowed(s)
    }
}
```

`Cow::into_owned()` converts to `String` (clones if `Borrowed`, unwraps if `Owned`).

---

## OsStr and OsString

Source: [std::ffi::OsStr](https://doc.rust-lang.org/std/ffi/struct.OsStr.html)

`OsStr` and `OsString` are for platform-native strings. They are not guaranteed to be
valid UTF-8. On Unix, they are arbitrary byte sequences. On Windows, they are
WTF-8 (a superset of UTF-8 that can represent unpaired surrogates from Windows UTF-16).

| Type | Owned? | Notes |
|---|---|---|
| `OsStr` | No | Borrowed view, analogous to `str` |
| `OsString` | Yes | Owned, analogous to `String` |
| `str` | No | UTF-8 guaranteed |
| `String` | Yes | UTF-8 guaranteed |

Conversion:
```rust
use std::ffi::{OsStr, OsString};

let s: &OsStr = OsStr::new("hello");    // from &str, always succeeds
let s: OsString = OsString::from("hello");

// OsStr → &str (may fail)
let maybe: Option<&str> = s.to_str();

// &str → OsStr (always succeeds — UTF-8 ⊆ OS strings)
let os: &OsStr = OsStr::new("hello");
```

---

## Path and PathBuf

Source: [std::path::Path](https://doc.rust-lang.org/std/path/struct.Path.html)

`Path` and `PathBuf` are wrappers around `OsStr`/`OsString` that provide
platform-aware path manipulation.

| Type | Owned? | Notes |
|---|---|---|
| `Path` | No | Borrowed path slice |
| `PathBuf` | Yes | Owned, growable path |

```rust
use std::path::{Path, PathBuf};
```

### Key Methods on Path

| Method | Return | Notes |
|---|---|---|
| `file_name` | `Option<&OsStr>` | Last component (filename + ext) |
| `file_stem` | `Option<&OsStr>` | Filename without extension |
| `extension` | `Option<&OsStr>` | Extension without leading dot |
| `parent` | `Option<&Path>` | Everything except the last component |
| `is_absolute` | `bool` | Whether path starts from filesystem root |
| `is_relative` | `bool` | Opposite of `is_absolute` |
| `exists` | `bool` | Filesystem existence check (I/O) |
| `is_file` | `bool` | Exists and is a regular file |
| `is_dir` | `bool` | Exists and is a directory |
| `to_str` | `Option<&str>` | May fail for non-UTF-8 paths |
| `display` | `Display` | Safe display even for non-UTF-8 paths |
| `join` | `PathBuf` | Appends component to path |
| `components` | `Components` | Iterator over path components |

### PathBuf-only Methods

| Method | Notes |
|---|---|
| `push(P)` | Extends path in place |
| `pop()` | Removes last component |
| `set_extension(s)` | Changes extension |
| `set_file_name(s)` | Changes filename |
| `with_extension(s)` | Returns new PathBuf with changed extension |

---

## Common String Conversion Summary

```rust
// Between &str and String
&str  → String:   s.to_string() | String::from(s) | s.to_owned()
String → &str:    &s | s.as_str()

// To bytes
&str   → &[u8]:   s.as_bytes()
String → Vec<u8>: s.into_bytes()

// From bytes (validates UTF-8)
&[u8]  → &str:    std::str::from_utf8(bytes)  // returns Result
Vec<u8> → String: String::from_utf8(bytes)    // returns Result

// OS strings
&str    → OsString: OsString::from(s)
OsStr   → &str:     os.to_str()  // Option<&str>
String  → PathBuf:  PathBuf::from(s)
Path    → &str:     p.to_str()   // Option<&str>
Path    → display:  p.display()  // always works
```

---

## References

- [The Rust Programming Language — ch. 8.2: Strings](https://doc.rust-lang.org/book/ch08-02-strings.html)
- [std::str](https://doc.rust-lang.org/std/str/index.html)
- [std::string::String](https://doc.rust-lang.org/std/string/struct.String.html)
- [std::fmt](https://doc.rust-lang.org/std/fmt/index.html)
- [std::borrow::Cow](https://doc.rust-lang.org/std/borrow/enum.Cow.html)
- [std::ffi::OsStr](https://doc.rust-lang.org/std/ffi/struct.OsStr.html)
- [std::path::Path](https://doc.rust-lang.org/std/path/struct.Path.html)
- [The Rustonomicon — Text and Strings](https://doc.rust-lang.org/nomicon/)
