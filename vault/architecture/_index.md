---
title: Architecture Patterns
category: architecture
tags: [index, moc]
created: 2026-02-07
updated: 2026-02-07
---

# Architecture Patterns

How to structure entire systems. These are compositions of design patterns applied at a larger scale.

## Modules

### Tier 4 — Foundations

- [[layered]] — Traditional presentation / business / data layers
- [[event-driven]] — Architecture built around event production and consumption
- [[api-gateway-bff]] — Edge routing, aggregation, client-specific APIs

### Tier 5 — Advanced

- [[hexagonal]] — Ports and adapters, domain isolation
- [[clean-architecture]] — Concentric layers with the dependency rule
- [[cqrs]] — Separate read and write models
- [[event-sourcing]] — Store state as a sequence of events
- [[microservices]] — Service decomposition and distributed systems

## Prerequisites

Architecture modules assume familiarity with several design patterns. At minimum:

- [[repository]], [[strategy]], [[dependency-injection]] for layered and hexagonal
- [[observer]], [[pub-sub]] for event-driven and CQRS
- [[circuit-breaker]] for microservices

## How To Study These

1. Start with [[layered]] — it's the most common and the easiest to reason about
2. Understand its limitations, then see how [[hexagonal]] addresses them
3. [[cqrs]] and [[event-sourcing]] build on hexagonal
4. [[microservices]] is where everything converges

Each architecture module has a corresponding build in `/builds/` that implements a working system.

## Cross-Cutting Themes

- **Dependency direction** — who depends on whom defines the architecture
- **Testability** — good architecture makes testing easy, not hard
- **Complexity budget** — every layer of indirection must earn its keep
- **Evolution** — systems start simple and grow. Know the upgrade paths.
