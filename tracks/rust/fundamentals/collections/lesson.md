# Collections — Rust

## How Collections Work Under the Hood

### Ownership Is the Whole Story

Before touching any collection API, internalize this: every collection in Rust is just a
struct that owns heap-allocated memory. When the collection goes out of scope, the
allocator is called to free that memory and every element inside it is dropped.

```rust
{
    let mut v: Vec<String> = Vec::new();
    v.push(String::from("hello"));
    v.push(String::from("world"));
}   // v is dropped here: each String inside is dropped, then the backing array is freed
```

No garbage collector, no reference counting (for plain collections). One owner, one
lifetime, deterministic cleanup.

This design has one important consequence: the borrow checker will stop you from doing
things that look innocent in other languages. The most common pattern to hit:

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];     // immutable borrow of v
v.push(4);             // ERROR: cannot borrow `v` as mutable because it is also borrowed as immutable
println!("{first}");
```

Why does this fail? Because `push` may need to reallocate the backing array. If it does,
`first` would point to freed memory — a dangling reference. The borrow checker catches
this at compile time. No segfaults, no undefined behavior.

### Your notes
<!-- -->


---

## Vec\<T\>

### The Basics

`Vec<T>` is Rust's growable array. Three fields on the stack: a pointer to heap memory,
a length (how many elements are initialized), and a capacity (how many the allocation can
hold).

```
Stack:                   Heap:
┌─────────────────┐      ┌────┬────┬────┬────┬────┐
│ ptr ───────────────────→│ 1  │ 2  │ 3  │ ?? │ ?? │
│ len: 3          │      └────┴────┴────┴────┴────┘
│ capacity: 5     │
└─────────────────┘
```

```rust
// Creation
let v: Vec<i32> = Vec::new();
let v = vec![1, 2, 3];             // macro — initializes with values
let v: Vec<i32> = Vec::with_capacity(100);  // pre-allocate, len=0, cap=100

// Adding and removing
let mut v = vec![1, 2, 3];
v.push(4);                          // append to end — O(1) amortized
let last = v.pop();                 // remove from end — returns Option<T>

// Indexing
let x = v[0];                       // panics if out of bounds (only use when you're certain)
let x = v.get(0);                   // returns Option<&T> — safe
let x = v.get(0).unwrap_or(&0);     // Option handling
```

### Capacity and Pre-Allocation

When a Vec runs out of capacity, it allocates a new (larger) backing array and copies all
elements. This is the same strategy as Go slices, Java's ArrayList, C++'s `std::vector`.
The growth factor in the standard library is roughly 2x.

```rust
let mut v: Vec<i32> = Vec::new();
println!("capacity: {}", v.capacity()); // 0

v.push(1);
println!("capacity: {}", v.capacity()); // typically 4 (first allocation)

// Know your size upfront? Pre-allocate.
let mut scores: Vec<f64> = Vec::with_capacity(1000);
// Now 1000 pushes will not trigger any reallocations.
```

The rule of thumb: if you know approximately how many elements you'll have, use
`with_capacity`. Avoids repeated reallocations. Same advice as `make([]T, 0, n)` in Go.

### Slices: Borrowing Part of a Vec

A slice `&[T]` is a view into contiguous memory — it doesn't own the data, it borrows it.

```rust
let v = vec![1, 2, 3, 4, 5];
let slice: &[i32] = &v[1..4];   // [2, 3, 4] — borrows elements 1..4
let all: &[i32] = &v[..];        // entire vec as a slice

fn sum(data: &[i32]) -> i32 {    // accept slices, not Vecs — more flexible
    data.iter().sum()
}

sum(&v);                           // Vec coerces to &[T] automatically
sum(&v[2..]);                      // or pass a subslice
```

Writing functions that accept `&[T]` instead of `&Vec<T>` is idiomatic. The caller can
pass any contiguous slice — a full Vec, a subrange, a fixed-size array. It's the
equivalent of accepting `[]T` in Go (which is already a slice).

### drain and retain

```rust
let mut v = vec![1, 2, 3, 4, 5];

// drain: remove elements in a range and iterate over them
let removed: Vec<i32> = v.drain(1..3).collect();  // removes [2, 3]
// v is now [1, 4, 5], removed = [2, 3]

// retain: keep only elements matching a predicate (in-place filter)
let mut scores = vec![40, 80, 55, 90, 30, 75];
scores.retain(|&x| x >= 60);
// scores is now [80, 90, 75]
```

`retain` is like calling `filter` but mutating in place instead of producing a new
collection. O(n), no allocation for the output.

### Your notes
<!-- -->


---

## HashMap\<K, V\>

### Under the Hood

`HashMap` uses open addressing with a hash table. Keys are hashed (using SipHash by
default — designed to resist hash flooding attacks), and entries are stored in buckets.

When to use `HashMap` vs `BTreeMap`:
- `HashMap`: O(1) average lookup/insert, unordered. **Use this by default.**
- `BTreeMap`: O(log n) lookup/insert, sorted by key. Use when you need ordering or
  range queries.

```rust
use std::collections::HashMap;

// Creation
let mut map: HashMap<String, i32> = HashMap::new();
let mut map = HashMap::new();     // type inferred from usage

// Insert and retrieve
map.insert(String::from("alice"), 42);
map.insert(String::from("bob"), 37);

let age = map.get("alice");        // Option<&i32> — &str works for String keys via Borrow
let age = map["alice"];            // i32 — panics if key missing
let age = map.get("alice").copied(); // Option<i32> — copies the i32 out
```

Note that `map.get("alice")` works even though the key type is `String`. This is because
`HashMap<String, V>` implements `Index<&str>` — the standard library's `Borrow` trait
lets `&str` borrow from `String`. Same principle as `v.get(0)` returning `Option<&T>`.

### The Entry API: The Most Important Pattern

The entry API solves the single most common HashMap ergonomics problem: "insert a
default value if the key doesn't exist, then return a mutable reference."

Without entry API, you'd write:

```rust
// Clunky: two lookups
if !map.contains_key("errors") {
    map.insert(String::from("errors"), 0);
}
*map.get_mut("errors").unwrap() += 1;
```

With entry API:

```rust
// Idiomatic: single lookup, no duplication
map.entry(String::from("errors")).or_insert(0);
*map.entry(String::from("errors")).or_insert(0) += 1;
```

Or for a counting pattern (word frequency, log level counts, etc.):

```rust
let mut counts: HashMap<&str, u32> = HashMap::new();
let words = vec!["http", "tcp", "http", "grpc", "http", "tcp"];

for word in &words {
    let count = counts.entry(word).or_insert(0);
    *count += 1;
}
// counts: {"http": 3, "tcp": 2, "grpc": 1}
```

`entry()` returns an `Entry` enum — `Occupied` (key exists) or `Vacant` (key missing).
`or_insert(v)` inserts `v` if vacant and returns a `&mut V` either way. One lookup, no
double-borrow, no key cloning.

More entry variants:

```rust
// Use a closure to compute the default (only called if key is absent)
map.entry("errors").or_insert_with(|| expensive_default());

// Modify the existing value or set a default
map.entry("retries")
   .and_modify(|v| *v += 1)
   .or_insert(1);
```

### Iterating

```rust
let map: HashMap<String, i32> = ...;

// Immutable iteration — borrows map
for (key, value) in &map {
    println!("{}: {}", key, value);
}

// Mutable iteration — borrows map mutably
for (_, value) in &mut map {
    *value *= 2;
}

// Consuming iteration — moves map
for (key, value) in map {
    println!("{}: {}", key, value);
}
```

Iteration order is **not guaranteed** for `HashMap`. If you need consistent ordering, use
`BTreeMap` or collect into a `Vec` and sort.

### Your notes
<!-- -->


---

## HashSet\<T\>

`HashSet<T>` is essentially `HashMap<T, ()>`. Same hash table, same O(1) operations, but
storing only keys (members), no associated values.

```rust
use std::collections::HashSet;

let mut online: HashSet<String> = HashSet::new();
online.insert(String::from("alice"));
online.insert(String::from("bob"));
online.insert(String::from("alice"));  // duplicate — silently ignored

println!("online: {}", online.len()); // 2

let is_online = online.contains("alice"); // &str works for String sets
```

### Set Operations

The real power of `HashSet` is set math:

```rust
let a: HashSet<i32> = [1, 2, 3, 4].iter().cloned().collect();
let b: HashSet<i32> = [3, 4, 5, 6].iter().cloned().collect();

// Union: all elements from both
let union: HashSet<_> = a.union(&b).collect();         // {1, 2, 3, 4, 5, 6}

// Intersection: elements in both
let inter: HashSet<_> = a.intersection(&b).collect();  // {3, 4}

// Difference: in a but not b
let diff: HashSet<_> = a.difference(&b).collect();     // {1, 2}

// Symmetric difference: in either but not both
let sym: HashSet<_> = a.symmetric_difference(&b).collect(); // {1, 2, 5, 6}

// Subset check
let sub: HashSet<i32> = [3, 4].iter().cloned().collect();
println!("sub is subset: {}", sub.is_subset(&a));      // true
```

**When to use HashSet over Vec:** If you're doing membership tests (`contains`) repeatedly,
and elements should be unique, use `HashSet`. Linear scan through a `Vec` for membership
is O(n); `HashSet::contains` is O(1).

### Your notes
<!-- -->


---

## BTreeMap and BTreeSet

`BTreeMap<K, V>` and `BTreeSet<T>` store elements in sorted order. They use a B-tree
internally (not a binary search tree). All operations are O(log n).

```rust
use std::collections::BTreeMap;

let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
counts.insert("error", 10);
counts.insert("warn", 5);
counts.insert("info", 100);
counts.insert("debug", 50);

// Iteration is always in key-sorted order
for (level, count) in &counts {
    println!("{}: {}", level, count);  // debug, error, info, warn (alphabetical)
}

// Range queries — only possible with BTreeMap
for (level, count) in counts.range("e".."w") {
    // yields: error, info
}
```

### When to Choose BTreeMap over HashMap

| Situation | Use |
|---|---|
| Need sorted output without sorting manually | `BTreeMap` |
| Need range queries (all keys between A and B) | `BTreeMap` |
| Just need fast lookup, order doesn't matter | `HashMap` |
| Using non-hashable keys (types without `Hash`) | `BTreeMap` (only needs `Ord`) |
| Keys are floats | `BTreeMap` (f32/f64 don't implement `Hash`) |

If you don't have a specific reason for ordering, default to `HashMap`. It's faster.

### Your notes
<!-- -->


---

## VecDeque

`VecDeque<T>` is a double-ended queue backed by a ring buffer. O(1) push/pop from
**both** ends. `Vec` only gives O(1) at the back.

```rust
use std::collections::VecDeque;

let mut deque: VecDeque<String> = VecDeque::new();

deque.push_back(String::from("job-1"));   // add to back
deque.push_back(String::from("job-2"));
deque.push_front(String::from("priority")); // add to front (O(1))

let next = deque.pop_front();             // remove from front — FIFO queue behavior
let last = deque.pop_back();              // remove from back — stack behavior

// Convert between Vec and VecDeque
let v: Vec<i32> = vec![1, 2, 3];
let mut dq: VecDeque<i32> = v.into();    // cheap O(1) conversion in many cases

let v: Vec<i32> = dq.into();             // back to Vec
```

**When to use VecDeque:** Whenever you need efficient insertions or removals at both ends.
Common cases: job queues, BFS traversal, sliding window algorithms, event buffers where
the front gets consumed faster than the back fills.

### Your notes
<!-- -->


---

## Iterating Collections

This is where Rust gets powerful — and where ownership rules matter most.

### Three Iteration Modes

Every standard collection provides three iterator methods, each with different ownership
semantics:

```rust
let v = vec![String::from("a"), String::from("b"), String::from("c")];

// 1. iter() — borrows each element (&T), v is still usable after
for s in v.iter() {        // s: &String
    println!("{s}");
}
println!("v still alive: {}", v.len());

// 2. into_iter() — consumes the collection, moves each element (T)
for s in v.into_iter() {   // s: String, v is moved
    println!("{s}");
}
// v is gone — do not use after this

// 3. iter_mut() — borrows each element mutably (&mut T), allows in-place modification
let mut v = vec![1, 2, 3, 4, 5];
for n in v.iter_mut() {    // n: &mut i32
    *n *= 2;
}
// v is now [2, 4, 6, 8, 10]
```

The `for x in &collection` syntax desugars to `for x in collection.iter()`. This is the
idiomatic form for read-only iteration.

### collect: From Iterator to Collection

The `.collect()` method is how you turn an iterator back into a collection. You must tell
Rust what type to collect into, either via annotation or the turbofish syntax:

```rust
let v = vec![1, 2, 3, 4, 5];

// Annotation on the binding
let doubled: Vec<i32> = v.iter().map(|&x| x * 2).collect();

// Turbofish on collect
let doubled = v.iter().map(|&x| x * 2).collect::<Vec<i32>>();

// Collect into a HashMap
use std::collections::HashMap;
let pairs = vec![("a", 1), ("b", 2), ("c", 3)];
let map: HashMap<&str, i32> = pairs.into_iter().collect();

// Collect into a String (from chars)
let chars = vec!['h', 'e', 'l', 'l', 'o'];
let s: String = chars.into_iter().collect();

// Collect into a HashSet (deduplicates)
use std::collections::HashSet;
let with_dupes = vec![1, 2, 2, 3, 3, 3];
let unique: HashSet<i32> = with_dupes.into_iter().collect();
```

### Iterator Chaining

Iterators are lazy — nothing executes until you call a consuming adapter like
`.collect()`, `.sum()`, `.count()`, `.for_each()`, etc.

```rust
let log_lines = vec![
    "INFO starting up",
    "DEBUG loading config",
    "ERROR disk full",
    "WARN high memory",
    "ERROR connection refused",
];

let errors: Vec<&str> = log_lines.iter()
    .copied()                               // &&&str -> &str
    .filter(|line| line.starts_with("ERROR"))
    .collect();

let error_messages: Vec<&str> = log_lines.iter()
    .copied()
    .filter(|line| line.starts_with("ERROR"))
    .map(|line| &line[6..])                 // strip "ERROR " prefix
    .collect();

let total_len: usize = log_lines.iter()
    .map(|line| line.len())
    .sum();
```

### Your notes
<!-- -->


---

## Borrowing Rules with Collections

The borrow checker enforces rules that can feel restrictive at first, but each one
prevents a real class of bug.

### Cannot Hold a Reference and Mutate

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];       // immutable borrow
v.push(4);               // ERROR: mutable borrow while immutable borrow active
println!("{first}");
```

This fails because `push` might reallocate, which would invalidate `first`. The solution
is to not hold the reference across the mutation:

```rust
let mut v = vec![1, 2, 3];
let first_val = v[0];    // Copy the value out (i32 is Copy), no borrow needed
v.push(4);               // fine — no active borrow
println!("{first_val}");
```

Or restructure so the reference's scope ends before the mutation:

```rust
let mut v = vec![1, 2, 3];
{
    let first = &v[0];
    println!("{first}");
}   // first drops here
v.push(4);   // now fine
```

### Cannot Iterate and Mutate Simultaneously

```rust
let mut map: HashMap<String, i32> = HashMap::new();
map.insert("a".to_string(), 1);

for (k, v) in &map {                 // immutable borrow of map
    map.insert(k.clone(), v + 1);    // ERROR: cannot mutate while iterating
}
```

This is by design — modifying the collection during iteration can invalidate the iterator,
skip entries, or visit entries twice. Solutions depend on the goal:

```rust
// Option A: collect keys first, then mutate
let keys: Vec<String> = map.keys().cloned().collect();
for key in keys {
    if let Some(v) = map.get_mut(&key) {
        *v += 1;
    }
}

// Option B: build a separate map
let updated: HashMap<String, i32> = map.iter()
    .map(|(k, v)| (k.clone(), v + 1))
    .collect();
```

### The Entry API Is the Solution for HashMap Mutation

The classic "check then insert" pattern breaks the borrow rules:

```rust
// Does NOT compile:
if !map.contains_key("errors") {     // immutable borrow (1)
    map.insert("errors".to_string(), 0);  // mutable borrow (2) -- ERROR
}
```

The entry API was designed specifically for this. It performs a single lookup and returns
a handle (`Entry`) that lets you interact with the slot:

```rust
// Works fine:
map.entry("errors".to_string()).or_insert(0);
```

### Your notes
<!-- -->


---

## String as a Collection

`String` is UTF-8 encoded text stored as a `Vec<u8>` internally. This has implications
for how you iterate and index.

### Why You Can't Index a String by Integer

```rust
let s = String::from("hello");
// let ch = s[0];  // COMPILE ERROR — not allowed
```

The reason: UTF-8 characters can be 1-4 bytes each. Index 0 of a string could be the
first byte of a multi-byte character, and returning a `char` would be wrong. Returning a
`u8` would also be confusing. Rust just disallows it to avoid the ambiguity.

```rust
let s = String::from("café");

// Byte slicing (panics if you slice in the middle of a multi-byte char)
let bytes_slice = &s[0..4];   // "café" is 5 bytes in UTF-8 ('é' is 2 bytes)

// Iterate over Unicode characters (codepoints) — O(n)
for ch in s.chars() {
    println!("{ch}");   // c, a, f, é
}

// Iterate over raw bytes
for b in s.bytes() {
    println!("{b}");    // 99, 97, 102, 195, 169 (é = 0xC3 0xA9 in UTF-8)
}

// Get nth character (not O(1) — chars() is an iterator)
let third = s.chars().nth(2);  // Some('f')
```

### String Operations

```rust
let mut s = String::from("hello");

s.push(' ');                        // append a char
s.push_str("world");                // append a &str
let s2 = s + " again";             // + consumes s, borrows "again"
let s3 = format!("{} {}", s2, "!"); // format! doesn't consume

let split: Vec<&str> = "a,b,c".split(',').collect();
let joined = split.join(" | ");

let trimmed = "  hello  ".trim();
let upper = "hello".to_uppercase();
let contains = "hello world".contains("world");
```

### Your notes
<!-- -->


---

## Comparison to Go

| Feature | Go | Rust |
|---|---|---|
| Growable array | `[]T` (slice) | `Vec<T>` |
| Length | `len(s)` | `v.len()` |
| Capacity | `cap(s)` | `v.capacity()` |
| Pre-allocate | `make([]T, 0, n)` | `Vec::with_capacity(n)` |
| Append | `append(s, x)` (returns new slice) | `v.push(x)` (mutates in place) |
| Sub-slice | `s[1:3]` | `&v[1..3]` |
| Hash map | `map[K]V` | `HashMap<K, V>` |
| Map literal | `map[string]int{"a": 1}` | `HashMap::from([("a", 1)])` |
| Map access | `v, ok := m[k]` | `m.get(k)` returns `Option<&V>` |
| Set | No built-in | `HashSet<T>` |
| Ordered map | No built-in | `BTreeMap<K, V>` |
| Iterate map | `for k, v := range m` | `for (k, v) in &m` |

Key differences:
- **Go slices are shared views** — `s2 := s[1:3]` shares the backing array. Appending to
  `s2` might overwrite elements of `s`. Rust slices (`&[T]`) are purely borrowed views
  and can never cause this surprise.
- **Go maps return zero value** for missing keys — `m["missing"]` returns `0` for `int`
  maps. Rust forces you to handle the `None` case explicitly.
- **No nil collections in Rust** — you can't accidentally iterate a nil slice or map.
  `Vec::new()` is always safe to use.

## Comparison to TypeScript/JavaScript

| Feature | TypeScript | Rust |
|---|---|---|
| Growable array | `Array<T>` / `T[]` | `Vec<T>` |
| Append | `arr.push(x)` | `v.push(x)` |
| Remove last | `arr.pop()` | `v.pop()` (returns `Option<T>`) |
| Find element | `arr.find(fn)` | `v.iter().find(fn)` |
| Filter | `arr.filter(fn)` (new array) | `v.iter().filter(fn).collect()` |
| Map | `arr.map(fn)` (new array) | `v.iter().map(fn).collect()` |
| Hash map | `Map<K, V>` | `HashMap<K, V>` |
| Set | `Set<T>` | `HashSet<T>` |
| Object as map | `Record<K, V>` | `HashMap<String, V>` |

Key differences:
- **TypeScript arrays are reference types** — passing an array to a function shares it.
  In Rust, you choose: pass `&Vec<T>` (borrow), `&mut Vec<T>` (mutable borrow), or
  `Vec<T>` (transfer ownership).
- **`pop()` in TS returns undefined** if the array is empty. Rust's `pop()` returns
  `Option<T>` — you must handle `None`.
- **No implicit conversions** — Rust's `collect()` makes type conversions explicit.
  Chaining `.filter().map().collect()` is intentional, not invisible.

### Your notes
<!-- -->
