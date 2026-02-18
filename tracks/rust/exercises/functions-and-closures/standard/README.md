# Standard Exercise: Data Transformation Pipeline

## Scenario

You're building a lightweight ETL (extract-transform-load) component for a data ingestion
service. Raw records arrive as strings from various sources (CSV files, webhooks, message
queues) and must pass through a configurable pipeline of transformation steps before being
stored. Each step is a closure: filter out invalid records, normalize fields, validate
constraints, and enrich data. The pipeline itself chains these steps together.

## Brief

Implement a `Pipeline<T>` struct that accepts a series of transformer closures, then applies
them in sequence to a `Vec<T>`. Transformers either keep, modify, or discard each item.
Also implement standalone higher-order functions that produce common transformer closures.

## Acceptance Criteria

1. **`Transformer<T>` type alias**
   - `type Transformer<T> = Box<dyn Fn(T) -> Option<T>>;`
   - `None` means the record is discarded. `Some(t)` means keep (possibly modified) record.

2. **`Pipeline<T>` struct** with:
   - `transformers: Vec<Transformer<T>>` — ordered list of steps

3. **`Pipeline::new()`** — creates an empty pipeline

4. **`Pipeline::add_step(self, step: impl Fn(T) -> Option<T> + 'static) -> Pipeline<T>`**
   - Builder method: takes ownership of `self`, returns a new `Pipeline` with the step added
   - Takes any closure matching the signature (boxes it internally)

5. **`Pipeline::run(self, records: Vec<T>) -> Vec<T>`**
   - Consumes the pipeline and the records
   - Passes each record through all steps in order
   - If any step returns `None`, the record is dropped immediately (does not proceed to later steps)
   - Returns only the records that survived all steps

6. **`keep_if(predicate: impl Fn(&T) -> bool + 'static) -> Transformer<T>`**
   - Returns a `Transformer<T>` that keeps records where `predicate` returns true, discards otherwise
   - The predicate borrows the record; the transformer owns it

7. **`map_field(transform: impl Fn(T) -> T + 'static) -> Transformer<T>`**
   - Returns a `Transformer<T>` that applies `transform` to each record (always keeps it)

8. **`take_first(n: usize) -> Transformer<T>`**
   - Returns a `Transformer<T>` that keeps only the first `n` records
   - Hint: this requires captured mutable state — the transformer must count how many it has seen

9. **`apply_pipeline(records: Vec<T>, steps: Vec<impl Fn(T) -> Option<T>>) -> Vec<T>`** (standalone function)
   - Applies a list of closures to the records in sequence
   - This version does not box the closures — use a generic parameter instead
   - Note: all steps must have the same concrete type (unlike `Pipeline` which uses trait objects)

## Constraints

- No external crates — stdlib only
- All tests in the starter file must pass
- `Pipeline::add_step` must use the builder pattern (takes `self`, returns `Self`)
- `take_first` must use a `move` closure with mutable captured state

## Hints

<details>
<summary>Hint 1: Type alias for Transformer</summary>

`type Transformer<T> = Box<dyn Fn(T) -> Option<T>>;`

This is a heap-allocated closure (trait object). Boxing allows you to store closures
of different concrete types in the same `Vec`. You create one with:
`Box::new(|record| Some(record))`

</details>

<details>
<summary>Hint 2: Builder pattern with ownership</summary>

```rust
fn add_step(mut self, step: impl Fn(T) -> Option<T> + 'static) -> Pipeline<T> {
    self.transformers.push(Box::new(step));
    self
}
```

The `'static` bound is needed because `Box<dyn Fn(T) -> Option<T>>` requires the
closure to own all its captures (no borrowed references with shorter lifetimes).

</details>

<details>
<summary>Hint 3: take_first with mutable captured state</summary>

```rust
pub fn take_first<T>(n: usize) -> Transformer<T> {
    let mut seen = 0usize;
    Box::new(move |record| {
        if seen < n {
            seen += 1;
            Some(record)
        } else {
            None
        }
    })
}
```

The closure mutates `seen`. This means it implements `FnMut`, not `Fn`. That's fine —
`Box<dyn Fn>` requires `Fn`, but `Box<dyn FnMut>` works here. Adjust the type alias
or use `FnMut` directly.

</details>

<details>
<summary>Hint 4: apply_pipeline without boxing</summary>

If all steps have the same concrete type (e.g., all produced by `keep_if`), you don't
need trait objects. But closures produced by different factory functions have different
types. To make `apply_pipeline` work with any single closure type:

```rust
fn apply_pipeline<T, F: Fn(T) -> Option<T>>(records: Vec<T>, steps: Vec<F>) -> Vec<T>
```

The trade-off: all steps in `steps` must be the same concrete closure type. For
heterogeneous steps, you need `Vec<Box<dyn Fn>>`.

</details>
