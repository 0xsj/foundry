# Standard Exercise: Webhook Payload Handler

## Scenario

Your platform receives webhook events from three external providers: GitHub (repository
events), Stripe (payment events), and Slack (message events). Each provider sends JSON
with a completely different shape. Your job is to build a normalizer that deserializes
each provider's raw payload into a typed enum variant, then converts it to a unified
internal `InternalEvent` format for downstream processing.

## Brief

Implement a multi-provider webhook handler. You'll model each provider's payload as a
typed struct or enum variant, deserialize raw JSON strings into those types, validate
required fields, and convert to a unified internal representation. No external crates —
use the manual JSON parsing helpers provided in the starter.

## Acceptance Criteria

1. **`ProviderPayload` enum** with variants:
   - `GitHub(GitHubEvent)` — carries a parsed GitHub event
   - `Stripe(StripeEvent)` — carries a parsed Stripe event
   - `Slack(SlackEvent)` — carries a parsed Slack event

2. **`GitHubEvent` struct** with fields:
   - `action: String` — the event action (e.g., "opened", "closed")
   - `repository: String` — the repository full name (e.g., "acme/api")
   - `sender: String` — the GitHub username
   - `number: Option<u64>` — PR/issue number, if applicable

3. **`StripeEvent` struct** with fields:
   - `id: String` — event ID (must start with "evt_")
   - `event_type: String` — the event type (e.g., "payment_intent.succeeded")
   - `amount_cents: Option<u64>` — amount if applicable
   - `currency: Option<String>` — currency code if applicable
   - `customer_id: Option<String>` — Stripe customer ID

4. **`SlackEvent` struct** with fields:
   - `event_type: String` — the event type (e.g., "message")
   - `channel: String` — channel ID
   - `user: String` — user ID
   - `text: Option<String>` — message text (absent for non-message events)

5. **`InternalEvent` struct** — the normalized representation:
   - `source: String` — "github", "stripe", or "slack"
   - `event_type: String` — normalized event type (see conversion rules)
   - `actor: String` — who triggered the event (sender, customer_id, or user)
   - `summary: String` — human-readable description
   - `metadata: Vec<(String, String)>` — additional key-value pairs

6. **`parse_provider_payload(provider: &str, json: &str) -> Result<ProviderPayload, String>`**
   - `provider`: "github", "stripe", or "slack"
   - Parses the JSON using the provided helpers
   - Returns `Err(String)` for unknown provider, missing required fields, or invalid field values
   - GitHub: `repository` and `sender` are required; `action` defaults to `"unknown"` if absent
   - Stripe: `id`, `event_type` required; `id` must start with "evt_"
   - Slack: `event_type`, `channel`, `user` required

7. **`impl ProviderPayload { fn into_internal(self) -> InternalEvent }`**
   - Converts a parsed payload into the internal normalized format
   - Conversion rules:
     - **GitHub**: source="github", event_type=action, actor=sender,
       summary="GitHub {action} on {repository}" (e.g., "GitHub opened on acme/api"),
       metadata=[("repo", repository), ("number", number.to_string() or "")]
     - **Stripe**: source="stripe", event_type=event_type, actor=customer_id or "anonymous",
       summary="Stripe {event_type}" (e.g., "Stripe payment_intent.succeeded"),
       metadata=[("stripe_event_id", id), ("amount", amount_cents as string or "")]
     - **Slack**: source="slack", event_type=event_type, actor=user,
       summary="Slack {event_type} in {channel}",
       metadata=[("channel", channel), ("text", text or "")]

8. **`fn process_webhook(provider: &str, json: &str) -> Result<InternalEvent, String>`**
   - Top-level function: calls `parse_provider_payload` then `into_internal`
   - Returns the `InternalEvent` or propagates the parsing error

## Constraints

- No external crates — use only the JSON helpers provided in the starter file
- `parse_provider_payload` must return ALL field-level errors as a descriptive `String`
- `into_internal` must be a method on `ProviderPayload` (not a free function)
- All tests in the starter must pass

## Hints

<details>
<summary>Hint 1: Structuring parse_provider_payload</summary>

Match on the provider string first, then call a dedicated parse function for each:

```rust
fn parse_provider_payload(provider: &str, json: &str) -> Result<ProviderPayload, String> {
    match provider {
        "github" => parse_github(json).map(ProviderPayload::GitHub),
        "stripe" => parse_stripe(json).map(ProviderPayload::Stripe),
        "slack"  => parse_slack(json).map(ProviderPayload::Slack),
        other    => Err(format!("unknown provider: {other}")),
    }
}
```

Each `parse_*` function returns `Result<TypedEvent, String>`. `.map(ProviderPayload::GitHub)` wraps
the success value in the enum variant without unwrapping.

</details>

<details>
<summary>Hint 2: Optional fields with defaults</summary>

`Option<u64>` fields: return `None` when the field is absent (not an error).
`String` fields with defaults: return the default when absent (not an error).

```rust
let action = get_str(json, "action")
    .unwrap_or_else(|| "unknown".to_string());  // default, not an error

let number = get_u64(json, "number");  // Option<u64> — None is fine
```

</details>

<details>
<summary>Hint 3: into_internal on ProviderPayload</summary>

Match on `self` (consuming the enum) and construct `InternalEvent` for each variant:

```rust
fn into_internal(self) -> InternalEvent {
    match self {
        ProviderPayload::GitHub(event) => InternalEvent {
            source: String::from("github"),
            event_type: event.action.clone(),
            actor: event.sender.clone(),
            summary: format!("GitHub {} on {}", event.action, event.repository),
            metadata: vec![
                (String::from("repo"), event.repository),
                (String::from("number"), event.number
                    .map(|n| n.to_string())
                    .unwrap_or_default()),
            ],
        },
        // ...
    }
}
```

</details>

<details>
<summary>Hint 4: Stripe ID validation</summary>

Validate after parsing the field, inside your `parse_stripe` function:

```rust
let id = get_str(json, "id").ok_or("missing field: id")?;
if !id.starts_with("evt_") {
    return Err(format!("stripe event id must start with 'evt_', got '{id}'"));
}
```

</details>
