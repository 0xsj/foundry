# Debugging Exercise — Strategy Pattern Bugs

## Symptoms

The file `buggy.rs` implements a task executor with pluggable execution strategies (sequential, parallel, priority-based). It does **not compile**. There are four distinct bugs, each related to a different aspect of Rust's trait and strategy pattern mechanics.

When you try to compile:

```
$ rustc buggy.rs
error[E0038]: the trait `ExecutionStrategy` cannot be made into an object
error[E0597]: `strategy` does not live long enough
error[E0277]: `dyn ExecutionStrategy` cannot be sent between threads safely
error: other compilation errors...
```

## Context

A developer is building a task execution framework. The `ExecutionStrategy` trait defines how tasks are scheduled and run. Three strategies exist:

- **SequentialExecutor** — runs tasks one at a time in order
- **PriorityExecutor** — runs tasks sorted by priority
- **BatchExecutor** — groups tasks into batches of N

The code was ported from a TypeScript version where these patterns worked fine. The developer assumed Rust traits work like TypeScript interfaces — they do not, and the code has four bugs that expose fundamental differences.

## Your Task

1. Read through `buggy.rs` and identify all four bugs
2. Fix each bug so the code compiles and the `main` function runs correctly
3. Do NOT rewrite the architecture — fix the bugs in place
4. Document your debugging process below

## Files

- `buggy.rs` — The code with four bugs
- `solution.md` — Explanation of each bug and fix (don't peek until you've tried!)

## Bug Inventory

There are exactly **four bugs**:

| # | Category | Hint |
|---|----------|------|
| 1 | Object safety | A trait method prevents `dyn Trait` usage |
| 2 | Lifetimes | A borrowed strategy doesn't live long enough |
| 3 | Thread safety | A trait object is missing required bounds |
| 4 | Wrong dispatch | Static dispatch used where dynamic dispatch is needed |

---

## Your Debugging Process

### Bug 1
What I thought the problem was:

What I changed:

### Bug 2
What I thought the problem was:

What I changed:

### Bug 3
What I thought the problem was:

What I changed:

### Bug 4
What I thought the problem was:

What I changed:

### Lessons
What I learned from this exercise:
