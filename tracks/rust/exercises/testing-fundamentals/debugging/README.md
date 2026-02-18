# Debugging Exercise: Broken Test Suite

## Scenario

A colleague wrote tests for the rate limiter, but the test suite has problems. Some tests
pass when they shouldn't. Some tests fail for the wrong reasons. Some tests don't test
anything. One test corrupts the test environment for other tests running in parallel.

The bugs are all in the **test code**, not the production code. The rate limiter itself
is correct.

## Symptoms

1. A test that should catch a regression doesn't — the assertion is written backwards.
2. A test always passes, even when the behavior it's supposed to verify is completely broken.
3. A `#[should_panic]` test passes when it should only pass for a specific panic.
4. Tests that run in isolation pass, but running the full suite produces intermittent failures.

## Your Task

Read `buggy.rs`. Find all four bugs. Fix them so the tests accurately reflect the
behavior described in each test's name.

Run the buggy file: `rustc --test buggy.rs && ./buggy`

After fixing all four bugs, every test should pass — and each test should fail if the
production code were broken in the way the test name implies.

## What You Should NOT Do

- Do not look at `solution.md` until you've identified all four bugs yourself
- Do not modify the production rate limiter code (everything above the `#[cfg(test)]` line)
- The fix for each bug is small — one or two lines at most
