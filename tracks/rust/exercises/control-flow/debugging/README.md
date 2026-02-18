# Debugging Exercise: Job Scheduler

## Context

A teammate wrote a simple job scheduler that reads jobs from a queue, routes
them to handlers based on job type, and tracks which jobs were processed vs.
skipped. The code compiles (with one `#[allow(...)]` suppressing a warning),
but produces incorrect results at runtime.

Your job: find the bugs, explain why they happen, and fix them.

## How to Run

```bash
# Run the tests (some will fail)
rustc --test buggy.rs && ./buggy

# After fixing all bugs, all tests should pass
```

## Symptoms

### Symptom 1: Urgent jobs not routed correctly

`route_job` should return `"urgent-queue"` for any `Job` with priority 8 or
higher. For priority 9, it returns `"standard-queue"` instead.

**Failing test:** `test_route_urgent_high_priority`

### Symptom 2: retry_count never changes

`schedule_retry` is supposed to increment `job.retry_count` on each call.
After calling it 3 times, `job.retry_count` is still 0.

**Failing tests:** `test_schedule_retry_increments`, `test_schedule_retry_max`

### Symptom 3: The scheduler loops forever instead of draining the queue

`drain_queue` takes a `Vec<Job>` and processes each job, returning processed
and skipped counts. When called with a non-empty queue, it never returns.

**Failing test:** `test_drain_queue`

### Symptom 4: Cancelled jobs counted as processed

`count_outcomes` should count `JobOutcome::Processed` separately from
`JobOutcome::Cancelled`. Instead, cancelled jobs are counted in the `processed`
total along with actually-processed jobs.

**Failing test:** `test_count_outcomes`

## Rules

- Fix the bugs in `buggy.rs` directly
- Do not change the test assertions
- After fixing, all tests should pass
- The `#[allow(unreachable_patterns)]` on one match is a hint — remove it when you fix the bug
- Check `solution.md` only after you've attempted all four fixes
