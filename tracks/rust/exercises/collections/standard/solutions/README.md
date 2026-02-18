# Solution Notes: Log Analyzer

## Approach

The solution maintains three collections:

1. `Vec<LogEntry>` for ordered, indexed storage of all entries
2. `HashMap<&'static str, u32>` for O(1) level and source counting at ingest time
3. `BTreeMap` produced on demand from the HashMap for sorted output

Counts are maintained incrementally at ingest time (O(1) per entry) rather than computed
on demand (O(n) per query). This is the right tradeoff for high-ingest, high-query systems.

## Key Decisions

### Entry API in `ingest`

```rust
*self.level_counts.entry(entry.level).or_insert(0) += 1;
```

The entry API does a single map lookup, inserts a default if absent, and returns a
`&mut u32`. Dereferencing with `*` increments the actual stored value. The alternative
(`contains_key` + `insert` + `get_mut`) would do up to three lookups and won't compile
cleanly due to borrow rules.

### `&'static str` keys

Level names and service names are string literals baked into the binary. Using
`&'static str` as the HashMap key means no heap allocation for keys, and the HashMap
can store them directly without any ownership complexity. The tradeoff: you can't use
this approach for dynamically constructed strings (those need `String` keys).

### Sorting with `.then()`

```rust
pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
```

`Ordering::then` chains comparisons: if the primary comparison (count, descending) is
`Equal`, fall back to the secondary (name, ascending). More composable than a manual
`if` chain.

### BTreeSet for `sources_with_errors`

Collecting filtered sources into a `BTreeSet` is the idiomatic way to get a sorted,
deduplicated set in one pass. The alternative — collect into a `Vec`, sort, and dedup —
requires an extra allocation and two passes.

### `most_frequent_errors` lifetime

The HashMap in `most_frequent_errors` maps `&str` slices that point into the
`message: String` fields stored in `self.entries`. These lifetimes are valid as long as
`&self` is borrowed, so returning `Vec<(&str, u32)>` compiles correctly. If we needed
the result to outlive the analyzer, we'd need `String` keys.

## Variants

### Variant A: Lazy counting (simpler, slower)

Instead of maintaining `level_counts` and `source_counts` incrementally, compute them
on demand in `level_summary` and `top_sources`:

```rust
fn level_summary(&self) -> BTreeMap<&'static str, u32> {
    let mut counts: HashMap<&'static str, u32> = HashMap::new();
    for entry in &self.entries {
        *counts.entry(entry.level).or_insert(0) += 1;
    }
    counts.into_iter().collect()
}
```

Trade-off: simpler struct, but O(n) per query instead of O(1). Fine for small log sets.
The incremental approach in the reference solution is better for high-query workloads.

### Variant B: `entries_in_range` with binary search

If entries are guaranteed to arrive in timestamp order (monotonic), you could binary
search for the start and end indices:

```rust
fn entries_in_range(&self, start_ms: u64, end_ms: u64) -> &[LogEntry] {
    let start = self.entries.partition_point(|e| e.timestamp_ms < start_ms);
    let end   = self.entries.partition_point(|e| e.timestamp_ms <= end_ms);
    &self.entries[start..end]
}
```

O(log n) instead of O(n), and returns a slice instead of a Vec (no allocation).
Requires the monotonic-timestamp invariant to be maintained by the caller.

## Complexity

| Method | Time | Notes |
|---|---|---|
| `ingest` | O(1) amortized | Vec push + 2x HashMap entry |
| `level_summary` | O(L log L) | L = distinct levels (always ≤ 4) |
| `top_sources` | O(S log S) | S = distinct sources |
| `errors` | O(n) | Linear scan |
| `most_frequent_errors` | O(E log E) | E = distinct error messages |
| `sources_with_errors` | O(n) | Linear scan + BTreeSet insert |
| `entries_in_range` | O(n) | Linear scan |
