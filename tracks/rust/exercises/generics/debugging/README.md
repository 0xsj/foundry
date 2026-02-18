# Debugging Exercise: Generic Data Pipeline

## Scenario

You've been handed a generic data processing pipeline module that doesn't compile. The module
is supposed to provide:

- A `largest_by_score` function that finds the highest-scoring item in a slice
- A `parse_all` function that parses a list of strings into a chosen type
- A `Processor` trait with a `process_batch` method, and a dynamic dispatch wrapper around it
- A `Cache` struct that holds borrowed references with generic lifetime annotations

The code has four bugs — one per section. Each produces a distinct compiler error about generics.

## Symptoms

Running `rustc --test buggy.rs` produces compiler errors. The errors involve:

1. A function using an operator on a generic type without the trait bound that enables it
2. A type inference failure where the compiler cannot determine which `T` to use — turbofish required
3. A generic method on a trait that makes the trait non-object-safe, breaking `Box<dyn Trait>` usage
4. A generic struct holding a reference without the lifetime bound needed to satisfy the compiler

Work through each error one at a time. Rust's error messages for generics are especially precise —
they will tell you exactly which bound is missing or which lifetime constraint is needed.

## What Not To Do

- Do not remove the tests
- Do not change function signatures unless the fix requires it (some fixes do require a signature change)
- Do not work around the errors by adding unnecessary `clone()` calls or changing types to `String`

Fix the root cause of each error. The lesson is in understanding why each error occurs.
