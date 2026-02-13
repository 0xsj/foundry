# Solution: Student Grade Tracker

## Bug 1: Crash on Empty Grades — Partial Function (`head` / division by zero)

**Location:** `average` (lines 12-15) and `topScore` (line 44)

**What:** `average` divides by `length grades` without checking if the list is empty. When `grades` is `[]`, `length` returns 0 and division by zero produces `NaN` or crashes. The function always returns `Just`, never `Nothing`. Similarly, `topScore` calls `head` on an empty list → crash.

**Fix:**
```haskell
average :: GradeRecord -> Maybe Double
average (_, []) = Nothing  -- handle empty case first
average (_, grades) =
  let total = sum grades
      count = length grades
  in Just (fromIntegral total / fromIntegral count)
```

**Why it happens:** In Go, you'd check `if len(grades) == 0` before dividing. In Haskell, the idiomatic approach is pattern matching on `[]` first. The original code skips this, and `head` is a partial function — it crashes on empty lists (GHC 9.14 even warns about this).

**Prevention:** Always pattern match on empty lists before using `head`, `tail`, `last`, or dividing by `length`. Better: avoid `head`/`tail` entirely and use pattern matching.

---

## Bug 2: Wrong Letter Grades — Guard Order Matters

**Location:** `letterGrade` (lines 19-25)

**What:** Guards are evaluated top-to-bottom. The first guard `score >= 60` matches *everything* 60 and above — including 90, 80, 70. So a score of 95 hits `>= 60` first and returns "D".

```haskell
-- Bug: 95 >= 60 is true, so this returns "D" for a 95
letterGrade score
  | score >= 60 = "D"  -- catches everything >= 60
  | score >= 70 = "C"  -- never reached for 70-100
  | score >= 80 = "B"  -- never reached
  | score >= 90 = "A"  -- never reached
```

**Fix:** Reverse the order — check the most specific (highest) condition first:
```haskell
letterGrade :: Score -> String
letterGrade score
  | score >= 90 = "A"
  | score >= 80 = "B"
  | score >= 70 = "C"
  | score >= 60 = "D"
  | otherwise   = "F"
```

**Why it happens:** This is the same logic error you'd make in Go with `if/else if` chains or in JS with `if/else`. Guard order matters just like conditional branch order. The difference: in Go, a linter might catch overlapping conditions. In Haskell, the compiler doesn't warn about guard order — it's a logic error, not a type error.

**Prevention:** When using range-based guards, always go from most specific to least specific (highest threshold first).

---

## Bug 3: Integer Division in GPA — Wrong Numeric Type

**Location:** `gpa` (line 34)

**What:** `` fromIntegral total `div` fromIntegral count `` uses `div`, which is integer division. Even though `fromIntegral` converts to a numeric type, `div` constrains both operands to `Integral` — so the result is truncated to an integer, then converted back to `Double`. `10 `div` 4` gives `2`, not `2.5`.

```haskell
-- Bug: div is integer division
fromIntegral total `div` fromIntegral count  -- 10 `div` 4 = 2, not 2.5
```

**Fix:** Use `/` (floating-point division):
```haskell
gpa scores =
  let points = map scoreToPoints scores
      total = sum points
      count = length scores
  in fromIntegral total / fromIntegral count
```

**Why it happens:** In Go, `10 / 4` is `2` (integer division) and you'd write `float64(10) / float64(4)` for `2.5`. Haskell has the same distinction: `div` is integer division, `/` is fractional division. The bug is using `div` when `/` was intended. `fromIntegral` converts the operands, but `div` forces the operation back to integer semantics.

**Prevention:** Know the difference: `div` (`Integral` class) truncates, `/` (`Fractional` class) gives fractional results. If your return type is `Double`, use `/`.

---

## Summary

| Bug | Concept | Root Cause |
|-----|---------|------------|
| Crash on empty list | Partial functions | `head` and division by zero on empty lists |
| Wrong letter grades | Guard evaluation order | Most general condition matched first |
| GPA is wrong | Numeric type system | `div` (integer) used instead of `/` (fractional) |

All three bugs map to variables-and-types fundamentals: understanding partial functions (head/tail), how pattern matching and guards work, and Haskell's numeric type hierarchy.
