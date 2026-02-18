# Debugging: Generics

There are **3 bugs** in `buggy.go`. All bugs produce compile errors or incorrect behavior directly tied to the generics concepts from the lesson.

## Symptoms

Running `go test ./...` in this directory produces compilation errors and test failures:

1. **Build fails on `BuildIndex`** — something about the type constraint on `K`
2. **Build fails on `WrapAll`** — a custom type is rejected even though it "should work"
3. **`SumScores` test passes but returns wrong result** — a method call compiles but produces the wrong value

The test file tells you what each function is supposed to do. Your job is to diagnose the root cause of each bug and fix it in `buggy.go`.

## What NOT to do

- Do not change `buggy_test.go`
- Do not change the function signatures (names, parameters, return types)
- Fixes should be minimal — don't rewrite the whole function

## Concepts Tested

- Why map keys require `comparable` (Bug 1)
- When `~` is required to accept named types with a matching underlying type (Bug 2)
- Why a method defined on a constraint's interface is not accessible without that constraint (Bug 3)
