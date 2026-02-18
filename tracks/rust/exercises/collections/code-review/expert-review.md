# Expert Review: Word Frequency Counter

## Critical Issues

### 1. `top_words` returns an interleaved flat Vec — misleading and error-prone

**Location:** `top_words` function, return type `Vec<String>`

```rust
let result: Vec<String> = pairs
    .iter()
    .take(n)
    .flat_map(|(word, count)| vec![word.to_string(), count.to_string()])
    .collect();
```

**Problem:** The return type `Vec<String>` says "a list of words" but actually returns
an interleaved `["the", "3", "fox", "2"]`. Callers must know to read this in pairs, and
nothing in the type system enforces that. Any caller that iterates expecting strings will
get counts mixed in. The test itself reveals the oddity:

```rust
assert_eq!(top[0], "the");
assert_eq!(top[1], "3");  // this is a count, not a word
```

This design also loses type information — the count is serialized to a string and cannot
be used numerically without re-parsing.

**Fix:** Return structured data. A `Vec<(String, u32)>` is the obvious choice:

```rust
pub fn top_words(frequencies: &HashMap<String, u32>, n: usize) -> Vec<(String, u32)> {
    let mut pairs: Vec<(&String, &u32)> = frequencies.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    pairs.iter()
        .take(n)
        .map(|(word, count)| (word.to_string(), **count))
        .collect()
}
```

Callers can destructure: `for (word, count) in top_words(&freq, 5) { ... }`.

**Concept:** API design — return types should carry meaningful structure. `Vec<String>`
erases the distinction between words and counts. Prefer `Vec<(String, u32)>` or a named
struct when the result has internal structure.

---

## Major Concerns

### 2. `word_frequencies` does not use the entry API — three lookups per word

**Location:** `word_frequencies`, the counting loop

```rust
if counts.contains_key(&word) {
    let current = counts.get(&word).unwrap();
    counts.insert(word, current + 1);
} else {
    counts.insert(word, 1);
}
```

**Problem:** Three separate HashMap lookups per word: `contains_key`, `get`, and
`insert`. The entry API was designed precisely for this pattern and does it in one
lookup:

```rust
*counts.entry(word).or_insert(0) += 1;
```

Beyond performance (3x fewer hash computations), the entry API is safer: there is no
`.unwrap()` to potentially panic, and the code is 1 line instead of 6.

**Fix:**

```rust
pub fn word_frequencies(text: &str) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}
```

**Concept:** The entry API is the idiomatic way to insert-or-update. `contains_key` +
`insert` is a pattern from languages where this API doesn't exist (like early Java, or
Go which uses a different idiom). In Rust, use `entry` every time.

---

### 3. `word_exists` is O(n) when O(1) is available

**Location:** `word_exists`

```rust
pub fn word_exists(frequencies: &HashMap<String, u32>, word: &str) -> bool {
    let target = word.to_lowercase();
    for (k, _) in frequencies.iter() {  // linear scan through all keys
        if k == &target {
            return true;
        }
    }
    false
}
```

**Problem:** This is a linear scan — O(n) in the number of unique words. HashMap exists
specifically to make membership checks O(1). `contains_key` does exactly this:

```rust
pub fn word_exists(frequencies: &HashMap<String, u32>, word: &str) -> bool {
    frequencies.contains_key(&word.to_lowercase())
}
```

For a small document this is irrelevant, but for the "multi-megabyte documents" mentioned
in the requirements, calling `word_exists` repeatedly on a large HashMap becomes O(n*m)
instead of O(m).

**Concept:** `HashMap::contains_key` is O(1). Never use a loop to check membership in a
HashMap — that defeats the entire purpose of the data structure. This is exactly the
kind of regression a HashSet or HashMap is designed to prevent (compared to linear
search through a Vec or slice).

---

### 4. `total_word_count` and `unique_word_count` rebuild the full HashMap unnecessarily

**Location:** `total_word_count` and `unique_word_count`

```rust
pub fn total_word_count(text: &str) -> usize {
    let frequencies = word_frequencies(text);  // full O(n) pass + HashMap allocation
    frequencies.values().sum::<u32>() as usize
}

pub fn unique_word_count(text: &str) -> usize {
    let frequencies = word_frequencies(text);  // same
    frequencies.len()
}
```

**Problem:** Both functions build a full frequency HashMap — allocating and hashing
every word — just to get aggregate counts. `total_word_count` only needs to count words
(no HashMap needed), and `unique_word_count` only needs to count unique words (a
`HashSet` suffices).

```rust
pub fn total_word_count(text: &str) -> usize {
    text.split_whitespace().count()  // O(n), no allocation
}

pub fn unique_word_count(text: &str) -> usize {
    use std::collections::HashSet;
    text.split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<HashSet<String>>()
        .len()
}
```

`total_word_count` is now a one-liner with no allocations. `unique_word_count` allocates
a HashSet instead of a HashMap — cheaper since we don't store counts.

**Concept:** Match the data structure to what you actually need. If you need deduplication
only (no associated values), use `HashSet`. If you're only counting, don't build an index.
The principle: collect no more than you need.

---

### 5. `remove_stop_words` takes `Vec<String>` — forces allocation at every call site

**Location:** `remove_stop_words` signature

```rust
pub fn remove_stop_words(
    frequencies: &mut HashMap<String, u32>,
    stop_words: Vec<String>,           // takes ownership of a Vec
) {
```

**Problem:** Callers must either have an owned `Vec<String>` or create one with
`.to_string()` allocations. A slice reference is more flexible:

```rust
pub fn remove_stop_words(
    frequencies: &mut HashMap<String, u32>,
    stop_words: &[&str],               // borrows a slice of string slices
) {
    for word in stop_words {
        frequencies.remove(*word);     // HashMap<String, u32> supports &str lookup
    }
}
```

Callers can now write `remove_stop_words(&mut freq, &["the", "and", "or"])` — no
allocation for the stop words list. `HashMap::remove` accepts `&str` for `String` keys
via the `Borrow` trait.

**Concept:** Accept the most general borrowed type in parameters. `Vec<String>` requires
ownership. `&[&str]` (or `&[String]`) only requires a borrow. Functions that don't
need to own their input should borrow it. This is the same principle as accepting `&str`
instead of `String` for single strings.

---

### 6. `missing_words` takes `Vec<String>` — same issue

**Location:** `missing_words` signature

```rust
pub fn missing_words(text: &str, required: Vec<String>) -> Vec<String> {
```

Same problem as `remove_stop_words`. The function only reads `required`, never stores
it, yet it demands ownership. Fix:

```rust
pub fn missing_words(text: &str, required: &[&str]) -> Vec<String> {
    let frequencies = word_frequencies(text);
    required.iter()
        .filter(|&&word| !frequencies.contains_key(word))
        .map(|&word| word.to_string())
        .collect()
}
```

---

## Minor Suggestions

### 7. `unwrap()` in `top_words` is unnecessary

**Location:** The `unwrap()` was removed in the entry-API fix, but worth noting:

```rust
let current = counts.get(&word).unwrap();
```

After `contains_key` returns true, `.get()` will succeed — so this `unwrap()` won't
panic in practice. But the reasoning is fragile: the correctness depends on the
`contains_key` check immediately above not being moved. The entry API eliminates the
unwrap entirely.

**Concept:** Avoid `unwrap()` where a safe alternative exists. Every `unwrap()` is a
potential panic that callers cannot recover from.

---

### 8. `word_frequencies` is called once per utility function — no sharing

**Observation:** `unique_word_count`, `total_word_count`, and `missing_words` all call
`word_frequencies` internally, building separate frequency maps for each call. In a
real pipeline, you'd build the frequency map once and pass it to each function.

The better design: expose `word_frequencies` (which is already public), and make the
utility functions take `&HashMap<String, u32>` as a parameter rather than raw `&str`.
This lets callers amortize the analysis cost:

```rust
let freq = word_frequencies(text);
let top = top_words(&freq, 10);
let unique = freq.len();
let missing = missing_words_from_freq(&freq, required);
```

---

## Positive Feedback

1. **`remove_stop_words` uses `HashMap::remove` correctly** — calling `.remove()` on a
   missing key is a no-op in Rust (returns `None`), so no pre-check is needed. Clean.

2. **Sort comparator uses `.then()`** — the tie-breaking with `.then(a.0.cmp(b.0))` is
   the idiomatic Rust way to chain sort criteria. Well done.

3. **`word_frequencies` correctly lowercases before inserting** — normalizing case at
   ingest time rather than at query time is the right approach. Consistent keys.

4. **Good test coverage** — tests cover the main cases: counting, top words, existence,
   stop word removal, and total count. Edge cases like empty input could be added.

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | `top_words` returns interleaved flat `Vec<String>` | API design, return type structure |
| 2 | Major | `contains_key` + `get` + `insert` instead of entry API | Entry API, HashMap idioms |
| 3 | Major | `word_exists` is O(n) linear scan | `HashMap::contains_key` is O(1) |
| 4 | Major | `total_word_count` / `unique_word_count` build full HashMap | Right tool for the job; HashSet vs HashMap |
| 5 | Major | `remove_stop_words` takes `Vec<String>` (ownership) | Accept `&[&str]` — borrow, don't own |
| 6 | Major | `missing_words` takes `Vec<String>` (ownership) | Same as above |
| 7 | Minor | `unwrap()` after `contains_key` — fragile correctness | Entry API eliminates need |
| 8 | Minor | Utility functions rebuild frequency map on each call | Architecture — compute once, query many |

## Related Concepts

- [[fundamentals/rust/collections]] — Vec, HashMap, HashSet deep dive
- [[fundamentals/collections]] — Cross-language comparison
- [[pitfalls/rust-entry-api]] — When to use entry vs contains_key + insert
- [[pitfalls/rust-vec-where-set-fits]] — Choosing the right collection for membership tests
