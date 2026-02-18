# Debugging Exercise — Observer Pattern Bugs

## Symptoms

The code in `buggy.rs` implements a notification dispatcher using the observer pattern. It compiles (mostly) but exhibits several runtime failures:

1. **Compilation error**: One observer function fails with a borrow checker error about mutable and immutable borrows conflicting.
2. **Deadlock**: The program hangs indefinitely during notification when an observer tries to log through a shared logger.
3. **Moved value error**: A `use of moved value` error appears when trying to subscribe the same observer to two different subjects.
4. **Silent data loss**: A channel-based subscriber stops receiving events midway through the program, but no error is printed.

Example output:
```
error[E0502]: cannot borrow `subject` as immutable because it is also borrowed as mutably
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: ...
[hangs indefinitely]
```

## Context

A developer is building a notification dispatch system for a deployment pipeline. The system needs to:
- Notify multiple observers when deployment events occur
- Allow observers to maintain internal state
- Support a shared logger that multiple observers write to
- Use channels for async subscribers

The developer has four bugs to fix, each related to a different aspect of Rust's ownership model interacting with the observer pattern.

## Your Task

1. Read through `buggy.rs` and identify all four bugs
2. For each bug, understand the root cause (not just the symptom)
3. Fix each bug
4. Verify your fixes compile and run correctly
5. Document your debugging process below

## Files

- `buggy.rs` — The code with four intentional bugs
- `solution.md` — Explanation of each bug and fix (don't peek until you've tried!)

---

## Your Debugging Process

### Bug 1: Borrow checker conflict
What I thought the problem was:

Result:

### Bug 2: Deadlock
What I thought the problem was:

Result:

### Bug 3: Moved value
What I thought the problem was:

Result:

### Bug 4: Silent data loss
What I thought the problem was:

Result:

### Lessons
What I learned from this:
