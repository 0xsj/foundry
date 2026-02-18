# Solution: Data Transformation Pipeline

## Approach

The key design decision is using `Box<dyn FnMut(T) -> Option<T>>` as the transformer type.
This allows the `Vec<Transformer<T>>` inside `Pipeline` to hold closures of different concrete
types (a `keep_if` closure and a `take_first` closure are different types, but both erase to the
same `Box<dyn FnMut>` trait object).

The core of `Pipeline::run` uses `Option::and_then` to short-circuit: once a record is `None`,
subsequent transformers are never called. This is implemented with `fold` starting from `Some(record)`.

`take_first` demonstrates the pattern that most distinguishes `FnMut` from `Fn`: mutable captured
state. The `seen` counter is owned by the closure, and incrementing it on each call is why the
closure is `FnMut` rather than `Fn`.

## Key Decisions

**Why `FnMut` instead of `Fn` in the Transformer type alias?**
`take_first` mutates `seen` on every call. A closure that mutates captured state implements `FnMut`
but not `Fn`. Using `Fn` in the type alias would make `take_first` impossible to express.

**Why `Box<dyn FnMut>` instead of generics in Pipeline?**
A `Pipeline` with generic closure types (`struct Pipeline<T, F: FnMut(T) -> Option<T>>`) can only
hold one concrete closure type in `transformers`. You couldn't chain a `keep_if` step (one type) and
a `take_first` step (a different type) in the same pipeline. `Box<dyn FnMut>` erases both to the same
type, enabling heterogeneous steps at the cost of one heap allocation per step.

**Why does `apply_pipeline` use a generic bound instead of trait objects?**
To demonstrate the contrast. With `F: FnMut(T) -> Option<T>`, all steps must be the same concrete
type — the function is monomorphized. This is zero-cost but inflexible. The caller test uses
`Vec<Box<dyn FnMut(...)>>` as the concrete `F` to work around the single-type limitation.

## Variant Approaches

| Approach | Trade-off |
|---|---|
| `Box<dyn FnMut>` (reference solution) | Heap allocation per step; allows heterogeneous steps |
| Generic `F: FnMut` struct field | Zero cost; only one closure type per pipeline |
| `Vec<fn(T) -> Option<T>>` (fn pointers) | Cheapest; only non-capturing closures, no `take_first` |
| Enum of step types | Zero-cost; explicit variant per step type; verbose to extend |

## Connection to Bigger Picture

The `Option::and_then` chain in `run` is the `Maybe` monad pattern from functional programming
(you'll see it explicitly in Haskell). Rust's `Option` and `Result` types make monadic chaining
idiomatic without needing special syntax.

The `Pipeline` builder pattern here also previews the Builder pattern, which you'll formalize in
the patterns module. Notice how taking `self` (not `&mut self`) in `add_step` means the compiler
statically ensures you don't reuse a partially-built pipeline after consuming it.
