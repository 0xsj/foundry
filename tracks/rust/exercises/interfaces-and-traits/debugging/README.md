# Debugging Exercise: Notification Pipeline

## Scenario

A teammate built a notification pipeline for an ops tooling service. The code is supposed
to support pluggable formatters and senders, store them in a central registry, and dispatch
alerts. When you try to compile it, you get several errors — and even after fixing those,
there's one runtime bug hiding in the logic.

## Symptoms

Run with:

```
rustc --test buggy.rs && ./buggy
```

You will see multiple compile errors. Fix them one at a time, recompiling after each fix.
After all compile errors are resolved, run the tests — one will still fail.

## What You're Looking For

There are **4 bugs** in this file. Each maps to a core trait concept from this module:

- A trait that cannot be used as `dyn Trait` (object safety violation)
- A missing `Display` implementation causing a confusing downstream error
- An orphan rule violation
- Using `&dyn Trait` where `Box<dyn Trait>` is required (ownership issue)

## Constraints

- Do not change the test code
- Fix the implementation only — types, impls, and function signatures may be changed
- Each fix should be minimal — don't redesign the whole file
