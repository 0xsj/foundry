---
title: Design Patterns
category: patterns
tags: [index, moc]
created: 2026-02-07
updated: 2026-02-07
---

# Design Patterns

Reusable solutions to recurring problems. The goal is not to memorize pattern names but to recognize the problems they solve and reach for them instinctively.

## Modules

### Tier 2 — Foundational

- [[strategy]] — Swap behavior via interchangeable implementations
- [[observer]] — Notify dependents of state changes
- [[factory]] — Controlled construction
- [[builder]] — Step-by-step construction of complex objects
- [[decorator-middleware]] — Compose behavior by wrapping
- [[iterator-generator]] — Lazy evaluation and streaming

### Tier 3 — Integration

- [[repository]] — Abstract data access
- [[dependency-injection]] — Invert control of dependencies
- [[pub-sub]] — Decouple producers from consumers
- [[circuit-breaker]] — Prevent cascading failures
- [[chain-of-responsibility]] — Pass requests through handler chains

## How To Study These

For each pattern:

1. Understand the **problem** it solves before looking at the solution
2. Implement it in your current focus language with a realistic scenario
3. Compare how it looks across languages — some patterns dissolve in certain languages
4. Connect it to the architecture modules — most architectures are compositions of these patterns

## Cross-Cutting Themes

- **Composition over inheritance** — most patterns here favor composition
- **Interface segregation** — small, focused contracts enable most of these patterns
- **Indirection cost** — every pattern adds a layer. Know when the cost is worth it.
- **Language idiom** — a pattern in Java may be a function in Go. The concept transfers, the shape changes.
