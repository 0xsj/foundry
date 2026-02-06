---
title: Data Structures & Algorithms
category: dsa
tags: [index, moc]
created: 2026-02-07
updated: 2026-02-07
---

# Data Structures & Algorithms

Foundational computer science. The goal is fluency in recognizing which structure or technique fits a problem, not memorizing solutions.

## Modules

### Tier 1

- [[arrays-and-hashing]] — Array manipulation, hash maps, sets, two pointers, sliding window

### Tier 2

- [[linked-lists]] — Pointer manipulation, fast/slow pointers, reversal
- [[stacks-and-queues]] — LIFO/FIFO, monotonic stacks, priority queues
- [[sorting]] — Merge, quick, heap, counting, radix. Stability and complexity
- [[searching]] — Binary search variations, search space reduction

### Tier 3

- [[trees]] — Binary trees, BSTs, traversals, tries
- [[graphs]] — BFS, DFS, topological sort, shortest path
- [[recursion-and-backtracking]] — Recursive decomposition, constraint satisfaction

### Tier 4

- [[dynamic-programming]] — Memoization, tabulation, common DP patterns

## How To Study These

Each DSA module lives in `/dsa/problems/` with implementations across all five languages. For each problem:

1. Understand the **brute force** approach first
2. Identify the **bottleneck** — what's slow and why
3. Apply the relevant **technique** to optimize
4. Analyze **time and space complexity** — be able to explain it, not just state it
5. Implement in your current focus language, then compare across languages

## Cross-Cutting Themes

- **Time/space tradeoffs** — almost every optimization trades one for the other
- **Pattern recognition** — most problems are variations of a small set of techniques
- **Language differences** — Rust's ownership makes linked lists hard. Python's lists aren't arrays. Know what your language gives you.
- **Practical relevance** — connect DSA to real systems (LRU cache, task scheduling, dependency resolution)
