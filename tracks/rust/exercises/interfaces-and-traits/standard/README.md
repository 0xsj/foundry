# Standard Exercise: Pluggable Notification System

## Scenario

You're building the alerting layer for an incident management platform. The platform
needs to send notifications through multiple channels — Email, SMS, and Webhook — and
must support routing by priority. High-priority alerts go to all channels simultaneously.
Normal alerts go only to the primary channel. The notification channels themselves are
pluggable: new channel types can be added without touching the routing logic.

## Brief

Implement a `Notifier` trait and three concrete types (`EmailNotifier`, `SmsNotifier`,
`WebhookNotifier`). Build a `NotificationRouter` that holds a heterogeneous list of
notifiers and dispatches based on priority. Demonstrate both static dispatch (`impl Trait`)
and dynamic dispatch (`Box<dyn Notifier>`) in the same codebase.

## Acceptance Criteria

1. **`Priority` enum** with variants:
   - `Critical` — triggers all channels; also calls `send_urgent` instead of `send`
   - `High` — triggers all channels
   - `Normal` — triggers only the primary (first) channel
   - `Low` — no-ops (suppressed; returns Ok without sending)
   - Derives: `Debug`, `Clone`, `Copy`, `PartialEq`, `PartialOrd`

2. **`Notifier` trait**:
   - Required: `fn channel_name(&self) -> &str`
   - Required: `fn send(&self, subject: &str, body: &str) -> Result<(), NotifyError>`
   - Default: `fn send_urgent(&self, subject: &str, body: &str) -> Result<(), NotifyError>`
     — default implementation prefixes `subject` with `"[URGENT] "` and calls `send`
   - Default: `fn describe(&self) -> String`
     — returns `"Notifier(<channel_name>)"`

3. **`NotifyError`** — a simple struct with a `message: String` field:
   - Implements `Debug`, `Display` (shows the message), and `From<String>`

4. **`EmailNotifier`** — fields: `to: String`, `from: String`:
   - `channel_name()` returns `"email"`
   - `send()` simulates sending: `println!("[Email] {from} -> {to} | {subject}: {body}")`
   - Uses the default `send_urgent` and `describe`

5. **`SmsNotifier`** — fields: `number: String`:
   - `channel_name()` returns `"sms"`
   - `send()` simulates sending: `println!("[SMS] {number} | {subject}: {body}")`
   - Overrides `send_urgent`: prepends `"ALERT: "` to the subject (SMS has no headers)

6. **`WebhookNotifier`** — fields: `url: String`, `secret: String`:
   - `channel_name()` returns `"webhook"`
   - `send()` simulates sending: `println!("[Webhook] POST {url} | {subject}: {body}")`
   - `secret` must never appear in any printed output (redact in Debug impl)

7. **`fn send_to_one(notifier: &impl Notifier, subject: &str, body: &str)`**:
   - Static dispatch. Calls `notifier.send()`. Prints an error to stderr if it fails.
   - Returns `()` (swallows the error after logging it)

8. **`NotificationRouter`** — fields: `channels: Vec<Box<dyn Notifier>>`:
   - `fn new() -> Self` — empty router
   - `fn add_channel(&mut self, channel: Box<dyn Notifier>)`
   - `fn dispatch(&self, priority: Priority, subject: &str, body: &str) -> Vec<Result<(), NotifyError>>`:
     - `Critical`: calls `send_urgent` on ALL channels
     - `High`: calls `send` on ALL channels
     - `Normal`: calls `send` on the FIRST channel only (index 0); empty vec if no channels
     - `Low`: returns empty vec (suppressed)
   - `fn channel_count(&self) -> usize`

9. **`fn summarize_results(results: &[Result<(), NotifyError>]) -> (usize, usize)`**:
   - Returns `(success_count, failure_count)`

## Constraints

- No external crates — stdlib only
- All tests in the starter file must pass against your implementation
- `WebhookNotifier`'s `secret` field must never appear in `{:?}` output

## Hints

<details>
<summary>Hint 1: Making a heterogeneous Vec of notifiers</summary>

`Vec<Box<dyn Notifier>>` stores trait objects. When adding a concrete type:

```rust
router.add_channel(Box::new(EmailNotifier { to: ..., from: ... }));
router.add_channel(Box::new(SmsNotifier { number: ... }));
```

Each `Box<dyn Notifier>` is a fat pointer: data pointer + vtable pointer.
All have the same fixed size, so they fit in a `Vec`.

</details>

<details>
<summary>Hint 2: Dispatching to all vs first channel</summary>

```rust
fn dispatch(&self, priority: Priority, ...) -> Vec<Result<(), NotifyError>> {
    match priority {
        Priority::Low => vec![],
        Priority::Normal => {
            if let Some(ch) = self.channels.first() {
                vec![ch.send(subject, body)]
            } else {
                vec![]
            }
        }
        Priority::High => self.channels.iter().map(|ch| ch.send(subject, body)).collect(),
        Priority::Critical => self.channels.iter().map(|ch| ch.send_urgent(subject, body)).collect(),
    }
}
```

</details>

<details>
<summary>Hint 3: Redacting a field in Debug</summary>

Implement `Debug` manually for `WebhookNotifier`:

```rust
impl std::fmt::Debug for WebhookNotifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebhookNotifier")
            .field("url", &self.url)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}
```

</details>

<details>
<summary>Hint 4: Default method calling another trait method</summary>

Default methods can call required methods on `self`:

```rust
trait Notifier {
    fn channel_name(&self) -> &str;
    fn send(&self, subject: &str, body: &str) -> Result<(), NotifyError>;

    fn send_urgent(&self, subject: &str, body: &str) -> Result<(), NotifyError> {
        // Calling send() is safe — it's a required method, so it always exists.
        self.send(&format!("[URGENT] {}", subject), body)
    }
}
```

</details>
