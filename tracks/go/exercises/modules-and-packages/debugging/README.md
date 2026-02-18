# Debugging: Package Structure Errors

## Scenario

A developer started reorganizing a shared notification service into packages. The code compiles on their machine (they said), but you're seeing errors as soon as you try to build. Your job is to figure out what's wrong and fix it — without running the code.

## Symptoms

When you run `go build ./...` from the `buggy/` directory, you see:

```
package github.com/foundry/notify/pkg/b: import cycle not allowed
    github.com/foundry/notify/pkg/a imports github.com/foundry/notify/pkg/b
    github.com/foundry/notify/pkg/b imports github.com/foundry/notify/pkg/a

./main.go:18:13: cfg.retryCount undefined (cannot refer to unexported field or method retryCount)
./main.go:26:30: cannot use b.NewSender("smtp") (type *b.sender) as type b.Sender
```

## Your Task

1. Read `buggy/main.go`, `buggy/pkg/a/a.go`, and `buggy/pkg/b/b.go` carefully.
2. Identify the root cause of each error — do not run the code yet.
3. Write down your diagnosis (what's wrong and why) before looking at `solution.md`.
4. Fix the bugs.

## Concepts Being Tested

- Import cycle detection and resolution
- Exported vs unexported identifiers (field visibility)
- Exported vs unexported types returned from functions
- Understanding the Go compiler's visibility rules

## Notes

- There are exactly **three bugs**, one per compiler error shown above.
- Each bug maps directly to a concept from the modules-and-packages lesson.
- The bugs are in the code structure itself, not in logic — no test server required.
