# Rust Reference — Collections

> Extracted from [The Rust Standard Library Documentation](https://doc.rust-lang.org/std/collections/index.html)
> for the `collections` module. Covers: `Vec<T>`, `HashMap<K,V>`, `HashSet<T>`,
> `BTreeMap<K,V>`, `BTreeSet<T>`, `VecDeque<T>`, and iteration semantics.

---

## Overview

Source: [std::collections](https://doc.rust-lang.org/std/collections/index.html)

Rust's collections can be grouped into four major categories:

| Category | Collections |
|---|---|
| **Sequences** | `Vec`, `VecDeque`, `LinkedList` |
| **Maps** | `HashMap`, `BTreeMap` |
| **Sets** | `HashSet`, `BTreeSet` |
| **Misc** | `BinaryHeap` |

**When to use which:**

- Use `Vec` when you want a resizable array.
- Use `VecDeque` when you want a `Vec` that can efficiently insert at either end.
- Use `LinkedList` when you want a `Vec` or `VecDeque`, and you're absolutely certain you
  really, truly, actually need an O(1) split or append.
- Use `HashMap` when you want to associate arbitrary keys with an arbitrary value.
- Use `BTreeMap` when you want a map sorted by its keys.
- Use `HashSet` when you just want to remember which keys you've seen.
- Use `BTreeSet` when you want a set sorted by value.
- Use `BinaryHeap` when you want to store a bunch of elements, but only ever want to
  process the "biggest" or "most important" one at a time.

---

## Vec\<T\>

Source: [std::vec::Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html)

A contiguous growable array type, written as `Vec<T>`, short for "vector."

**Guarantees:**
- Elements are laid out contiguously in memory.
- The length is always less than or equal to the capacity.
- Reallocation only occurs when `len == capacity`.

**Layout:** `Vec<T>` consists of three fields:
- `ptr: NonNull<T>` — pointer to the heap-allocated backing array
- `len: usize` — number of initialized elements
- `capacity: usize` — number of elements the allocation can hold

### Creation

```rust
Vec::new()                   // empty, no allocation
Vec::with_capacity(n)        // empty, pre-allocated for n elements
vec![x, y, z]                // initialized with values (macro)
vec![default_val; n]         // n copies of default_val (T must be Clone)
Vec::from([1, 2, 3])         // from array
```

### Core Methods

| Method | Signature | Notes |
|---|---|---|
| `push` | `(val: T)` | Appends to the end. O(1) amortized. |
| `pop` | `() -> Option<T>` | Removes from end. O(1). |
| `insert` | `(idx: usize, val: T)` | Inserts at index. O(n). |
| `remove` | `(idx: usize) -> T` | Removes at index, shifts. O(n). |
| `swap_remove` | `(idx: usize) -> T` | Removes by swapping with last. O(1). |
| `len` | `() -> usize` | Number of elements. |
| `is_empty` | `() -> bool` | True if len == 0. |
| `capacity` | `() -> usize` | Allocated slots. |
| `get` | `(idx: usize) -> Option<&T>` | Safe index. |
| `get_mut` | `(idx: usize) -> Option<&mut T>` | Safe mutable index. |
| `contains` | `(&val: &T) -> bool` | Linear search. O(n). |
| `clear` | `()` | Removes all elements (capacity unchanged). |
| `truncate` | `(len: usize)` | Shortens to len. |
| `drain` | `(range: RangeBounds<usize>) -> Drain<T>` | Removes range, returns iterator. |
| `retain` | `(f: FnMut(&T) -> bool)` | Keeps only elements matching predicate. |
| `dedup` | `()` | Removes consecutive duplicates. |
| `sort` | `()` | Sorts in place. Requires T: Ord. |
| `sort_by` | `(cmp: FnMut(&T, &T) -> Ordering)` | Sort with custom comparator. |
| `sort_by_key` | `(key: FnMut(&T) -> K)` | Sort by key function. |
| `extend` | `(iter: IntoIterator<Item=T>)` | Appends from iterator. |
| `append` | `(other: &mut Vec<T>)` | Drains other into self. |
| `split_off` | `(at: usize) -> Vec<T>` | Splits into two at index. |

### Indexing

```rust
// Index with usize — panics on out of bounds
let x: i32 = v[0];
let x: &i32 = &v[0];

// Safe access via get
let x: Option<&i32> = v.get(0);
let x: Option<&mut i32> = v.get_mut(0);

// Slice indexing — panics if range invalid
let s: &[i32] = &v[1..3];
```

### Capacity Management

```rust
v.reserve(additional)       // ensure at least n more elements can be pushed without realloc
v.reserve_exact(additional) // reserve exactly n more (less overallocation)
v.shrink_to_fit()           // release excess capacity
v.shrink_to(min_capacity)   // shrink to at least min_capacity
```

---

## HashMap\<K, V\>

Source: [std::collections::HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html)

A hash map implemented with quadratic probing and SIMD lookup (via the hashbrown library).

**Type requirements:**
- Keys must implement `Eq + Hash`
- Values have no requirements

**Hash function:** SipHash 1-3 by default. Provides protection against HashDoS attacks.
Can be replaced with a custom hasher (e.g., FxHashMap for non-adversarial data).

### Creation

```rust
HashMap::new()
HashMap::with_capacity(n)
HashMap::from([(k1, v1), (k2, v2)])  // from array of tuples
pairs.into_iter().collect::<HashMap<K, V>>()
```

### Core Methods

| Method | Signature | Notes |
|---|---|---|
| `insert` | `(k: K, v: V) -> Option<V>` | Inserts. Returns old value if key existed. |
| `get` | `(&Q) -> Option<&V>` | Lookup by key reference. |
| `get_mut` | `(&Q) -> Option<&mut V>` | Mutable lookup. |
| `remove` | `(&Q) -> Option<V>` | Removes and returns value. |
| `contains_key` | `(&Q) -> bool` | Membership check. |
| `len` | `() -> usize` | Number of key-value pairs. |
| `is_empty` | `() -> bool` | |
| `keys` | `() -> Keys<K, V>` | Iterator over keys. |
| `values` | `() -> Values<K, V>` | Iterator over values. |
| `values_mut` | `() -> ValuesMut<K, V>` | Mutable iterator over values. |
| `iter` | `() -> Iter<K, V>` | Iterator over `(&K, &V)` pairs. |
| `iter_mut` | `() -> IterMut<K, V>` | Iterator over `(&K, &mut V)` pairs. |
| `entry` | `(k: K) -> Entry<K, V>` | Entry API. See below. |
| `retain` | `(f: FnMut(&K, &mut V) -> bool)` | Remove entries not matching predicate. |

### Entry API

Source: [std::collections::hash_map::Entry](https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html)

```rust
pub enum Entry<'a, K, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}
```

`Entry` methods (available on both variants, or variant-specific):

| Method | Signature | Notes |
|---|---|---|
| `or_insert` | `(default: V) -> &mut V` | Insert default if absent. Return &mut V. |
| `or_insert_with` | `(f: FnOnce() -> V) -> &mut V` | Insert computed default if absent. |
| `or_insert_with_key` | `(f: FnOnce(&K) -> V) -> &mut V` | Default computed from key. |
| `or_default` | `() -> &mut V` where `V: Default` | Insert `V::default()` if absent. |
| `and_modify` | `(f: FnOnce(&mut V)) -> Self` | Apply closure to value if occupied. |
| `key` | `() -> &K` | Returns reference to the entry's key. |

**Pattern — word/event counting:**

```rust
let mut counts = HashMap::new();
for word in text.split_whitespace() {
    counts.entry(word).or_insert(0) += 1;
    // equivalent to:
    // let c = counts.entry(word).or_insert(0);
    // *c += 1;
}
```

**Pattern — update if exists, insert if not:**

```rust
map.entry(key)
   .and_modify(|v| *v += 1)
   .or_insert(1);
```

### Key Lookup with Borrow

`HashMap<K, V>` implements `Index` and `get` using the `Borrow` trait. This means:

```rust
// Map with String keys — can look up with &str (no allocation needed)
let mut map: HashMap<String, i32> = HashMap::new();
map.insert(String::from("host"), 8080);

// All of these work:
let v = map.get("host");                    // &str borrows from String
let v = map.get(&String::from("host"));     // &String also works
let v = map["host"];                        // indexing with &str
```

---

## HashSet\<T\>

Source: [std::collections::HashSet](https://doc.rust-lang.org/std/collections/struct.HashSet.html)

A hash set implemented as a `HashMap<T, ()>`.

**Type requirements:** `T: Eq + Hash`

### Creation

```rust
HashSet::new()
HashSet::with_capacity(n)
HashSet::from([1, 2, 3])
iter.collect::<HashSet<T>>()
```

### Core Methods

| Method | Signature | Notes |
|---|---|---|
| `insert` | `(val: T) -> bool` | Insert. Returns false if already present. |
| `remove` | `(&Q) -> bool` | Remove. Returns false if not present. |
| `contains` | `(&Q) -> bool` | Membership test. O(1) average. |
| `len` | `() -> usize` | |
| `is_empty` | `() -> bool` | |
| `iter` | `() -> Iter<T>` | Unordered iteration. |

### Set Operations

All set operations return iterators (lazy). Call `.collect()` to materialize.

| Method | Returns | Description |
|---|---|---|
| `union(&other)` | `Union<T>` | All elements from self or other. |
| `intersection(&other)` | `Intersection<T>` | Elements in both self and other. |
| `difference(&other)` | `Difference<T>` | Elements in self but not other. |
| `symmetric_difference(&other)` | `SymmetricDifference<T>` | In either but not both. |
| `is_subset(&other)` | `bool` | True if self ⊆ other. |
| `is_superset(&other)` | `bool` | True if self ⊇ other. |
| `is_disjoint(&other)` | `bool` | True if self ∩ other = ∅. |

---

## BTreeMap\<K, V\>

Source: [std::collections::BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)

A map based on a B-tree, ordered by key.

**Type requirements:** `K: Ord`

**Complexity:** O(log n) for all basic operations (get, insert, remove, contains_key).
Iteration is O(n), in sorted key order.

### Additional Methods (not in HashMap)

| Method | Signature | Notes |
|---|---|---|
| `range` | `(range: RangeBounds<K>) -> Range<K,V>` | Iterate over a key range. |
| `range_mut` | `(range: RangeBounds<K>) -> RangeMut<K,V>` | Mutable range iteration. |
| `first_key_value` | `() -> Option<(&K, &V)>` | Smallest key. |
| `last_key_value` | `() -> Option<(&K, &V)>` | Largest key. |
| `pop_first` | `() -> Option<(K, V)>` | Remove and return smallest entry. |
| `pop_last` | `() -> Option<(K, V)>` | Remove and return largest entry. |

---

## BTreeSet\<T\>

Source: [std::collections::BTreeSet](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html)

A set based on a B-tree. Same as `HashSet` for set operations, but elements are always
in sorted order.

**Type requirements:** `T: Ord`

All `HashSet` set operations are also available on `BTreeSet`.

Additional:
- `range(range)` — iterate over elements in a range
- `first()` — smallest element
- `last()` — largest element

---

## VecDeque\<T\>

Source: [std::collections::VecDeque](https://doc.rust-lang.org/std/collections/struct.VecDeque.html)

A double-ended queue implemented with a growable ring buffer.

**Complexity:**
- `push_front`, `push_back`: O(1) amortized
- `pop_front`, `pop_back`: O(1) amortized
- Index access: O(1) (note: may not be contiguous in memory)

### Core Methods

| Method | Signature | Notes |
|---|---|---|
| `push_front` | `(val: T)` | Add to front. O(1) amortized. |
| `push_back` | `(val: T)` | Add to back. O(1) amortized. |
| `pop_front` | `() -> Option<T>` | Remove from front. |
| `pop_back` | `() -> Option<T>` | Remove from back. |
| `front` | `() -> Option<&T>` | Peek at front without removing. |
| `back` | `() -> Option<&T>` | Peek at back without removing. |
| `make_contiguous` | `() -> &mut [T]` | Rearranges so elements are contiguous (needed for sort). |

### Conversion

```rust
// Vec -> VecDeque: O(1) — reuses the allocation
let v: Vec<i32> = vec![1, 2, 3];
let dq: VecDeque<i32> = v.into();

// VecDeque -> Vec: may need to rotate buffer to be contiguous — O(n) worst case
let v: Vec<i32> = dq.into();
```

---

## Iterator Methods on Collections

Source: [std::iter::Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html)

All collections implement `IntoIterator`. Once you have an iterator, these are the key
adapters and consumers.

### Adapters (lazy — do not execute until consumed)

| Method | Notes |
|---|---|
| `map(f)` | Transform each element. |
| `filter(pred)` | Keep elements where predicate is true. |
| `filter_map(f)` | Combine filter and map via `Option`. |
| `flat_map(f)` | Map to iterators, flatten one level. |
| `flatten()` | Flatten one level of nesting. |
| `take(n)` | Yield first n elements. |
| `skip(n)` | Skip first n elements. |
| `enumerate()` | Yield `(index, element)` pairs. |
| `zip(other)` | Zip two iterators together. |
| `chain(other)` | Concatenate two iterators. |
| `peekable()` | Adds `.peek()` to look without consuming. |
| `cloned()` | Call `.clone()` on each `&T`. |
| `copied()` | Call `.copy()` on each `&T` (requires T: Copy). |

### Consumers (eager — execute immediately, return a value)

| Method | Returns | Notes |
|---|---|---|
| `collect()` | `B: FromIterator<T>` | Materialize into a collection. |
| `count()` | `usize` | Count elements. |
| `sum()` | `S: Sum<T>` | Sum all elements. |
| `product()` | `P: Product<T>` | Product of all elements. |
| `fold(init, f)` | accumulator type | Reduce with initial value. |
| `reduce(f)` | `Option<T>` | Reduce without initial value. |
| `find(pred)` | `Option<T>` | First element matching predicate. |
| `position(pred)` | `Option<usize>` | Index of first match. |
| `any(pred)` | `bool` | True if any element matches. |
| `all(pred)` | `bool` | True if all elements match. |
| `max()` | `Option<T>` | Maximum element. |
| `min()` | `Option<T>` | Minimum element. |
| `max_by_key(f)` | `Option<T>` | Maximum by key function. |
| `for_each(f)` | `()` | Apply closure to each element. |
| `last()` | `Option<T>` | Last element (consumes iterator). |
| `nth(n)` | `Option<T>` | nth element. |

---

## Iteration: iter(), into_iter(), iter_mut()

Source: [std::iter](https://doc.rust-lang.org/std/iter/index.html)

The standard convention for collections:

| Method | Self | Yields | Ownership of items |
|---|---|---|---|
| `iter()` | `&Collection<T>` | `&T` | Borrowed — collection not consumed |
| `iter_mut()` | `&mut Collection<T>` | `&mut T` | Mutably borrowed — collection not consumed |
| `into_iter()` | `Collection<T>` | `T` | Moved — collection is consumed |

**For loops desugar to `into_iter()`:**

```rust
// These are equivalent:
for x in v { ... }
for x in v.into_iter() { ... }

// To borrow, write either:
for x in &v { ... }
for x in v.iter() { ... }

// To borrow mutably:
for x in &mut v { ... }
for x in v.iter_mut() { ... }
```

---

## collect() and FromIterator

Source: [std::iter::FromIterator](https://doc.rust-lang.org/std/iter/trait.FromIterator.html)

`collect()` works with any type that implements `FromIterator`. The type must be
specified, either via annotation or turbofish:

```rust
// Type annotation on binding
let v: Vec<i32> = iter.collect();
let s: HashSet<i32> = iter.collect();
let m: HashMap<K, V> = pairs.collect();
let s: String = chars.collect();

// Turbofish on collect
let v = iter.collect::<Vec<i32>>();
let v = iter.collect::<Vec<_>>();  // _ lets Rust infer the element type
```

**Collecting Results:** When you have an iterator of `Result<T, E>`, you can collect into
`Result<Vec<T>, E>` — it will short-circuit on the first error:

```rust
let strings = vec!["1", "2", "three", "4"];
let parsed: Result<Vec<i32>, _> = strings.iter()
    .map(|s| s.parse::<i32>())
    .collect();
// Err(ParseIntError) — stopped at "three"
```

---

## String

Source: [std::string::String](https://doc.rust-lang.org/std/string/struct.String.html)

`String` is a growable, heap-allocated, UTF-8 encoded string type. Internally it is a
`Vec<u8>` that is guaranteed to be valid UTF-8.

**Key distinction:** `String` owns its data. `&str` is a borrowed reference to UTF-8
bytes (could be in the binary, on the heap via String, or on the stack).

### Why String Indexing Is Not O(1)

UTF-8 is a variable-width encoding. A `char` is 1-4 bytes. Indexing by character
position requires scanning from the beginning — O(n). Indexing by byte position is O(1)
but may split a multi-byte character (which is undefined behavior to allow).

Rust prevents byte-level indexing (`s[0]`) entirely to avoid silent corruption.
Byte-range slicing (`&s[0..4]`) is allowed but panics if the range splits a character.

### Key Methods

| Method | Notes |
|---|---|
| `chars()` | Iterator over Unicode scalar values (`char`). O(n) per char. |
| `bytes()` | Iterator over raw UTF-8 bytes (`u8`). |
| `len()` | Length in **bytes**, not characters. |
| `is_empty()` | True if len == 0. |
| `contains(&str)` | Substring check. |
| `starts_with(&str)` | |
| `ends_with(&str)` | |
| `find(&str)` | Byte index of first occurrence. Returns `Option<usize>`. |
| `split(pat)` | Iterator over substrings split by pattern. |
| `trim()` | Strip leading and trailing whitespace. Returns `&str`. |
| `to_uppercase()` / `to_lowercase()` | Returns a new `String`. |
| `replace(from, to)` | Returns a new `String`. |
| `push(char)` | Append a character. |
| `push_str(&str)` | Append a string slice. |

### Conversion

```rust
// &str -> String
"hello".to_string()
"hello".to_owned()
String::from("hello")

// String -> &str
let s = String::from("hello");
let slice: &str = &s;          // deref coercion
let slice: &str = s.as_str();  // explicit

// Number -> String
42i32.to_string()
format!("{}", 42)

// String -> number
"42".parse::<i32>()  // Result<i32, ParseIntError>
```
