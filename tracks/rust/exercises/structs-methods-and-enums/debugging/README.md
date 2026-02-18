# Debugging Exercise: Event Processor

## Scenario

A team is building a webhook event processor that classifies incoming events,
transforms them into normalized records, and dispatches them for downstream handling.
The code was written quickly and has several bugs related to struct and enum ownership.

## Symptoms

Run the tests with:

```
rustc --test buggy.rs && ./buggy
```

You will see compile errors and at least one test failure. Fix all issues so that
the test suite passes completely.

## What You're Looking For

There are **4 bugs** in this file. They are all related to concepts from this module:

- How match arms interact with ownership
- When `#[derive(Clone)]` is required
- The difference between `self` and `&self` as method receivers
- Non-exhaustive match expressions on enums

Each bug is a distinct category of mistake. Fix them one at a time, re-compiling
after each fix.

## Constraints

- Do not change the test code
- Do not change the struct or enum definitions
- Only fix the implementation code (functions, methods, match expressions)
