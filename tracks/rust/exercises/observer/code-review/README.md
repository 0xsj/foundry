# Code Review Exercise — Event System

## Overview

**Scenario:** A colleague has implemented a generic event system for your Rust microservice framework. The PR adds an `EventBus` that services can use to publish and subscribe to typed domain events. The system needs to work across threads (handlers may be registered from different services running on different threads).

**Your Task:** Review the proposed changes and provide feedback on:
- Correctness and bugs
- Thread safety
- Trait object safety
- Lifetime and ownership issues
- Memory leak potential
- Idiomatic Rust practices
- Alternative approaches

---

## Context

**Codebase:** A Rust microservice framework used by multiple internal teams. Services register event handlers at startup, and the event bus dispatches domain events during request processing. The bus must be `Send + Sync` (shared across request-handling threads via `Arc`).

**Feature:** Replace the existing channel-based event dispatch with a synchronous callback-based `EventBus` for lower latency in hot paths.

**Files Changed:** `proposed.rs` (new file — the entire event bus implementation)

---

## The Pull Request

### PR Description
```
feat: Add synchronous EventBus for low-latency event dispatch

Replaces the channel-based dispatch for performance-critical paths.
Handlers are registered as trait objects and called synchronously
during emit(). Supports typed events via generics.

Benchmarks show 3x improvement over channel dispatch for small events.
```

### Changes
See `proposed.rs` for the full implementation.

---

## Your Review

Use `my-review.md` to document your findings:

1. **Critical Issues** -- Bugs, thread safety violations, soundness problems
2. **Major Concerns** -- Design problems, lifetime issues, memory leaks
3. **Minor Suggestions** -- Style, naming, small improvements
4. **Positive Feedback** -- What was done well

---

## Hints (Progressive)

<details>
<summary>Hint 1: Check the trait bounds</summary>

Look at the `EventHandler` trait. Is it object-safe? Can you actually store `dyn EventHandler<E>` in a `Vec`? Check the associated type and generic constraints.

</details>

<details>
<summary>Hint 2: Thread safety</summary>

The PR description says the bus needs to be `Send + Sync`. Look at what smart pointer types are used for the handlers. Are they thread-safe? What about the `Rc` usage?

</details>

<details>
<summary>Hint 3: Lifetime of stored references</summary>

Look at how observers are stored. Are there any borrowed references (`&dyn`) that create lifetime issues? What happens when the thing behind the reference is dropped?

</details>

<details>
<summary>Hint 4: Observer cleanup</summary>

What happens when a handler is no longer needed? Is there a way to unsubscribe? What happens to dead handlers?

</details>

---

## Learning Objectives

This exercise practices:
- [ ] Identifying thread safety issues (`Rc` vs `Arc`, `Send`/`Sync` bounds)
- [ ] Recognizing trait object safety violations
- [ ] Spotting lifetime issues with borrowed references in long-lived collections
- [ ] Evaluating memory leak potential in observer patterns
- [ ] Understanding `Send`/`Sync` requirements for shared state
- [ ] Giving constructive, specific code review feedback

---

## Notes

**Difficulty:** Medium-Hard

**Estimated Time:** 20-30 min

**Related Concepts:** [[observer]], [[interfaces-and-traits]], [[pointers-and-smart-pointers]]
