# Exercise: Notification Dispatcher

## Scenario

Your team runs a platform that sends notifications across four channels: email, SMS, webhook, and Slack. Each channel has different configuration, delivery behavior, and retry limits. A dispatcher sits in front, receives a `Notification`, picks the right channel based on the notification type, and drives the delivery loop — retrying on failure, logging each attempt, and returning a final delivery report.

The core behaviors (retry tracking, attempt logging) are shared across all channels. The delivery mechanism is channel-specific.

## Brief

Implement a notification dispatcher with the following types and behaviors:

1. **`NotificationType`** — an enum (`iota`) for: `TypeEmail`, `TypeSMS`, `TypeWebhook`, `TypeSlack`
2. **`DeliveryStatus`** — an enum for: `StatusPending`, `StatusDelivered`, `StatusFailed`; both with `String()` methods
3. **`RetryPolicy`** struct — holds `MaxAttempts int` and `BackoffMs int`; methods: `ShouldRetry(attempts int) bool`
4. **`DeliveryLog`** struct — holds `[]string` entries; method: `Append(msg string)`, `Entries() []string`
5. **Channel structs** — `EmailChannel`, `SMSChannel`, `WebhookChannel`, `SlackChannel`:
   - Each embeds `RetryPolicy` and `DeliveryLog` by value
   - Each has a `Deliver(n Notification) error` method (stub: succeed on even attempts, fail on odd)
   - Each has a `Name() string` method
6. **`Notification`** struct — fields: `ID string`, `Type NotificationType`, `Recipient string`, `Subject string`, `Body string`; constructor `NewNotification` that validates required fields
7. **`Dispatcher`** struct — holds a `map[NotificationType]Deliverer` of registered channels; method `Dispatch(n Notification) DeliveryReport`
8. **`Deliverer`** interface — implemented by all four channel types: `Deliver(Notification) error`, `Name() string`
9. **`DeliveryReport`** struct — fields: `NotificationID string`, `Channel string`, `Status DeliveryStatus`, `Attempts int`, `Log []string`

### Dispatch Logic

`Dispatch` should:
1. Look up the channel for `n.Type` — return a report with `StatusFailed` and a log entry if no channel is registered
2. Drive the retry loop: call `Deliver`, record the attempt in the channel's `DeliveryLog`, check `ShouldRetry` on failure
3. Return a `DeliveryReport` with the final status, attempt count, and all log entries

## Acceptance Criteria

- [ ] `NotificationType` and `DeliveryStatus` are typed `int` constants using `iota`, both have `String()` methods
- [ ] `RetryPolicy.ShouldRetry(attempts int) bool` returns `attempts < MaxAttempts`
- [ ] `DeliveryLog.Append` adds a timestamped entry; `Entries()` returns all entries
- [ ] All four channel structs embed `RetryPolicy` and `DeliveryLog` by value
- [ ] All four channel structs have pointer receivers on `Deliver` and `Name`
- [ ] `NewNotification` returns `(*Notification, error)`, rejects empty `ID` or `Recipient`
- [ ] `Dispatcher` is initialized via `NewDispatcher()` with an initialized channel map
- [ ] `Dispatcher.Register(t NotificationType, d Deliverer)` adds a channel
- [ ] `Dispatcher.Dispatch(n Notification) DeliveryReport` runs the retry loop
- [ ] Dispatch returns a failed report (not a panic) when no channel is registered for the notification type
- [ ] All tests in `main_test.go` pass

## Constraints

- Standard library only — no third-party packages
- All mutation must use pointer receivers — no value receiver that accidentally modifies a copy
- Channel map must be initialized with `make` — zero value panics on write
- Do not use global state — all state lives in structs

## Concepts Exercised

- Struct definitions with embedded types
- Pointer vs value receivers (and when each applies)
- Constructor pattern with validation
- `iota` enums with `String()` methods
- Struct embedding: promoted fields and methods
- Interface definition and satisfaction
- Type switch for dispatching on channel type

## Hints

<details>
<summary>Hint 1: Embedding vs field</summary>

To embed `RetryPolicy`, write:
```go
type EmailChannel struct {
    RetryPolicy      // embedded — no field name
    DeliveryLog
    // ... other fields
}
```
Then you can call `e.ShouldRetry(attempts)` directly on an `EmailChannel`.
</details>

<details>
<summary>Hint 2: Interface with pointer receivers</summary>

If `Deliver` has a pointer receiver `(c *EmailChannel)`, then `*EmailChannel` implements `Deliverer`, but `EmailChannel` does not. Store `*EmailChannel` in the dispatcher map:
```go
dispatcher.Register(TypeEmail, &EmailChannel{...})
```
</details>

<details>
<summary>Hint 3: Dispatch loop structure</summary>

```go
for attempts := 0; ; attempts++ {
    err := channel.Deliver(n)
    if err == nil {
        // success
        break
    }
    if !channel.ShouldRetry(attempts + 1) {
        // exhausted retries
        break
    }
}
```
</details>

<details>
<summary>Hint 4: DeliveryLog and embedded access</summary>

The `DeliveryLog` appended to inside `Deliver` should be the one embedded in the channel struct — not a copy. Make sure `Deliver` has a pointer receiver so it mutates the real embedded `DeliveryLog`.
</details>
