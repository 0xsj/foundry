# Solution: Telemetry Aggregator Bugs

## Bug 1: `[]` indexing causes runtime panic in `aggregate`

### Root Cause

```rust
let mut max = samples[10].value;  // BUG: panics if len < 11
```

`samples[10]` uses unchecked indexing. If the slice has fewer than 11 elements, Rust
panics with "index out of bounds." The code appears to want the first element as the
initial maximum, but hardcoded index 10 is clearly wrong.

### Why It Compiles

`[]` indexing is valid Rust — the panic happens at runtime, not compile time. The
compiler sees a valid expression; the bounds check is a runtime invariant.

### Fix

Use the first element of the slice as the initial maximum. Since we already check
`is_empty()`, indexing `[0]` is safe here. Alternatively, use an iterator-based approach
that avoids indexing entirely:

```rust
fn aggregate(samples: &[Sample]) -> Option<f64> {
    samples.iter().map(|s| s.value).reduce(f64::max)
}
```

Or the explicit version:

```rust
fn aggregate(samples: &[Sample]) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    let mut max = samples[0].value;   // use index 0, not 10
    for sample in samples {
        if sample.value > max {
            max = sample.value;
        }
    }
    Some(max)
}
```

### Lesson

- Use `.get(idx)` when you're not certain the index is valid — it returns `Option<&T>`
  instead of panicking.
- `[]` is appropriate when you have a proof the index is valid (e.g., immediately after
  checking `is_empty()`). When in doubt, use `.get()`.
- Iterator methods like `.reduce()` and `.max_by()` often eliminate the need to index
  directly.

---

## Bug 2: Iterating immutably while trying to mutate in `apply_multiplier`

### Root Cause

```rust
for sample in samples.iter() {     // immutable borrow: yields &Sample
    sample.value *= factor;        // ERROR: cannot assign to `sample.value`, as `sample` is a `&Sample`
}
```

`iter()` yields `&Sample` — an immutable reference. You cannot write to `sample.value`
through an immutable reference. The fix is to use `iter_mut()`, which yields `&mut Sample`.

### Why This Specific Error

The borrow checker enforces that `&T` references are read-only. `*=` is an assignment,
which requires a mutable reference (`&mut T`).

### Fix

```rust
fn apply_multiplier(samples: &mut Vec<Sample>, factor: f64) {
    for sample in samples.iter_mut() {   // yields &mut Sample
        sample.value *= factor;          // now valid — we have exclusive mutable access
    }
}
```

Or equivalently, using `&mut collection` in the for loop (desugars to `iter_mut()`):

```rust
for sample in samples {          // samples is &mut Vec<Sample>, desugars to iter_mut()
    sample.value *= factor;
}
```

### Lesson

- `iter()` → `&T` → read only
- `iter_mut()` → `&mut T` → read and write
- `into_iter()` → `T` → move out (collection consumed)
- If you need to modify elements in place, always use `iter_mut()`.
- In Go, `for i, v := range slice` copies each element into `v`. To modify in place,
  you write `slice[i] = ...`. In Rust, `iter_mut()` gives you direct mutable access
  without needing a separate index.

---

## Bug 3: Entry API misuse in `build_index` — always resets count to 1

### Root Cause

```rust
if !index.contains_key(&sample.agent_id) {
    index.insert(sample.agent_id, 1);
} else {
    index.insert(sample.agent_id, 1);  // BUG: inserts 1 instead of incrementing
}
```

Both branches insert `1`. When an agent_id is seen for the second time, the count is
reset to 1 instead of being incremented. Every agent ends up with a count of 1 regardless
of how many samples it sent.

Beyond the logic bug, this pattern also has a borrow issue: `contains_key` holds an
immutable borrow of the map, and `insert` requires a mutable borrow. The compiler allows
this here because the borrows don't overlap, but the pattern is fragile.

### Fix: Use the Entry API

```rust
fn build_index(samples: &[Sample]) -> HashMap<u32, u32> {
    let mut index: HashMap<u32, u32> = HashMap::new();
    for sample in samples {
        *index.entry(sample.agent_id).or_insert(0) += 1;
    }
    index
}
```

`entry(key).or_insert(0)` returns `&mut u32` — 0 if the key was absent and just
inserted, or the existing value if present. Dereferencing with `*` and adding `+= 1`
increments the count in both cases.

### Lesson

- The `contains_key` + `insert` pattern is a code smell in Rust. The entry API exists
  precisely to replace it.
- Entry API: one lookup, no borrow conflicts, correct in every case.
- This is idiomatic counting in Rust:
  ```rust
  *map.entry(key).or_insert(0) += 1;
  ```
  The equivalent in Go is `m[k]++` (Go returns zero value for missing keys).
  In TypeScript: `map.set(k, (map.get(k) ?? 0) + 1)`.

---

## Bug 4: Spurious `collect()` produces wrong type in `top_metrics`

### Root Cause

```rust
pairs.iter().collect()  // yields &&str and &f64 — not what the return type expects
```

`pairs` is a `Vec<(&str, f64)>`. Calling `.iter()` on it yields `&(&str, f64)` —
references to the tuples. `.collect()` would produce `Vec<&(&str, f64)>`, not
`Vec<(&str, f64)>`. This doesn't match the declared return type, so the code fails to
compile.

The pairs Vec was already built correctly on the lines above. The final `collect()` is
entirely spurious — `pairs` is already the right type.

### Fix

```rust
fn top_metrics(samples: &[Sample], n: usize) -> Vec<(&str, f64)> {
    let mut pairs: Vec<(&str, f64)> = samples.iter()
        .map(|s| (s.name.as_str(), s.value))
        .collect();

    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    pairs.truncate(n);

    pairs   // just return pairs directly — no re-collect needed
}
```

### Lesson

- When you see a compile error on `.collect()`, check what type is being inferred vs
  what the return type expects. The turbofish (`.collect::<Vec<_>>()`) can help diagnose.
- `.iter()` on a `Vec<T>` yields `&T`, not `T`. If you want to re-collect a `Vec<T>`
  back into a `Vec<T>`, use `.into_iter().collect()` — but in most cases you don't need
  to re-collect at all.
- In TypeScript, `arr.map(fn)` always returns a new array. In Rust, iterators are lazy
  and `.collect()` is the explicit step that materializes the result. This makes the
  "I already have the result" case more visible.

---

## Summary

| Bug | Category | Rust Concept | Prevention |
|-----|----------|-------------|------------|
| `samples[10]` on short slice | Runtime panic | Indexing vs `.get()` | Use `.get()` for uncertain bounds; prefer iterator methods |
| `iter()` then mutate | Compile error | Borrow rules, `iter_mut()` | Read-only iteration = `iter()`, mutation = `iter_mut()` |
| `insert(1)` instead of increment | Logic error | Entry API | Always use entry API for count-or-insert patterns |
| Spurious `collect()` on already-built Vec | Compile error | Iterator types, `&T` vs `T` | Don't re-collect; trust the Vec you already have |

## Related Pitfalls

- [[rust-vec-index-panic]] — `[]` vs `.get()` on Vec and slices
- [[rust-iter-vs-iter-mut]] — choosing the right iteration mode
- [[rust-entry-api]] — idiomatic HashMap counting and insertion
- [[rust-collect-type-mismatch]] — debugging `.collect()` type errors
