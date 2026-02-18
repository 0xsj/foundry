# Debugging: Interfaces and Traits

## Scenario

A teammate submitted a cache manager that wraps a storage backend and provides health checking. The code compiles but the tests are failing in unexpected ways: nil panics at runtime, mutations that silently disappear, and a panic from a type assertion. All three bugs are interface-related.

## Symptoms

**Test `TestCacheManagerHealthCheck` panics:**
```
panic: runtime error: invalid memory address or nil pointer dereference
```
This panic happens even though the health checker returned `nil` error, which should mean it's healthy.

**Test `TestCacheManagerSetCount` fails:**
```
expected SetCount=3, got SetCount=0
```
The counter never increments — every write is invisible.

**Test `TestCacheManagerInspect` panics:**
```
panic: interface conversion: cache.Storer is *cache.MemoryBackend, not *cache.FileBackend
```
The assertion should be safe — the code is supposed to check the type before accessing it.

## Your Task

1. Read the failing tests in `buggy_test.go` to understand expected behavior
2. Find the three bugs in `buggy.go`
3. Fix all three bugs so all tests pass
4. Write down what each bug was and why it happened (use the space in `solution.md` if you want, but try to diagnose independently first)

## Concepts Tested

- The nil interface trap: a typed nil concrete value is not a nil interface
- Value receivers do not mutate state — only pointer receivers do
- Type assertions without comma-ok panic on failure

## Constraints

- Do not change the test file
- Do not change the interfaces or struct field names
- Each bug should be a 1-5 line fix
