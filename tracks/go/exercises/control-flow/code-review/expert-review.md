# Code Review: Task Processor

**Reviewer:** Expert Go Developer
**Date:** 2026-02-14
**PR:** Task Processor Implementation

---

## Critical Issues

### Issue 1: Nil Pointer Panic Risk

**Location:** `proposed.go:18`

**Problem:** No nil check before dereferencing `task`. If `ProcessTask` is called with `nil`, it will panic.

**Impact:** Service crash on invalid input.

**Suggested Fix:**
```go
func ProcessTask(task *Task) error {
    if task == nil {
        return fmt.Errorf("task cannot be nil")
    }
    // ... rest of function
}
```

---

### Issue 2: Off-by-One Error in Retry Loop

**Location:** `proposed.go:89-90`

**Problem:** Loop condition `attempt <= maxAttempts` combined with post-operation increment causes one extra attempt.

**Impact:** Attempts one more time than `maxAttempts` specifies, violating function contract.

**Example:**
```go
// If maxAttempts = 3, this loop runs 4 times:
// attempt=1 (try), attempt=2 (try), attempt=3 (try), attempt=4 (try)
for attempt <= maxAttempts {
    op()
    attempt = attempt + 1
}
```

**Suggested Fix:**
```go
for attempt := 0; attempt < maxAttempts; attempt++ {
    err := op()
    if err == nil {
        return nil
    }
    fmt.Printf("Attempt %d/%d failed, retrying...\n", attempt+1, maxAttempts)
    time.Sleep(time.Second)
}
return fmt.Errorf("failed after %d attempts", maxAttempts)
```

---

## Major Concerns

### Issue 1: Deeply Nested Conditionals

**Location:** `proposed.go:19-77` (entire ProcessTask function)

**Problem:** 6+ levels of nesting make the code extremely hard to read and maintain. The happy path is buried deep inside nested blocks.

**Why it matters:**
- Hard to trace logic
- Error-prone to modify
- Violates Go idiom of "guard clauses and early returns"

**Suggested Refactor:**
```go
func ProcessTask(task *Task) error {
    // Guard clauses first
    if task == nil {
        return fmt.Errorf("task cannot be nil")
    }
    if task.Status == "completed" {
        fmt.Println("Task already completed")
        return nil
    }
    if task.Status == "failed" {
        return fmt.Errorf("task %s permanently failed", task.ID)
    }
    if task.Status != "pending" {
        return fmt.Errorf("invalid status: %s", task.Status)
    }
    if task.Type == "" {
        return fmt.Errorf("task type is empty")
    }
    if task.Payload == nil {
        return fmt.Errorf("payload is nil")
    }

    // Happy path at top level
    var err error
    switch task.Type {
    case "email":
        fmt.Printf("Processing email task %s\n", task.ID)
        err = sendEmail(task.Payload)
    case "sms":
        fmt.Printf("Processing SMS task %s\n", task.ID)
        err = sendSMS(task.Payload)
    default:
        return fmt.Errorf("unknown task type: %s", task.Type)
    }

    if err != nil {
        return task.handleRetry(err)
    }

    task.Status = "completed"
    return nil
}

func (t *Task) handleRetry(err error) error {
    if t.Retries >= t.MaxRetries {
        t.Status = "failed"
        return fmt.Errorf("max retries exceeded: %w", err)
    }

    t.Retries++
    fmt.Printf("Retry %d/%d\n", t.Retries, t.MaxRetries)
    time.Sleep(time.Second * 2)
    return ProcessTask(t)
}
```

**Benefits:**
- Guard clauses handle edge cases first
- Happy path is linear and clear
- Switch statement for task types
- Retry logic extracted to separate method
- Much easier to read and modify

---

### Issue 2: Code Duplication

**Location:** `proposed.go:26-66` (email and SMS processing blocks)

**Problem:** Email and SMS processing logic is almost identical—only the function called differs. This violates DRY (Don't Repeat Yourself).

**Why it matters:**
- Bug fixes must be applied twice
- Increases maintenance burden
- Easy to introduce inconsistencies

**Suggested Fix:**
See the refactor above using a switch statement and shared retry logic.

---

### Issue 3: Recursive Retry Can Cause Stack Overflow

**Location:** `proposed.go:38, 59` (recursive ProcessTask calls)

**Problem:** Using recursion for retries can exhaust stack space if MaxRetries is large.

**Why it matters:** While unlikely with small retry counts, it's not a scalable pattern.

**Suggested Fix:**
Use iteration instead of recursion:
```go
func ProcessTaskWithRetries(task *Task) error {
    for {
        err := ProcessTask(task)
        if err == nil {
            return nil
        }

        if task.Retries >= task.MaxRetries {
            task.Status = "failed"
            return fmt.Errorf("max retries exceeded: %w", err)
        }

        task.Retries++
        fmt.Printf("Retry %d/%d\n", task.Retries, task.MaxRetries)
        time.Sleep(time.Second * 2)
    }
}
```

---

## Minor Suggestions

### Issue 1: Use Switch Instead of If-Else Chain

**Location:** `proposed.go:105-115` (GetTaskPriority function)

**Suggestion:** Switch statements are more idiomatic for multi-way branches:

```go
func GetTaskPriority(taskType string) int {
    switch taskType {
    case "critical":
        return 1
    case "high":
        return 2
    case "medium":
        return 3
    case "low":
        return 4
    default:
        return 5
    }
}
```

**Why:** Clearer intent, less visual noise, easier to extend.

---

### Issue 2: Increment Operator

**Location:** `proposed.go:38, 59, 94`

**Suggestion:** Use `++` instead of `= X + 1`:

```go
// Current
task.Retries = task.Retries + 1

// Preferred
task.Retries++
```

**Why:** More idiomatic, less verbose.

---

### Issue 3: String Literals for Status

**Location:** Throughout

**Suggestion:** Define constants for task statuses to prevent typos:

```go
const (
    StatusPending   = "pending"
    StatusCompleted = "completed"
    StatusFailed    = "failed"
)
```

---

## Positive Feedback

- ✅ Good error wrapping with `%w` format verb
- ✅ Retry backoff is present (though could be exponential)
- ✅ Status field is updated appropriately
- ✅ Functions are small and focused (except ProcessTask)

---

## Summary

**Recommendation:** ⚠️ **Request Changes**

**Overall:** The implementation works but has several critical and major issues that should be addressed before merging. The primary concern is the deeply nested control flow in `ProcessTask`, which makes the code hard to read and maintain. Refactoring to use guard clauses, a switch statement, and extracting retry logic will significantly improve code quality.

**Before merging:**
1. ✅ Add nil check for task parameter
2. ✅ Fix off-by-one error in RetryOperation
3. ✅ Refactor ProcessTask to use guard clauses and switch
4. ✅ Extract retry logic to avoid recursion and duplication
5. ⚠️ Consider defining status constants
6. ⚠️ Consider using switch in GetTaskPriority

---

## Learning Points

### Control Flow Anti-Patterns Demonstrated

1. **Deep nesting vs guard clauses**
   - ❌ Nested if-else that buries the happy path
   - ✅ Guard clauses that exit early, keeping happy path at top level

2. **If-else chains vs switch**
   - ❌ Long if-else chains for value comparison
   - ✅ Switch statements for multi-way branches

3. **Loop counter bugs**
   - ❌ `<=` with post-increment causing off-by-one errors
   - ✅ `<` with clear iteration bounds

4. **Code duplication**
   - ❌ Copy-pasted logic for similar operations
   - ✅ Extracted common patterns with parameterization

5. **Recursion vs iteration**
   - ❌ Recursion for retries (stack risk)
   - ✅ Iteration for bounded loops

These are common real-world issues you'll encounter in code reviews.
