# Standard Exercise: Retry Mechanism with Backoff

## Scenario

You're building a microservice that needs to call external APIs. Network issues are common, so you need a robust retry mechanism with exponential backoff. When an API call fails, you should:
1. Wait a bit
2. Retry
3. If it fails again, wait longer
4. Eventually give up after max retries

This is a real pattern used in production systems (AWS SDKs, gRPC, HTTP clients, etc.).

## Brief

Implement a `Retry` function that:
- Takes an operation (a function that might fail)
- Retries it up to `maxRetries` times
- Uses exponential backoff between attempts (1s, 2s, 4s, 8s...)
- Returns the result on success, or an error after all retries exhausted
- Logs each attempt for observability

## Acceptance Criteria

1. **Retry logic**
   - Attempt the operation up to `maxRetries` times
   - Stop immediately on success (early return)
   - Stop immediately if the error is non-retryable

2. **Backoff strategy**
   - First retry: wait 1 second
   - Second retry: wait 2 seconds
   - Third retry: wait 4 seconds
   - Pattern: `2^attempt` seconds
   - Optional: add jitter to prevent thundering herd

3. **Error handling**
   - Distinguish between retryable and non-retryable errors
   - Wrap the final error with context (number of attempts)
   - Handle the case where operation is nil (guard clause)

4. **Edge cases**
   - maxRetries = 0 (should attempt once, no retries)
   - maxRetries < 0 (invalid input)
   - Operation succeeds on first try (no backoff)

## Constraints

- Use a `for` loop for retry logic
- Use `time.Sleep` for backoff delays
- Use early returns for success and non-retryable errors
- Use a switch or if-else for backoff calculation

## Function Signature

```go
// Operation is a function that might fail
type Operation func() error

// RetryableError wraps errors that can be retried
type RetryableError struct {
    Err error
}

func (e *RetryableError) Error() string {
    return e.Err.Error()
}

// Retry attempts an operation multiple times with exponential backoff
func Retry(op Operation, maxRetries int) error {
    // Your implementation here
}
```

## Hints

<details>
<summary>Hint 1: Guard clauses</summary>

Start with validation:
```go
if op == nil {
    return fmt.Errorf("operation cannot be nil")
}
if maxRetries < 0 {
    return fmt.Errorf("maxRetries must be >= 0")
}
```
</details>

<details>
<summary>Hint 2: Loop structure</summary>

Use a for loop that attempts up to `maxRetries + 1` times (initial attempt + retries):
```go
for attempt := 0; attempt <= maxRetries; attempt++ {
    // Try operation
    // Handle result
    // Calculate backoff if needed
}
```
</details>

<details>
<summary>Hint 3: Exponential backoff</summary>

Calculate delay as `2^attempt` seconds:
```go
delay := time.Second * time.Duration(1<<attempt)  // 1<<attempt is 2^attempt
time.Sleep(delay)
```
</details>

<details>
<summary>Hint 4: Early returns</summary>

Return immediately on success or non-retryable errors:
```go
if err == nil {
    return nil  // success
}

var retryableErr *RetryableError
if !errors.As(err, &retryableErr) {
    return err  // non-retryable, don't retry
}
```
</details>

## Example Usage

```go
func main() {
    // Simulated flaky API call
    attempts := 0
    flakyAPI := func() error {
        attempts++
        if attempts < 3 {
            return &RetryableError{Err: fmt.Errorf("network timeout")}
        }
        return nil  // succeeds on 3rd attempt
    }

    err := Retry(flakyAPI, 5)
    if err != nil {
        fmt.Println("Failed:", err)
    } else {
        fmt.Println("Success after", attempts, "attempts")
    }
}
```

## Testing Considerations

Your tests should cover:
- Operation succeeds immediately (no retries needed)
- Operation fails then succeeds (retries work)
- Operation always fails (max retries exhausted)
- Non-retryable error (stops immediately)
- Invalid inputs (nil operation, negative maxRetries)
- Backoff timing (verify delays increase exponentially)
