# Rust Track

Expanding perspective. Rust forces you to think about ownership, lifetimes, and memory layout in ways that improve how you think in every other language. The borrow checker is the teacher.

## Key Language Characteristics

- Ownership and borrowing (no garbage collector)
- Algebraic data types (enums with data)
- Pattern matching with exhaustiveness checking
- Traits for polymorphism (no inheritance)
- Result<T, E> and Option<T> for error handling
- Zero-cost abstractions
- Fearless concurrency through ownership rules

## What To Pay Attention To

- **Fight the borrow checker less.** If it's hard, your data model might be wrong. Restructure before reaching for `Rc<RefCell<T>>`.
- **Enums are powerful.** They replace union types, optional values, error types, and state machines.
- **The `?` operator is your friend.** Propagate errors idiomatically.
- **Traits are not interfaces.** They can carry default implementations, associated types, and be used for operator overloading.
- **Iterators over indexing.** Rust's iterator combinators are zero-cost and more idiomatic than manual loops.

## Directory Structure

```
rust/
├── fundamentals/    # Language basics exercises
├── patterns/        # Design pattern implementations
├── exercises/       # Generated exercises
└── builds/          # Larger architecture projects
```

## Tools & Setup

- `Cargo.toml` at the track root (workspace)
- Each exercise/build as a workspace member
- `cargo test` to run all tests
- `clippy` for linting — treat warnings as errors
- `rustfmt` for formatting

## Idiomatic Conventions

- Use `impl` blocks, not standalone functions when there's a clear receiver
- Prefer borrowing (`&T`) over ownership transfer unless you need to consume
- Use `From`/`Into` for type conversions
- Derive `Debug`, `Clone`, `PartialEq` by default
- Use `thiserror` for library errors, `anyhow` for application errors
