# Debugging Exercise: Event Processor

## Scenario

You've been handed a buggy event processing module. It's supposed to provide:
- A `deduplicate` function that removes consecutive duplicate events using a closure
- A `batch_process` function that splits events into batches and applies a processor closure
- A `make_counter` function that returns a closure tracking invocation count
- A `format_events` function that returns a closure for formatting event strings

The code doesn't compile. The compiler is reporting errors — your job is to understand
what each error is telling you and fix the root cause (not just silence the error).

## Symptoms

Running `rustc --test buggy.rs` produces compiler errors. The errors involve:

1. A closure passed to a function that expects `Fn`, but the closure requires `FnMut`
2. A `move` closure that tries to borrow data it doesn't own after the move
3. A function trying to return a closure but the return type doesn't compile
4. A function using a closure to capture a reference that doesn't live long enough

Work through each error one at a time. Read the compiler message carefully — Rust's
error messages tell you the exact problem and often suggest the fix.

## What Not To Do

- Do not change the test assertions
- Do not work around errors by cloning everything blindly
- Do not change function signatures unless the fix genuinely requires it

Fix the bugs in the implementation functions only.
