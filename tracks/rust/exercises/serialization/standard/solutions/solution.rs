// Webhook Payload Handler — Reference Solution
//
// A multi-provider webhook normalizer demonstrating:
//   - Typed structs per provider (what serde derives for you)
//   - Required vs optional fields with clear error messages
//   - Enum-based dispatch via into_internal
//   - Two-layer validation: structural (parse) + semantic (validate)
//
// Run tests: rustc --test solution.rs && ./solution

// ─────────────────────────────────────────────────────────────────────────────
// JSON helpers (stand-ins for serde_json::from_str + Deserialize derive)
// ─────────────────────────────────────────────────────────────────────────────

fn get_str(json: &str, field: &str) -> Option<String> {
    let key = format!(r#""{field}":""#);
    let start = json.find(&key)? + key.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn get_u64(json: &str, field: &str) -> Option<u64> {
    let key = format!(r#""{field}":"#);
    let start = json.find(&key)? + key.len();
    let rest = &json[start..];
    if rest.starts_with('"') { return None; }
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

#[allow(dead_code)]
fn get_bool(json: &str, field: &str) -> Option<bool> {
    if json.contains(&format!(r#""{field}":true"#)) { Some(true) }
    else if json.contains(&format!(r#""{field}":false"#)) { Some(false) }
    else { None }
}

// ─────────────────────────────────────────────────────────────────────────────
// Provider types — one struct per provider, modeling only the fields we care about
// In real code: #[derive(Deserialize)] on each struct
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct GitHubEvent {
    action: String,         // #[serde(default = "default_unknown")]
    repository: String,     // required
    sender: String,         // required
    number: Option<u64>,    // #[serde(skip_serializing_if = "Option::is_none")]
}

#[derive(Debug)]
struct StripeEvent {
    id: String,                      // required, must start with "evt_"
    event_type: String,              // required (field name: "type" in JSON)
    amount_cents: Option<u64>,
    currency: Option<String>,
    customer_id: Option<String>,
}

#[derive(Debug)]
struct SlackEvent {
    event_type: String,   // required (field name: "type" in JSON)
    channel: String,      // required
    user: String,         // required
    text: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// The enum carrying any provider's parsed payload
// In serde: #[serde(tag = "provider", rename_all = "lowercase")] on this enum
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
enum ProviderPayload {
    GitHub(GitHubEvent),
    Stripe(StripeEvent),
    Slack(SlackEvent),
}

// ─────────────────────────────────────────────────────────────────────────────
// The unified internal representation — format-agnostic, clean for downstream
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
struct InternalEvent {
    source: String,
    event_type: String,
    actor: String,
    summary: String,
    metadata: Vec<(String, String)>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsing — one function per provider
//
// Key decisions:
//   - Required fields: return Err with the field name if absent
//   - Optional fields: return None (not an error)
//   - Default fields: use .unwrap_or_else to provide the default
//   - Semantic validation: check after all fields are parsed
// ─────────────────────────────────────────────────────────────────────────────

fn parse_github(json: &str) -> Result<GitHubEvent, String> {
    // Required fields
    let repository = get_str(json, "repository")
        .ok_or_else(|| String::from("missing required field: repository"))?;
    let sender = get_str(json, "sender")
        .ok_or_else(|| String::from("missing required field: sender"))?;

    // Optional with default — #[serde(default = "default_unknown")] equivalent
    let action = get_str(json, "action").unwrap_or_else(|| String::from("unknown"));

    // Purely optional
    let number = get_u64(json, "number");

    Ok(GitHubEvent { action, repository, sender, number })
}

fn parse_stripe(json: &str) -> Result<StripeEvent, String> {
    // Required fields
    let id = get_str(json, "id")
        .ok_or_else(|| String::from("missing required field: id"))?;

    // Semantic validation immediately after the required field is found
    if !id.starts_with("evt_") {
        return Err(format!("stripe event id must start with 'evt_', got '{id}'"));
    }

    // Stripe uses "type" as the field name — we rename it to event_type internally
    // In serde: #[serde(rename = "type")] on the event_type field
    let event_type = get_str(json, "type")
        .ok_or_else(|| String::from("missing required field: type"))?;

    // Optional fields
    let amount_cents = get_u64(json, "amount_cents");
    let currency = get_str(json, "currency");
    let customer_id = get_str(json, "customer_id");

    Ok(StripeEvent { id, event_type, amount_cents, currency, customer_id })
}

fn parse_slack(json: &str) -> Result<SlackEvent, String> {
    // Slack also uses "type" — renamed to event_type
    let event_type = get_str(json, "type")
        .ok_or_else(|| String::from("missing required field: type"))?;
    let channel = get_str(json, "channel")
        .ok_or_else(|| String::from("missing required field: channel"))?;
    let user = get_str(json, "user")
        .ok_or_else(|| String::from("missing required field: user"))?;

    let text = get_str(json, "text"); // optional

    Ok(SlackEvent { event_type, channel, user, text })
}

// ─────────────────────────────────────────────────────────────────────────────
// Router — dispatch to the right parser based on provider name
//
// This is the pattern serde uses internally when deserializing a tagged enum:
// read the tag field, match on its value, call the right variant's deserializer.
// ─────────────────────────────────────────────────────────────────────────────

fn parse_provider_payload(provider: &str, json: &str) -> Result<ProviderPayload, String> {
    match provider {
        "github" => parse_github(json).map(ProviderPayload::GitHub),
        "stripe" => parse_stripe(json).map(ProviderPayload::Stripe),
        "slack"  => parse_slack(json).map(ProviderPayload::Slack),
        other    => Err(format!("unknown provider: '{other}'")),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Normalization — convert any provider's typed payload to InternalEvent
//
// This is the boundary where provider-specific details stop mattering.
// Downstream code only sees InternalEvent.
// ─────────────────────────────────────────────────────────────────────────────

impl ProviderPayload {
    fn into_internal(self) -> InternalEvent {
        match self {
            ProviderPayload::GitHub(e) => InternalEvent {
                source: String::from("github"),
                event_type: e.action.clone(),
                actor: e.sender.clone(),
                summary: format!("GitHub {} on {}", e.action, e.repository),
                metadata: vec![
                    (String::from("repo"), e.repository),
                    (String::from("number"), e.number
                        .map(|n| n.to_string())
                        .unwrap_or_default()),
                ],
            },

            ProviderPayload::Stripe(e) => InternalEvent {
                source: String::from("stripe"),
                event_type: e.event_type.clone(),
                actor: e.customer_id.clone().unwrap_or_else(|| String::from("anonymous")),
                summary: format!("Stripe {}", e.event_type),
                metadata: vec![
                    (String::from("stripe_event_id"), e.id),
                    (String::from("amount"), e.amount_cents
                        .map(|a| a.to_string())
                        .unwrap_or_default()),
                ],
            },

            ProviderPayload::Slack(e) => InternalEvent {
                source: String::from("slack"),
                event_type: e.event_type.clone(),
                actor: e.user.clone(),
                summary: format!("Slack {} in {}", e.event_type, e.channel),
                metadata: vec![
                    (String::from("channel"), e.channel),
                    (String::from("text"), e.text.unwrap_or_default()),
                ],
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Top-level entry point
// ─────────────────────────────────────────────────────────────────────────────

fn process_webhook(provider: &str, json: &str) -> Result<InternalEvent, String> {
    let payload = parse_provider_payload(provider, json)?;
    Ok(payload.into_internal())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── GitHub ──

    #[test]
    fn test_github_pr_opened() {
        let json = r#"{"action":"opened","repository":"acme/api","sender":"alice","number":42}"#;
        let payload = parse_provider_payload("github", json).unwrap();
        let event = payload.into_internal();

        assert_eq!(event.source, "github");
        assert_eq!(event.event_type, "opened");
        assert_eq!(event.actor, "alice");
        assert_eq!(event.summary, "GitHub opened on acme/api");
    }

    #[test]
    fn test_github_default_action() {
        let json = r#"{"repository":"acme/api","sender":"bob"}"#;
        let payload = parse_provider_payload("github", json).unwrap();
        let event = payload.into_internal();
        assert_eq!(event.event_type, "unknown");
    }

    #[test]
    fn test_github_missing_repository() {
        let json = r#"{"action":"opened","sender":"carol"}"#;
        let result = parse_provider_payload("github", json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("repository"));
    }

    #[test]
    fn test_github_missing_sender() {
        let json = r#"{"action":"closed","repository":"acme/api"}"#;
        let result = parse_provider_payload("github", json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sender"));
    }

    #[test]
    fn test_github_number_in_metadata() {
        let json = r#"{"action":"opened","repository":"acme/api","sender":"dave","number":99}"#;
        let payload = parse_provider_payload("github", json).unwrap();
        let event = payload.into_internal();
        let number_entry = event.metadata.iter().find(|(k, _)| k == "number");
        assert!(number_entry.is_some());
        assert_eq!(number_entry.unwrap().1, "99");
    }

    #[test]
    fn test_github_no_number() {
        let json = r#"{"action":"push","repository":"acme/api","sender":"eve"}"#;
        let payload = parse_provider_payload("github", json).unwrap();
        let event = payload.into_internal();
        let number_entry = event.metadata.iter().find(|(k, _)| k == "number");
        assert!(number_entry.is_some());
        assert_eq!(number_entry.unwrap().1, "");
    }

    // ── Stripe ──

    #[test]
    fn test_stripe_payment_succeeded() {
        let json = r#"{"id":"evt_001","type":"payment_intent.succeeded","amount_cents":4999,"currency":"usd","customer_id":"cus_abc"}"#;
        let payload = parse_provider_payload("stripe", json).unwrap();
        let event = payload.into_internal();

        assert_eq!(event.source, "stripe");
        assert_eq!(event.event_type, "payment_intent.succeeded");
        assert_eq!(event.actor, "cus_abc");
        assert_eq!(event.summary, "Stripe payment_intent.succeeded");
    }

    #[test]
    fn test_stripe_invalid_id_prefix() {
        let json = r#"{"id":"pi_001","type":"charge.succeeded"}"#;
        let result = parse_provider_payload("stripe", json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("evt_"));
    }

    #[test]
    fn test_stripe_missing_id() {
        let json = r#"{"type":"payment_intent.failed"}"#;
        let result = parse_provider_payload("stripe", json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("id"));
    }

    #[test]
    fn test_stripe_no_customer_uses_anonymous() {
        let json = r#"{"id":"evt_002","type":"charge.refunded","amount_cents":1000}"#;
        let payload = parse_provider_payload("stripe", json).unwrap();
        let event = payload.into_internal();
        assert_eq!(event.actor, "anonymous");
    }

    #[test]
    fn test_stripe_amount_in_metadata() {
        let json = r#"{"id":"evt_003","type":"payment_intent.succeeded","amount_cents":7500,"currency":"eur","customer_id":"cus_xyz"}"#;
        let payload = parse_provider_payload("stripe", json).unwrap();
        let event = payload.into_internal();
        let amt = event.metadata.iter().find(|(k, _)| k == "amount");
        assert!(amt.is_some());
        assert_eq!(amt.unwrap().1, "7500");
    }

    // ── Slack ──

    #[test]
    fn test_slack_message_event() {
        let json = r#"{"type":"message","channel":"C0123456","user":"U9876543","text":"hello world"}"#;
        let payload = parse_provider_payload("slack", json).unwrap();
        let event = payload.into_internal();

        assert_eq!(event.source, "slack");
        assert_eq!(event.event_type, "message");
        assert_eq!(event.actor, "U9876543");
        assert_eq!(event.summary, "Slack message in C0123456");
    }

    #[test]
    fn test_slack_missing_channel() {
        let json = r#"{"type":"message","user":"U1234"}"#;
        let result = parse_provider_payload("slack", json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("channel"));
    }

    #[test]
    fn test_slack_text_in_metadata() {
        let json = r#"{"type":"message","channel":"C0001","user":"U0001","text":"deploy complete"}"#;
        let payload = parse_provider_payload("slack", json).unwrap();
        let event = payload.into_internal();
        let text = event.metadata.iter().find(|(k, _)| k == "text");
        assert!(text.is_some());
        assert_eq!(text.unwrap().1, "deploy complete");
    }

    #[test]
    fn test_slack_no_text() {
        let json = r#"{"type":"reaction_added","channel":"C0002","user":"U0002"}"#;
        let payload = parse_provider_payload("slack", json).unwrap();
        let event = payload.into_internal();
        let text = event.metadata.iter().find(|(k, _)| k == "text");
        assert!(text.is_some());
        assert_eq!(text.unwrap().1, "");
    }

    // ── Unknown provider ──

    #[test]
    fn test_unknown_provider() {
        let json = r#"{"event":"ping"}"#;
        let result = parse_provider_payload("pagerduty", json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown"));
    }

    // ── process_webhook ──

    #[test]
    fn test_process_webhook_end_to_end() {
        let json = r#"{"id":"evt_999","type":"customer.created","customer_id":"cus_new"}"#;
        let event = process_webhook("stripe", json).unwrap();
        assert_eq!(event.source, "stripe");
        assert_eq!(event.event_type, "customer.created");
    }

    #[test]
    fn test_process_webhook_propagates_errors() {
        let result = process_webhook("stripe", r#"{"id":"bad"}"#);
        assert!(result.is_err());
    }
}

fn main() {
    // Demonstrate the full webhook processing pipeline
    let webhooks = vec![
        ("github", r#"{"action":"opened","repository":"acme/api","sender":"alice","number":101}"#),
        ("stripe", r#"{"id":"evt_pay_001","type":"payment_intent.succeeded","amount_cents":9900,"currency":"usd","customer_id":"cus_alice"}"#),
        ("slack",  r#"{"type":"message","channel":"C0DEPLOY","user":"U0ALICE","text":"deployment succeeded"}"#),
    ];

    println!("Webhook Processing Pipeline\n");

    for (provider, payload) in webhooks {
        match process_webhook(provider, payload) {
            Ok(event) => {
                println!("[{provider}] {}", event.summary);
                println!("  source:     {}", event.source);
                println!("  event_type: {}", event.event_type);
                println!("  actor:      {}", event.actor);
                for (k, v) in &event.metadata {
                    if !v.is_empty() {
                        println!("  {k}: {v}");
                    }
                }
                println!();
            }
            Err(e) => eprintln!("[{provider}] ERROR: {e}\n"),
        }
    }

    // Error cases
    println!("Error cases:");
    let bad_cases = vec![
        ("github", r#"{"action":"opened","sender":"alice"}"#, "missing repository"),
        ("stripe", r#"{"id":"pi_wrong","type":"charge.failed"}"#, "bad id prefix"),
        ("pagerduty", r#"{"message":"alert"}"#, "unknown provider"),
    ];

    for (provider, payload, desc) in bad_cases {
        match process_webhook(provider, payload) {
            Ok(_) => println!("  {desc}: unexpectedly succeeded"),
            Err(e) => println!("  {desc}: {e}"),
        }
    }
}
