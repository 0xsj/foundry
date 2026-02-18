# Code Review Exercise: Generic Cache Implementation

## Context

A teammate opened a PR adding a generic, expiration-aware cache to a service that handles
webhook delivery. The cache stores recently-resolved webhook endpoints so the service doesn't
re-resolve them on every delivery attempt. The code compiles and the tests pass, but the
generics design has several issues that will create friction for callers and make the code
harder to evolve.

## Your Task

Review `proposed.rs` as if it were a real PR. Look for:

1. **Too many type parameters** — a struct with more generics than it needs, forcing callers
   into verbose turbofish annotations
2. **Over-restrictive trait bounds** — bounds on struct definitions that should only exist on
   `impl` blocks
3. **Unnecessary `PhantomData`** — a phantom type parameter that adds complexity without value
4. **Associated type opportunity** — a place where an associated type would simplify the API
   compared to a generic parameter
5. **Monomorphization bloat risk** — generic code that will generate many specialized copies
   where a single implementation would suffice

Record your findings in `my-review.md` using the Critical / Major / Minor structure.

## Review Checklist

Work through the code in order:

- [ ] Struct definition — how many type parameters? Are all necessary?
- [ ] Trait bounds on the struct — should any move to `impl` blocks?
- [ ] `PhantomData` usage — is it serving a real purpose?
- [ ] The `Expiry` trait — generic parameter vs associated type?
- [ ] Function signatures — are bounds minimal or over-constrained?
- [ ] The `cache_stats` function — could it avoid being generic?

## How to Run the Code

```
rustc proposed.rs && ./proposed
```
