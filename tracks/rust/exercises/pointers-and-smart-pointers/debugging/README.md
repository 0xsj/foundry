# Debugging Exercise: Notification Dispatcher

## Scenario

A team built a notification dispatch system where channels (email, SMS, push) share
configuration state and optionally subscribe to each other's delivery receipts. The
code has four bugs related to smart pointer selection and usage — some fail at compile
time, some fail at runtime, and one silently leaks memory.

## Symptoms

Run the tests with:

```
rustc --test buggy.rs && ./buggy
```

You will see compile errors and at least one runtime panic. Fix all issues so that
the test suite passes completely and the program does not leak memory.

## What You're Looking For

There are **4 bugs** in this file. They are all related to smart pointer concepts:

- An `Rc` cycle that causes a memory leak (no compile error — silent leak)
- A `RefCell` double borrow that panics at runtime
- An `Rc` that cannot be sent across a thread boundary
- A `Box` used where multiple owners are needed

Each bug is a distinct category of mistake. Fix them one at a time, re-compiling
after each fix.

## Constraints

- Do not change the test code
- Fix the types and implementations — do not restructure the overall design
- The memory leak fix requires adding `Weak` in the right place
