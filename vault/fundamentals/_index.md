---
title: Fundamentals
category: fundamentals
tags: [index, moc]
created: 2026-02-07
updated: 2026-02-07
---

# Fundamentals

The building blocks. Everything else depends on these.

## Modules

### Tier 1 — Start Here

- [[variables-and-types]] — Primitive types, composite types, type systems
- [[control-flow]] — Conditionals, loops, pattern matching, guard clauses
- [[functions-and-closures]] — Signatures, closures, higher-order functions
- [[modules-and-packages]] — Code organization, visibility, imports

### Tier 2 — Core Mechanics

- [[error-handling]] — How each language models failure
- [[interfaces-and-traits]] — Contracts, polymorphism, structural vs nominal typing
- [[generics]] — Parameterized types and functions, constraints
- [[serialization]] — JSON, protobuf, schema validation, serde
- [[testing-fundamentals]] — Unit tests, table-driven tests, test doubles

### Tier 3 — Advanced

- [[concurrency]] — Goroutines, async/await, tokio, virtual threads
- [[memory-and-ownership]] — Ownership, borrowing, GC, stack vs heap

## Progression

Start with Tier 1 in your current focus language. Tier 2 unlocks after completing functions-and-closures. Tier 3 unlocks after error-handling and interfaces-and-traits.

## Cross-Cutting Themes

These themes recur across all fundamentals modules:

- **Type safety** — how much does the compiler catch vs runtime?
- **Explicitness vs convenience** — Go's verbosity vs Python's brevity
- **Zero values and defaults** — what happens when you don't initialize?
- **Composition vs inheritance** — how does each language encourage structuring code?
