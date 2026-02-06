---
title: System Design
category: system-design
tags: [index, moc]
created: 2026-02-07
updated: 2026-02-07
---

# System Design

How to build reliable, scalable, observable systems. This bridges the gap between code-level patterns and production infrastructure.

## Modules

### Tier 4

- [[resilience-patterns]] — Circuit breakers, retries, bulkheads, graceful degradation
- [[caching]] — Strategies, eviction, invalidation, local vs distributed
- [[load-balancing]] — Distribution strategies, health checking, connection pooling
- [[message-queues]] — Queue semantics, Kafka, RabbitMQ, NATS, backpressure
- [[database-patterns]] — Indexing, query plans, transactions, migrations, replication
- [[observability]] — Structured logging, metrics, tracing, OpenTelemetry
- [[security-fundamentals]] — Auth, JWT, OAuth2, input validation, OWASP

## Prerequisites

System design modules assume comfort with:

- [[error-handling]] and [[concurrency]] from fundamentals
- [[repository]], [[circuit-breaker]], [[decorator-middleware]] from patterns
- Basic familiarity with at least one architecture pattern

## How To Study These

These modules are best learned by building. Each topic connects to a build in `/builds/`:

- Resilience → HTTP server with circuit breakers and retries
- Message queues → Kafka microservice
- Caching → Add caching layer to an existing build
- Observability → Instrument any build with tracing and metrics

## Cross-Cutting Themes

- **Failure is normal** — design for it, don't prevent it
- **Measure first** — never optimize without data
- **Operational cost** — every component you add is something you maintain
- **Tradeoff awareness** — CAP, consistency vs availability, latency vs throughput
