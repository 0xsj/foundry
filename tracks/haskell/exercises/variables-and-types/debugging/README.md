# Debugging: Student Grade Tracker

## Symptoms

A teammate wrote a grade tracking module for a school admin dashboard. QA reports:

1. **Crashes at runtime** — the app crashes when calculating the average for a student with no grades
2. **Wrong letter grades** — some students get incorrect letter grades (e.g., a score of 90 is showing as "B" instead of "A")
3. **Type mismatch in GPA** — the GPA calculation produces wildly wrong numbers (e.g., 0 instead of 3.5)

There are 3 bugs — one causing each symptom.

## Your Task

1. Read `Buggy.hs` and `BuggyTest.hs`
2. Run the tests: `runhaskell -i. BuggyTest.hs`
3. Find all 3 bugs without looking at `solution.md`
4. Fix each bug and get all tests passing
5. For each bug, note: what it was, why it happened, how to prevent it

## Difficulty

**Basic to Intermediate** — Each bug maps to a core concept from the lesson: partial functions, pattern matching order, and numeric type conversions.

## Hints

<details>
<summary>Hint 1: The crash</summary>

What happens when you call `head` on an empty list? Is there a safer alternative?
</details>

<details>
<summary>Hint 2: The wrong letter grades</summary>

When Haskell pattern-matches guards, it tries them top to bottom. What happens if the first guard is too broad?
</details>

<details>
<summary>Hint 3: The GPA calculation</summary>

What does integer division produce in Haskell? If both operands are `Int`, what type is the result of `div`?
</details>
