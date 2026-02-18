# Code Review Exercise -- Notification Event System

## Overview

**Scenario:** A junior developer has submitted a PR implementing a notification event system for an internal tool. The system allows different parts of the application to react to user actions (e.g., send emails, update dashboards, trigger workflows). The PR author says it "works in their tests."

**Your Task:** Review the proposed changes in `proposed.go` and provide feedback on:
- Correctness and bugs
- Concurrency safety
- Code quality and maintainability
- Design issues
- Performance concerns
- Alternative approaches

---

## Context

**Codebase:** Internal notification platform used by multiple backend services.

**Feature:** Generic event system that other packages import to publish and subscribe to typed events.

**Files Changed:** `proposed.go` (new file)

---

## The Pull Request

### PR Description
```
Add event notification system

This PR adds a simple event system for decoupling service components.
Components can subscribe to events and get notified when things happen.
I've tested it manually and it works great.
```

### Changes
See `proposed.go`

---

## Your Review

Use `my-review.md` to document your findings:

1. **Critical Issues** -- Bugs, data races, deadlocks, panics
2. **Major Concerns** -- Design problems, coupling issues, missing features
3. **Minor Suggestions** -- Naming, style, documentation
4. **Positive Feedback** -- What was done well

---

## Hints (Progressive)

<details>
<summary>Hint 1: Look at the relationship between Observer and Subject</summary>

Check whether observers hold references back to the subject. What implications does this have for memory management and coupling?

</details>

<details>
<summary>Hint 2: Think about what happens during notification</summary>

What happens if an observer is slow? What happens if the observer list changes during notification? Are there concurrency protections?

</details>

<details>
<summary>Hint 3: Consider lifecycle management</summary>

Is there a way to unsubscribe? What happens when an observer is no longer needed? Can the observer list grow without bound?

</details>

<details>
<summary>Hint 4: Run the race detector mentally</summary>

Trace through concurrent access patterns. Multiple goroutines publishing simultaneously while others subscribe -- are all shared data structures protected?

</details>

---

## Learning Objectives

This exercise practices:
- [ ] Reading unfamiliar code and forming a mental model quickly
- [ ] Identifying concurrency bugs without running code
- [ ] Spotting design coupling (circular dependencies)
- [ ] Recognizing missing features (unsubscribe, error handling)
- [ ] Giving constructive, actionable feedback

---

## Notes

**Difficulty:** Medium

**Estimated Time:** 20-30 min

**Related Concepts:** [[observer]], [[interfaces-and-traits]], [[concurrency]]
