// Variant: Functional-style field extraction using and_then chains
//
// Instead of early-return with ? at each field, this variant uses
// Option::and_then / Option::ok_or to chain extractions.
// Useful when you want to be explicit about the whole path.
//
// The core logic is identical to solution.rs — this is a style comparison.

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

#[derive(Debug)]
struct StripeEvent {
    id: String,
    event_type: String,
    amount_cents: Option<u64>,
    customer_id: Option<String>,
}

// Functional chaining style — explicit None → Err transformation
fn parse_stripe_functional(json: &str) -> Result<StripeEvent, String> {
    // Using ok_or_else instead of the ? shorthand
    let id = get_str(json, "id")
        .ok_or_else(|| String::from("missing required field: id"))
        .and_then(|id| {
            // Chain validation inline
            if id.starts_with("evt_") {
                Ok(id)
            } else {
                Err(format!("id must start with 'evt_', got '{id}'"))
            }
        })?;

    let event_type = get_str(json, "type")
        .ok_or_else(|| String::from("missing required field: type"))?;

    Ok(StripeEvent {
        id,
        event_type,
        amount_cents: get_u64(json, "amount_cents"),
        customer_id: get_str(json, "customer_id"),
    })
}

fn main() {
    let valid = r#"{"id":"evt_001","type":"payment.succeeded","amount_cents":1000}"#;
    let bad_id = r#"{"id":"pi_001","type":"payment.succeeded"}"#;
    let missing = r#"{"type":"payment.succeeded"}"#;

    println!("Functional style — explicit chaining with and_then\n");
    println!("valid: {:?}", parse_stripe_functional(valid));
    println!("bad id: {:?}", parse_stripe_functional(bad_id));
    println!("missing id: {:?}", parse_stripe_functional(missing));
}
