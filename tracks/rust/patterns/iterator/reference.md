# Rust Reference — Iterator Pattern

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Book](https://doc.rust-lang.org/book/ch13-02-iterators.html), and
> [std library docs](https://doc.rust-lang.org/std/iter/)
> for the `iterator` module. Covers: Iterator trait, IntoIterator, adapters, consumers,
> FromIterator, DoubleEndedIterator, ExactSizeIterator.

---

## `Iterator` Trait

Source: [std::iter::Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html)

```rust
pub trait Iterator {
    type Item;

    // Required
    fn next(&mut self) -> Option<Self::Item>;

    // Provided (70+ methods) — selected below
}
```

### Associated Type: `Item`

The type of elements yielded by this iterator. Defined once per implementation (not generic — a given iterator yields exactly one type).

```rust
impl Iterator for MyIter {
    type Item = u32; // This iterator yields u32 values
    fn next(&mut self) -> Option<u32> { /* ... */ }
}
```

### `next(&mut self) -> Option<Self::Item>`

Advances the iterator and returns the next value. Returns `None` when iteration is finished. May be called again after returning `None` — behavior is implementation-defined (most return `None` forever, but this is not guaranteed by the trait).

---

## `IntoIterator` Trait

Source: [std::iter::IntoIterator](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)

```rust
pub trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter;
}
```

### Automatic Implementation

Every type that implements `Iterator` automatically implements `IntoIterator`:

```rust
impl<I: Iterator> IntoIterator for I {
    type Item = I::Item;
    type IntoIter = I;
    fn into_iter(self) -> I { self }
}
```

### For Loop Desugaring

```rust
// This:
for element in collection { /* ... */ }

// Desugars to:
let mut iter = IntoIterator::into_iter(collection);
loop {
    match iter.next() {
        Some(element) => { /* ... */ }
        None => break,
    }
}
```

### Standard Library Implementations for `Vec<T>`

| Call | Yields | Ownership |
|------|--------|-----------|
| `vec.into_iter()` | `T` | Consumes the `Vec` |
| `(&vec).into_iter()` / `vec.iter()` | `&T` | Borrows immutably |
| `(&mut vec).into_iter()` / `vec.iter_mut()` | `&mut T` | Borrows mutably |

The `for x in &vec` syntax calls `(&vec).into_iter()`, which delegates to `vec.iter()`.

---

## Iterator Adapters

Adapters transform an iterator into another iterator. They are **lazy** — no elements are processed until a consumer is called.

### `map`

```rust
fn map<B, F>(self, f: F) -> Map<Self, F>
where
    F: FnMut(Self::Item) -> B;
```

Transforms each element. The closure receives ownership of each `Item`.

### `filter`

```rust
fn filter<P>(self, predicate: P) -> Filter<Self, P>
where
    P: FnMut(&Self::Item) -> bool;
```

Yields only elements where the predicate returns `true`. Note: predicate receives `&Item`, not `Item`.

### `filter_map`

```rust
fn filter_map<B, F>(self, f: F) -> FilterMap<Self, F>
where
    F: FnMut(Self::Item) -> Option<B>;
```

Combines `filter` and `map`. Yields the inner value of `Some(B)`, drops `None`.

### `enumerate`

```rust
fn enumerate(self) -> Enumerate<Self>;
```

Wraps each element as `(index, element)` where index starts at 0.

### `take`

```rust
fn take(self, n: usize) -> Take<Self>;
```

Yields at most `n` elements, then stops.

### `skip`

```rust
fn skip(self, n: usize) -> Skip<Self>;
```

Skips the first `n` elements, then yields the rest.

### `take_while`

```rust
fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>
where
    P: FnMut(&Self::Item) -> bool;
```

Yields elements while the predicate returns `true`. Stops (and does not yield) the first element that fails.

### `skip_while`

```rust
fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>
where
    P: FnMut(&Self::Item) -> bool;
```

Skips elements while the predicate returns `true`. Once the predicate fails, yields all remaining elements.

### `chain`

```rust
fn chain<U>(self, other: U) -> Chain<Self, U::IntoIter>
where
    U: IntoIterator<Item = Self::Item>;
```

Concatenates two iterators. Exhausts `self` first, then `other`.

### `zip`

```rust
fn zip<U>(self, other: U) -> Zip<Self, U::IntoIter>
where
    U: IntoIterator;
```

Pairs elements from two iterators. Stops when either is exhausted.

### `flat_map`

```rust
fn flat_map<U, F>(self, f: F) -> FlatMap<Self, U, F>
where
    F: FnMut(Self::Item) -> U,
    U: IntoIterator;
```

Maps each element to an iterator, then flattens. Equivalent to `.map(f).flatten()`.

### `flatten`

```rust
fn flatten(self) -> Flatten<Self>
where
    Self::Item: IntoIterator;
```

Flattens nested iterators. `[[1,2],[3,4]].iter().flatten()` yields `1, 2, 3, 4`.

### `peekable`

```rust
fn peekable(self) -> Peekable<Self>;
```

Wraps the iterator so you can `.peek()` at the next element without consuming it.

### `inspect`

```rust
fn inspect<F>(self, f: F) -> Inspect<Self, F>
where
    F: FnMut(&Self::Item);
```

Calls the closure on each element (by reference) for side effects (logging, debugging), then yields the element unchanged.

### `cloned` / `copied`

```rust
fn cloned<'a, T: 'a + Clone>(self) -> Cloned<Self>
where
    Self: Iterator<Item = &'a T>;

fn copied<'a, T: 'a + Copy>(self) -> Copied<Self>
where
    Self: Iterator<Item = &'a T>;
```

Converts an iterator of `&T` to an iterator of `T` by cloning/copying. `copied` is restricted to `Copy` types and is preferred when applicable.

### `step_by`

```rust
fn step_by(self, step: usize) -> StepBy<Self>;
```

Yields every `step`-th element, starting from the first. `step` must be > 0.

### `rev`

```rust
fn rev(self) -> Rev<Self>
where
    Self: DoubleEndedIterator;
```

Reverses the iterator. Requires `DoubleEndedIterator`.

### `cycle`

```rust
fn cycle(self) -> Cycle<Self>
where
    Self: Clone;
```

Repeats the iterator endlessly. Requires `Clone` because the iterator must be reset.

---

## Consumers

Consumers drive iteration by calling `next()` repeatedly to produce a final value.

### `collect`

```rust
fn collect<B>(self) -> B
where
    B: FromIterator<Self::Item>;
```

Transforms an iterator into a collection. The target type must implement `FromIterator`. Common targets:

| Target | Constraint | Example |
|--------|-----------|---------|
| `Vec<T>` | None | `.collect::<Vec<_>>()` |
| `String` | `Item = char` or `Item = &str` or `Item = String` | `chars.collect::<String>()` |
| `HashMap<K, V>` | `Item = (K, V)` | `pairs.collect::<HashMap<_, _>>()` |
| `HashSet<T>` | None | `.collect::<HashSet<_>>()` |
| `BTreeMap<K, V>` | `Item = (K, V)`, `K: Ord` | `pairs.collect::<BTreeMap<_, _>>()` |
| `Result<Vec<T>, E>` | `Item = Result<T, E>` | Short-circuits on first `Err` |
| `Option<Vec<T>>` | `Item = Option<T>` | Short-circuits on first `None` |

### `fold`

```rust
fn fold<B, F>(self, init: B, f: F) -> B
where
    F: FnMut(B, Self::Item) -> B;
```

Reduces the iterator to a single value. The accumulator `B` can be any type.

### `reduce`

```rust
fn reduce<F>(self, f: F) -> Option<Self::Item>
where
    F: FnMut(Self::Item, Self::Item) -> Self::Item;
```

Like `fold` but uses the first element as the initial accumulator. Returns `None` if the iterator is empty.

### `sum` / `product`

```rust
fn sum<S>(self) -> S where S: Sum<Self::Item>;
fn product<P>(self) -> P where P: Product<Self::Item>;
```

Numeric specializations of `fold`. Require the `Sum` / `Product` trait.

### `count`

```rust
fn count(self) -> usize;
```

Consumes the iterator and counts elements.

### `any` / `all`

```rust
fn any<F>(&mut self, f: F) -> bool where F: FnMut(Self::Item) -> bool;
fn all<F>(&mut self, f: F) -> bool where F: FnMut(Self::Item) -> bool;
```

Short-circuit boolean tests. `any` returns on first `true`, `all` returns on first `false`. Note: these take `&mut self` — they can be resumed after.

### `find` / `find_map`

```rust
fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
where P: FnMut(&Self::Item) -> bool;

fn find_map<B, F>(&mut self, f: F) -> Option<B>
where F: FnMut(Self::Item) -> Option<B>;
```

Return the first matching element. `find_map` combines `find` and `map`.

### `position` / `rposition`

```rust
fn position<P>(&mut self, predicate: P) -> Option<usize>
where P: FnMut(Self::Item) -> bool;
```

Index of the first matching element. `rposition` searches from the back (requires `DoubleEndedIterator` + `ExactSizeIterator`).

### `min` / `max` / `min_by_key` / `max_by_key` / `min_by` / `max_by`

```rust
fn min(self) -> Option<Self::Item> where Self::Item: Ord;
fn max(self) -> Option<Self::Item> where Self::Item: Ord;

fn min_by_key<B: Ord, F>(self, f: F) -> Option<Self::Item>
where F: FnMut(&Self::Item) -> B;

fn min_by<F>(self, compare: F) -> Option<Self::Item>
where F: FnMut(&Self::Item, &Self::Item) -> std::cmp::Ordering;
```

### `for_each`

```rust
fn for_each<F>(self, f: F) where F: FnMut(Self::Item);
```

Calls the closure on each element. Preferred over `for` loop when you want to chain it with other operations or need to consume a `map()` chain for side effects.

### `unzip`

```rust
fn unzip<A, B, FromA, FromB>(self) -> (FromA, FromB)
where
    FromA: Default + Extend<A>,
    FromB: Default + Extend<B>,
    Self: Iterator<Item = (A, B)>;
```

Converts an iterator of pairs into a pair of collections.

### `partition`

```rust
fn partition<B, F>(self, f: F) -> (B, B)
where
    B: Default + Extend<Self::Item>,
    F: FnMut(&Self::Item) -> bool;
```

Splits elements into two collections based on a predicate.

---

## `FromIterator` Trait

Source: [std::iter::FromIterator](https://doc.rust-lang.org/std/iter/trait.FromIterator.html)

```rust
pub trait FromIterator<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self;
}
```

This is the trait that `collect()` calls. Implement it to make your custom types collectable:

```rust
struct Metrics {
    values: Vec<f64>,
    count: usize,
}

impl FromIterator<f64> for Metrics {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        let values: Vec<f64> = iter.into_iter().collect();
        let count = values.len();
        Metrics { values, count }
    }
}

// Now you can:
let m: Metrics = vec![1.0, 2.0, 3.0].into_iter().collect();
```

---

## `DoubleEndedIterator` Trait

Source: [std::iter::DoubleEndedIterator](https://doc.rust-lang.org/std/iter/trait.DoubleEndedIterator.html)

```rust
pub trait DoubleEndedIterator: Iterator {
    fn next_back(&mut self) -> Option<Self::Item>;

    // Provided
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> { /* ... */ }
    fn rfold<B, F>(self, init: B, f: F) -> B { /* ... */ }
    fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item> { /* ... */ }
}
```

Enables iteration from both ends simultaneously. `next()` advances from the front, `next_back()` from the rear. They share state — the iterator shrinks from both sides until they meet.

Enables: `.rev()`, `.rposition()`, `.rfold()`, `.rfind()`

### Implementors

| Type | `DoubleEndedIterator`? |
|------|------------------------|
| Slice `[T]` | Yes |
| `Vec<T>` | Yes |
| `Range<T>` | Yes |
| `HashMap` iter | No (unordered) |
| `BTreeMap` iter | Yes (ordered) |
| `String::chars()` | Yes |
| Custom generators | Typically no |

---

## `ExactSizeIterator` Trait

Source: [std::iter::ExactSizeIterator](https://doc.rust-lang.org/std/iter/trait.ExactSizeIterator.html)

```rust
pub trait ExactSizeIterator: Iterator {
    fn len(&self) -> usize {
        let (lower, upper) = self.size_hint();
        assert_eq!(upper, Some(lower));
        lower
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
```

Guarantees that `size_hint()` returns an exact count. Enables the `.len()` method on the iterator and allows consumers to optimize allocation.

### Contract

When implementing:
- `size_hint()` must return `(n, Some(n))` for some `n`
- After calling `next()` once, `size_hint()` must return `(n-1, Some(n-1))`

---

## `size_hint()`

```rust
fn size_hint(&self) -> (usize, Option<usize>) {
    (0, None) // Default: "I don't know"
}
```

Returns `(lower_bound, upper_bound)`:
- `lower_bound`: guaranteed minimum number of remaining elements
- `upper_bound`: `Some(n)` if a maximum is known, `None` if potentially infinite

Used by `collect()` and other consumers to pre-allocate. The default `(0, None)` means no optimization is possible.

| Iterator | `size_hint()` |
|----------|---------------|
| `vec.iter()` where `vec.len() == 5` | `(5, Some(5))` |
| `(0..10)` | `(10, Some(10))` |
| `iter.filter(...)` | `(0, upper_of_inner)` — filter can't predict how many pass |
| `iter.map(...)` | same as inner — map doesn't change count |
| `iter.chain(a, b)` | `(a.lower + b.lower, a.upper + b.upper)` |
| `iter.take(5)` | `(min(5, lower), min(5, upper))` |
| `iter.zip(a, b)` | `(min(a.lower, b.lower), min(a.upper, b.upper))` |

---

## Iterator Creation Functions

Source: [std::iter module functions](https://doc.rust-lang.org/std/iter/index.html#functions)

| Function | Description | Example |
|----------|-------------|---------|
| `iter::empty()` | Yields nothing | `iter::empty::<i32>()` |
| `iter::once(val)` | Yields `val` once | `iter::once(42)` |
| `iter::once_with(f)` | Yields `f()` once (lazy) | `iter::once_with(|| expensive())` |
| `iter::repeat(val)` | Yields `val` forever | `iter::repeat(0).take(10)` |
| `iter::repeat_with(f)` | Calls `f()` forever | `iter::repeat_with(|| rand())` |
| `iter::successors(seed, f)` | `seed, f(seed), f(f(seed)), ...` | `iter::successors(Some(1), |&n| n.checked_mul(2))` |
| `iter::from_fn(f)` | Calls `f()` for each element | `iter::from_fn(|| Some(42))` |

---

## Ranges as Iterators

Ranges implement `Iterator`:

| Range | Type | Yields | Bounds |
|-------|------|--------|--------|
| `0..5` | `Range<i32>` | `0, 1, 2, 3, 4` | Exclusive end |
| `0..=5` | `RangeInclusive<i32>` | `0, 1, 2, 3, 4, 5` | Inclusive end |

Ranges also implement `DoubleEndedIterator` and `ExactSizeIterator`.

```rust
// Step by
(0..20).step_by(3)  // 0, 3, 6, 9, 12, 15, 18

// Reverse
(0..5).rev()  // 4, 3, 2, 1, 0
```

---

## `Extend` Trait

Source: [std::iter::Extend](https://doc.rust-lang.org/std/iter/trait.Extend.html)

```rust
pub trait Extend<A> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T);
}
```

Adds elements from an iterator to an existing collection. Used internally by `partition` and `unzip`.

```rust
let mut vec = vec![1, 2, 3];
vec.extend(4..=6);
// vec is now [1, 2, 3, 4, 5, 6]
```

---

## Performance Characteristics

| Operation | Time | Allocation |
|-----------|------|------------|
| `.next()` on slice iter | O(1) | None |
| `.map()` creation | O(1) | None (returns wrapper struct) |
| `.filter()` creation | O(1) | None |
| `.collect::<Vec<_>>()` | O(n) | One allocation (if `size_hint` accurate) |
| `.collect::<HashMap<_,_>>()` | O(n) amortized | Multiple (hash table resizing) |
| `.fold()` | O(n) | None (beyond accumulator) |
| `.take(k)` then `.collect()` | O(k) | One allocation of size k |
| `.chain(a, b)` creation | O(1) | None |
| `.zip(a, b)` creation | O(1) | None |

### Monomorphization

Each adapter type (`Map<Filter<Iter, F1>, F2>`) is a distinct concrete type. The compiler monomorphizes all `next()` calls, enabling full inlining and optimization. The resulting machine code is equivalent to a hand-written loop.

This breaks down with trait objects (`Box<dyn Iterator<Item = T>>`) where dynamic dispatch prevents inlining.

---

## References

- [The Rust Book, Ch. 13.2: Processing a Series of Items with Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [The Rust Book, Ch. 13.4: Comparing Performance: Loops vs. Iterators](https://doc.rust-lang.org/book/ch13-04-performance.html)
- [std::iter module documentation](https://doc.rust-lang.org/std/iter/index.html)
- [Iterator trait API reference](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [IntoIterator trait API reference](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)
- [Rust by Example: Iterators](https://doc.rust-lang.org/rust-by-example/trait/iter.html)
