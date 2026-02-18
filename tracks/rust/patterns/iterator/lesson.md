# Iterator Pattern — Rust

## Why Iterators Exist

Every program processes sequences of things. Log entries from a file, rows from a database, events from a message queue, bytes from a network socket. The naive approach is to shove everything into a `Vec` and loop over it:

```rust
let entries: Vec<LogEntry> = load_all_from_file("access.log"); // 2GB in memory
let errors: Vec<&LogEntry> = entries.iter().filter(|e| e.level == "ERROR").collect();
let first_ten: Vec<&LogEntry> = errors[..10].to_vec();
```

This materializes the entire dataset in memory before touching a single element. The iterator pattern solves this by making sequences **lazy** — elements are produced one at a time, on demand, and pipeline stages compose without intermediate allocations.

Rust's iterator system is not just a convenience — it is a **zero-cost abstraction**. An iterator chain like `.filter().map().take(10).collect()` compiles down to the same machine code as a hand-written loop with an index counter and early break. The optimizer sees through the abstraction completely.

### Your notes
<!-- -->

---

## The `Iterator` Trait

At the core of Rust's iterator system is a single trait with one required method:

```rust
pub trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;

    // 70+ provided methods: map, filter, fold, collect, etc.
}
```

That's it. `Item` is an associated type — the type of values this iterator yields. `next()` returns `Some(value)` when there's more data and `None` when the sequence is exhausted.

Everything else — `map`, `filter`, `fold`, `collect`, `zip`, `chain`, `take`, `skip`, `enumerate`, `flat_map`, `sum`, `any`, `all`, `find`, `count`, `min`, `max`, `peekable`, `windows`, `chunks` — is built on top of `next()`.

### Comparison: How other languages do this

| Language | Mechanism | Lazy by default? | Zero-cost? |
|----------|-----------|-------------------|------------|
| **Rust** | `Iterator` trait, `next()` method | Yes | Yes (monomorphized) |
| **Go** | `range` keyword, `iter.Seq` (Go 1.23+) | No (slices), yes (range-over-func) | Depends on implementation |
| **TypeScript** | `Symbol.iterator`, generators, Array methods | `.map`/`.filter` eager on arrays, generators lazy | No (allocation per step) |
| **Python** | `__iter__`/`__next__`, generators | Generators lazy, list comprehensions eager | No |
| **Haskell** | Lists are lazy by default | Yes | Depends on fusion rules |

Go's approach is fundamentally different. Until Go 1.23, there was no standard iterator protocol — you used channels (expensive goroutine per iterator), or returned slices (eager). The new `iter.Seq` type brings function-based iteration, but it's callback-style rather than pull-based. Rust's pull-based `next()` model gives the consumer control over when to advance, which is essential for combinators like `zip` and `take`.

TypeScript's `Array.prototype.map()` and `.filter()` create new arrays at every step. If you chain `.filter().map().filter()`, you get three intermediate arrays. Rust's iterator adapters return new iterator types that wrap the previous one — no allocation happens until you consume the chain.

### Your notes
<!-- -->

---

## Implementing `Iterator` for Custom Types

### A basic range iterator

```rust
struct CountUp {
    current: u32,
    max: u32,
}

impl CountUp {
    fn new(start: u32, max: u32) -> Self {
        CountUp { current: start, max }
    }
}

impl Iterator for CountUp {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.current < self.max {
            let val = self.current;
            self.current += 1;
            Some(val)
        } else {
            None
        }
    }
}
```

Once you implement `Iterator`, you get the entire adapter/consumer API for free:

```rust
let sum: u32 = CountUp::new(1, 101).filter(|n| n % 2 == 0).sum();
// sum of even numbers from 1 to 100 = 2550
```

### An infinite iterator

Iterators don't have to end. The `Fibonacci` sequence is naturally infinite:

```rust
struct Fibonacci {
    a: u64,
    b: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { a: 0, b: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let current = self.a;
        self.a = self.b;
        self.b = current + self.b;
        Some(current) // Always returns Some — infinite
    }
}
```

**Critical:** Calling `.collect()` on an infinite iterator will exhaust memory and crash. You must bound it with `.take(n)`:

```rust
let first_ten: Vec<u64> = Fibonacci::new().take(10).collect();
// [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
```

### `size_hint()` — helping consumers allocate

The default `size_hint()` returns `(0, None)` — "I might yield zero elements, I might yield infinite." Override it when you know better:

```rust
impl Iterator for CountUp {
    type Item = u32;

    fn next(&mut self) -> Option<u32> { /* ... */ }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.max - self.current) as usize;
        (remaining, Some(remaining))
    }
}
```

This matters because `collect::<Vec<_>>()` calls `size_hint()` to pre-allocate the `Vec`. Without it, the `Vec` starts empty and reallocates as it grows (amortized doubling). With an accurate hint, it allocates once.

### Your notes
<!-- -->

---

## `IntoIterator` — Making Types Work with `for` Loops

The `for` loop in Rust doesn't use `Iterator` directly. It uses `IntoIterator`:

```rust
pub trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter;
}
```

When you write `for x in collection`, the compiler desugars it to:

```rust
let mut iter = collection.into_iter();
while let Some(x) = iter.next() {
    // loop body
}
```

Every type that implements `Iterator` automatically implements `IntoIterator` (the `into_iter()` just returns `self`). But you can also implement `IntoIterator` for collection types that aren't themselves iterators:

```rust
struct Sensor {
    readings: Vec<f64>,
}

impl IntoIterator for Sensor {
    type Item = f64;
    type IntoIter = std::vec::IntoIter<f64>;

    fn into_iter(self) -> Self::IntoIter {
        self.readings.into_iter() // Consumes self, moves readings out
    }
}

// Now this works:
let sensor = Sensor { readings: vec![23.1, 24.5, 22.8] };
for reading in sensor {
    println!("{}", reading);
}
// sensor is MOVED here — can't use it after the loop
```

### Your notes
<!-- -->

---

## `iter()` vs `into_iter()` vs `iter_mut()` — Borrowing vs Ownership

This is one of the most important distinctions in Rust's iterator system. The three methods produce iterators with different ownership semantics:

| Method | Yields | Ownership | Collection after? |
|--------|--------|-----------|-------------------|
| `.iter()` | `&T` | Borrows immutably | Still usable |
| `.iter_mut()` | `&mut T` | Borrows mutably | Still usable (modified) |
| `.into_iter()` | `T` | Moves values out | **Consumed — gone** |

```rust
let names = vec!["alice".to_string(), "bob".to_string(), "carol".to_string()];

// .iter() — borrows, collection survives
for name in names.iter() {
    println!("{}", name); // name: &String
}
println!("Still have {} names", names.len()); // OK

// .iter_mut() — mutable borrows
let mut scores = vec![85, 92, 78];
for score in scores.iter_mut() {
    *score += 5; // Curve every grade up by 5
}
println!("{:?}", scores); // [90, 97, 83]

// .into_iter() — moves values out, consumes the Vec
let owned: Vec<String> = names.into_iter()
    .map(|name| name.to_uppercase())
    .collect();
// names is GONE here — moved into the iterator
```

**The `for` loop convention:**

When you write `for x in &collection`, Rust calls `collection.iter()`.
When you write `for x in &mut collection`, Rust calls `collection.iter_mut()`.
When you write `for x in collection`, Rust calls `collection.into_iter()`.

This is why `for x in &vec` doesn't consume the `Vec` — it's syntactic sugar for `.iter()`.

### Coming from TypeScript / Go

In TypeScript, `array.map()` always works on references to elements — there's no concept of "consuming" the array. In Go, `range` always copies the element (value semantics). Rust forces you to choose: are you borrowing, mutating, or consuming? This explicitness prevents an entire class of bugs around dangling references and use-after-move.

### Your notes
<!-- -->

---

## Iterator Adapters — Lazy Transformations

Adapters are methods on `Iterator` that return a new iterator wrapping the original. They are **lazy** — calling `.map()` does not iterate anything. It creates a `Map` struct that will transform elements when `next()` is called.

### Core adapters

```rust
let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// map — transform each element
let doubled: Vec<i32> = data.iter().map(|x| x * 2).collect();
// [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]

// filter — keep elements matching a predicate
let evens: Vec<&i32> = data.iter().filter(|x| *x % 2 == 0).collect();
// [&2, &4, &6, &8, &10]

// filter_map — filter and transform in one step (drops None values)
let parsed: Vec<i32> = vec!["1", "two", "3", "four", "5"]
    .into_iter()
    .filter_map(|s| s.parse::<i32>().ok())
    .collect();
// [1, 3, 5]

// take — stop after n elements
let first_three: Vec<&i32> = data.iter().take(3).collect();
// [&1, &2, &3]

// skip — discard the first n elements
let after_five: Vec<&i32> = data.iter().skip(5).collect();
// [&6, &7, &8, &9, &10]

// take_while / skip_while — predicate-based bounds
let until_five: Vec<&i32> = data.iter().take_while(|&&x| x < 5).collect();
// [&1, &2, &3, &4]

// enumerate — attach indices
let indexed: Vec<(usize, &i32)> = data.iter().enumerate().collect();
// [(0, &1), (1, &2), ..., (9, &10)]

// zip — pair two iterators element by element
let names = vec!["alice", "bob", "carol"];
let scores = vec![95, 87, 91];
let paired: Vec<(&&str, &i32)> = names.iter().zip(scores.iter()).collect();
// [("alice", 95), ("bob", 87), ("carol", 91)]

// chain — concatenate two iterators
let combined: Vec<i32> = vec![1, 2, 3].into_iter().chain(vec![4, 5, 6]).collect();
// [1, 2, 3, 4, 5, 6]

// flat_map — map then flatten (like flatMap in JS/Scala)
let words: Vec<&str> = vec!["hello world", "foo bar"]
    .iter()
    .flat_map(|s| s.split_whitespace())
    .collect();
// ["hello", "world", "foo", "bar"]

// flatten — collapse nested iterators
let nested = vec![vec![1, 2], vec![3, 4], vec![5]];
let flat: Vec<&i32> = nested.iter().flatten().collect();
// [&1, &2, &3, &4, &5]

// peekable — look ahead without consuming
let mut iter = data.iter().peekable();
assert_eq!(iter.peek(), Some(&&1)); // Peek at next without consuming
assert_eq!(iter.next(), Some(&1));  // Now consume it
```

### Laziness in action

```rust
// Nothing happens here — no iteration occurs
let lazy = (0..1_000_000)
    .filter(|x| x % 2 == 0)
    .map(|x| x * x);

// Only now does iteration begin, and it stops after 5 elements
let result: Vec<i64> = lazy.take(5).collect();
// [0, 4, 16, 36, 64]
// Only 10 elements were evaluated (filter checked ~10, map ran 5 times)
```

This is fundamentally different from TypeScript's eager array methods:

```typescript
// TypeScript: THREE full array traversals + THREE allocations
const result = [0, 1, 2, ..., 999999]
  .filter(x => x % 2 === 0)   // Creates 500,000-element array
  .map(x => x * x)             // Creates another 500,000-element array
  .slice(0, 5);                 // Finally takes 5
```

### Your notes
<!-- -->

---

## Consumers — Driving Iteration

Consumers call `next()` repeatedly to produce a final value. They trigger the actual computation.

```rust
let data = vec![1, 2, 3, 4, 5];

// collect — gather into a collection (most common)
let doubled: Vec<i32> = data.iter().map(|x| x * 2).collect();

// collect with type turbofish — when Rust can't infer the collection type
let set: std::collections::HashSet<i32> = data.iter().copied().collect();
// or equivalently:
let set = data.iter().copied().collect::<std::collections::HashSet<i32>>();

// fold — reduce to a single value with an accumulator
let sum = data.iter().fold(0, |acc, x| acc + x);
// 15

// sum — shorthand for numeric fold
let sum: i32 = data.iter().sum();
// 15

// count — how many elements
let n = data.iter().filter(|&&x| x > 3).count();
// 2

// any — does any element match?
let has_even = data.iter().any(|x| x % 2 == 0);
// true

// all — do all elements match?
let all_positive = data.iter().all(|x| *x > 0);
// true

// find — first element matching a predicate (returns Option)
let first_even = data.iter().find(|&&x| x % 2 == 0);
// Some(&2)

// position — index of first match
let idx = data.iter().position(|x| *x == 3);
// Some(2)

// min / max
let smallest = data.iter().min(); // Some(&1)
let largest = data.iter().max();  // Some(&5)

// min_by_key / max_by_key — compare by a derived key
let names = vec!["alice", "bob", "carol"];
let shortest = names.iter().min_by_key(|name| name.len());
// Some(&"bob")

// for_each — side effects (alternative to for loop)
data.iter().for_each(|x| println!("{}", x));
```

### `collect()` and `FromIterator`

`collect()` is powered by the `FromIterator` trait. Any type implementing `FromIterator<T>` can be the target of `.collect()`. The standard library implements it for `Vec`, `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `String`, `Result<Vec<T>, E>`, and more.

The `Result` collection is particularly powerful:

```rust
let results: Vec<Result<i32, String>> = vec![Ok(1), Ok(2), Err("bad".into()), Ok(4)];

// collect into Result<Vec<i32>, String> — short-circuits on first Err
let collected: Result<Vec<i32>, String> = results.into_iter().collect();
// Err("bad")
```

This replaces the common pattern of "try each item, bail on first error" with a single `.collect()` call.

### Your notes
<!-- -->

---

## Standard Library Iterators

Rust's standard library provides many built-in iterators that you use constantly:

```rust
// String iterators
let s = "hello, world";
let chars: Vec<char> = s.chars().collect();         // ['h', 'e', 'l', ...]
let bytes: Vec<u8> = s.bytes().collect();            // [104, 101, 108, ...]
let words: Vec<&str> = s.split(", ").collect();      // ["hello", "world"]
let lines: Vec<&str> = "line1\nline2".lines().collect();

// Range iterators
let range: Vec<i32> = (0..5).collect();              // [0, 1, 2, 3, 4]
let inclusive: Vec<i32> = (0..=5).collect();          // [0, 1, 2, 3, 4, 5]
let stepped: Vec<i32> = (0..10).step_by(3).collect(); // [0, 3, 6, 9]

// HashMap iteration
use std::collections::HashMap;
let mut map = HashMap::new();
map.insert("a", 1);
map.insert("b", 2);
for (key, value) in &map {
    println!("{}: {}", key, value);
}

// Repeat and once
let fives: Vec<i32> = std::iter::repeat(5).take(3).collect(); // [5, 5, 5]
let one: Vec<i32> = std::iter::once(42).collect();              // [42]
let empty: Vec<i32> = std::iter::empty::<i32>().collect();      // []

// successors — generate from a seed
let powers_of_two: Vec<u32> = std::iter::successors(Some(1u32), |&n| n.checked_mul(2))
    .take(10)
    .collect();
// [1, 2, 4, 8, 16, 32, 64, 128, 256, 512]
```

### `Peekable` — Look ahead without consuming

```rust
let mut iter = vec![1, 2, 3].into_iter().peekable();
while let Some(&next) = iter.peek() {
    if next % 2 == 0 {
        println!("Even: {}", iter.next().unwrap());
    } else {
        println!("Odd: {}", iter.next().unwrap());
    }
}
```

`Peekable` is essential for building parsers and tokenizers where you need to decide what to do based on the next element without consuming it.

### Your notes
<!-- -->

---

## Zero-Cost Abstractions

One of Rust's key promises: iterator chains compile to the same code as hand-written loops. The compiler monomorphizes each adapter (creates a specialized version for each concrete type) and then inlines everything.

Consider this chain:

```rust
fn sum_of_squares_of_evens(data: &[i32]) -> i32 {
    data.iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .sum()
}
```

After monomorphization and inlining, the compiler sees something equivalent to:

```rust
fn sum_of_squares_of_evens_manual(data: &[i32]) -> i32 {
    let mut total = 0;
    for &x in data {
        if x % 2 == 0 {
            total += x * x;
        }
    }
    total
}
```

Both produce identical assembly. You can verify this by compiling with `rustc --emit asm -O` or using [Compiler Explorer](https://godbolt.org/).

### When does it NOT zero-cost?

- **Trait objects (`Box<dyn Iterator>`)** — dynamic dispatch prevents inlining. The compiler can't see through the vtable to optimize.
- **Allocation in closures** — if a closure captures a `String` by clone, that allocation is real.
- **Collect into intermediate collections** — `iter.filter(...).collect::<Vec<_>>().iter().map(...)` defeats the pipeline. The `collect()` in the middle forces materialization.

The rule: keep the chain as one expression, avoid `collect()` between steps unless you need the intermediate result.

### Your notes
<!-- -->

---

## Custom Iterator Combinators

You can build reusable adapters by implementing `Iterator` on wrapper structs. This is how the standard library builds `Map`, `Filter`, `Take`, etc.

### Example: `Interleave` — alternate between two iterators

```rust
struct Interleave<A, B> {
    a: A,
    b: B,
    use_a: bool,
}

impl<A, B, T> Iterator for Interleave<A, B>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.use_a = !self.use_a;
        if !self.use_a {
            match self.a.next() {
                Some(val) => Some(val),
                None => self.b.next(),
            }
        } else {
            match self.b.next() {
                Some(val) => Some(val),
                None => self.a.next(),
            }
        }
    }
}

fn interleave<A, B, T>(a: A, b: B) -> Interleave<A, B>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    Interleave { a, b, use_a: false }
}
```

Usage:

```rust
let a = vec![1, 3, 5].into_iter();
let b = vec![2, 4, 6].into_iter();
let result: Vec<i32> = interleave(a, b).collect();
// [1, 2, 3, 4, 5, 6]
```

### Extension traits — adding methods to all iterators

You can extend `Iterator` with a trait:

```rust
trait IteratorExt: Iterator {
    fn interleave<B>(self, other: B) -> Interleave<Self, B::IntoIter>
    where
        Self: Sized,
        B: IntoIterator<Item = Self::Item>,
    {
        Interleave {
            a: self,
            b: other.into_iter(),
            use_a: false,
        }
    }
}

impl<I: Iterator> IteratorExt for I {}

// Now any iterator can use .interleave()
let result: Vec<i32> = (1..=3).interleave(4..=6).collect();
```

This is exactly how crates like `itertools` work — they provide an extension trait that adds dozens of additional combinators to all iterators.

### Your notes
<!-- -->

---

## `DoubleEndedIterator` and `ExactSizeIterator`

### `DoubleEndedIterator`

An iterator that can also yield elements from the back:

```rust
pub trait DoubleEndedIterator: Iterator {
    fn next_back(&mut self) -> Option<Self::Item>;
}
```

This enables `.rev()`:

```rust
let reversed: Vec<i32> = (1..=5).rev().collect();
// [5, 4, 3, 2, 1]
```

Slices, `Vec`, `VecDeque`, and ranges implement `DoubleEndedIterator`. Linked lists, hash maps, and custom generators typically do not.

### `ExactSizeIterator`

Declares that `size_hint()` returns exact bounds:

```rust
pub trait ExactSizeIterator: Iterator {
    fn len(&self) -> usize {
        let (lower, upper) = self.size_hint();
        assert_eq!(upper, Some(lower));
        lower
    }
}
```

Implementing this lets consumers use `.len()` and enables optimizations in `collect()`.

### Your notes
<!-- -->

---

## Real-World Patterns

### Pattern 1: Processing config entries lazily

```rust
fn load_env_config(prefix: &str) -> Vec<(String, String)> {
    std::env::vars()
        .filter(|(key, _)| key.starts_with(prefix))
        .map(|(key, val)| {
            let stripped = key.strip_prefix(prefix).unwrap_or(&key).to_lowercase();
            (stripped, val)
        })
        .collect()
}
```

### Pattern 2: Error-tolerant parsing pipeline

```rust
fn parse_log_levels(raw: &str) -> Result<Vec<u32>, String> {
    raw.lines()
        .enumerate()
        .map(|(line_num, line)| {
            line.trim()
                .parse::<u32>()
                .map_err(|e| format!("line {}: {} — '{}'", line_num + 1, e, line))
        })
        .collect() // Short-circuits on first error
}
```

### Pattern 3: Building a HashMap from pairs

```rust
use std::collections::HashMap;

fn word_frequencies(text: &str) -> HashMap<&str, usize> {
    text.split_whitespace()
        .fold(HashMap::new(), |mut counts, word| {
            *counts.entry(word).or_insert(0) += 1;
            counts
        })
}
```

### Pattern 4: Windowed statistics

```rust
fn moving_average(data: &[f64], window: usize) -> Vec<f64> {
    data.windows(window)
        .map(|w| w.iter().sum::<f64>() / w.len() as f64)
        .collect()
}
```

### Your notes
<!-- -->

---

## Cross-Language Comparison

### Go — The iterator landscape

Go historically had no iterator protocol. Common patterns:
- **Slices:** Materialize everything, range over it. Eager.
- **Channels:** `for val := range ch { ... }` — lazy but expensive (goroutine + synchronization per element).
- **Callbacks:** `filepath.Walk(root, func(path string, info fs.FileInfo, err error) error { ... })` — push-based, not composable.
- **`iter.Seq` (Go 1.23+):** `func(yield func(V) bool)` — push-based function iterators. Composable but the callback style is inverted compared to Rust's pull model.

Rust's pull-based model is more natural for composition: each adapter wraps the previous iterator and calls `next()` on demand. Go's push model requires the producer to call `yield`, which makes composition more awkward.

### TypeScript — Eager vs lazy

TypeScript has two worlds:
- **Array methods** (`.map()`, `.filter()`, etc.) — eager, allocate intermediate arrays.
- **Generators** (`function*`, `yield`) — lazy, but no built-in combinator library. You have to use libraries like `iter-tools` or `rxjs`.
- **Async iterators** (`for await...of`) — lazy, great for streams.

Rust gives you the lazy combinator library built into the language — no external dependency needed.

### Haskell — Lazy everywhere

Haskell lists are lazy by default. `[1..]` is an infinite list. `take 5 (filter even [1..])` just works. But laziness can also bite: unevaluated thunks pile up in memory (space leaks). Rust's iterators are lazy in control flow but strict in evaluation — each element is fully computed when `next()` yields it, avoiding the thunk buildup problem.

### Your notes
<!-- -->

---

## Key Takeaways

1. **The `Iterator` trait is one method** — `next(&mut self) -> Option<Self::Item>`. Everything else is derived.
2. **Adapters are lazy.** `.map()`, `.filter()`, `.take()` create wrapper types that do nothing until consumed.
3. **Consumers drive iteration.** `.collect()`, `.sum()`, `.fold()`, `.for_each()`, and `for` loops call `next()` repeatedly.
4. **Choose your ownership.** `.iter()` borrows, `.iter_mut()` mutates, `.into_iter()` consumes.
5. **`IntoIterator` powers `for` loops.** Implement it so your types work with `for x in my_type`.
6. **`collect()` is polymorphic.** It can produce `Vec`, `HashMap`, `String`, `Result<Vec<T>, E>`, etc. — driven by the target type.
7. **Zero-cost until you break the chain.** Intermediate `collect()` calls and `Box<dyn Iterator>` introduce overhead. Keep pipelines as single expressions.
8. **`size_hint()` matters.** Override it for custom iterators so `collect()` can pre-allocate.
9. **Custom combinators are just structs implementing `Iterator`.** Extension traits make them chainable.
10. **Infinite iterators are fine** — just always bound them with `.take()` or `.take_while()` before consuming.

### Your notes
<!-- -->
