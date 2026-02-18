// Webhook Payload Handler — Starter
//
// Build a multi-provider webhook normalizer. See standard/README.md for full requirements.
//
// Run tests: rustc --test main.rs && ./main

// ─────────────────────────────────────────────────────────────────────────────
// JSON helpers — these stand in for serde_json in this exercise.
// In production Rust, you'd use #[derive(Deserialize)] and serde_json::from_str.
//
// Available helpers:
//   get_str(json, "field") -> Option<String>
//   get_u64(json, "field") -> Option<u64>
//   get_bool(json, "field") -> Option<bool>
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
    if rest.starts_with('"') { return None; } // string, not number
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

#[allow(dead_code)]
fn get_bool(json: &str, field: &str) -> Option<bool> {
    let key_true = format!(r#""{field}":true"#);
    let key_false = format!(r#""{field}":false"#);
    if json.contains(&key_true) { Some(true) }
    else if json.contains(&key_false) { Some(false) }
    else { None }
}

// ─────────────────────────────────────────────────────────────────────────────
// Your implementation goes here
// ─────────────────────────────────────────────────────────────────────────────

// TODO: Define GitHubEvent struct
// Fields: action (String), repository (String), sender (String), number (Option<u64>)

// TODO: Define StripeEvent struct
// Fields: id (String), event_type (String), amount_cents (Option<u64>),
//         currency (Option<String>), customer_id (Option<String>)

// TODO: Define SlackEvent struct
// Fields: event_type (String), channel (String), user (String), text (Option<String>)

// TODO: Define ProviderPayload enum
// Variants: GitHub(GitHubEvent), Stripe(StripeEvent), Slack(SlackEvent)

// TODO: Define InternalEvent struct
// Fields: source (String), event_type (String), actor (String),
//         summary (String), metadata (Vec<(String, String)>)

// TODO: Implement parse_provider_payload(provider: &str, json: &str) -> Result<ProviderPayload, String>

// TODO: Implement ProviderPayload::into_internal(self) -> InternalEvent

// TODO: Implement process_webhook(provider: &str, json: &str) -> Result<InternalEvent, String>

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
        // action is missing — should default to "unknown"
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
        // Some GitHub events (push, etc.) don't have a number
        let json = r#"{"action":"push","repository":"acme/api","sender":"eve"}"#;
        let payload = parse_provider_payload("github", json).unwrap();
        let event = payload.into_internal();
        // number metadata should be empty string when absent
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
        // Some Slack events (reactions, etc.) have no text
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
    println!("Webhook Payload Handler — run with: rustc --test main.rs && ./main");
}
